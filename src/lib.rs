#![warn(clippy::undocumented_unsafe_blocks, missing_docs)]
//! Crate provides a way for tracking initialized to the world types,
//! automatically initializing components into the world on add
//! and conveniently adding system during runtime.
//!
//! # Motivation
//!
//! Sometimes it's impossible, or will be way too hard, to initialize everything on the
//! stage of building the app.
//!
//! For example:
//! ```
//! # use bevy_ecs::prelude::*;
//! # use bevy_ecs::world::DeferredWorld;
//! 
//! #[derive(Component)]
//! struct GenericComponent<A, B>(A, B)
//!     where A: Send + Sync + 'static, B: Send + Sync + 'static;
//!
//! fn system_operating_on_generic_component<A, B>(query: Query<&GenericComponent<A, B>>)
//!     where A: Send + Sync + 'static, B: Send + Sync + 'static 
//! {
//!     // do_something ...
//! }
//! ```
//!
//! Usual way to use generic types in bevy's ecs is to provide a method to register all
//! data, related to the generic parameter, to the world.
//! For example, [`add_event`](https://docs.rs/bevy/latest/bevy/app/struct.App.html#method.add_event)
//! registers all necessary data to the app so that it is possible to work with the registered event.
//! This way of handling generics should still be preferred, because it avoids unnecessary runtime checks.
//! 
//! But in the situation above, for it to be possible, user should register every possible combination of two generics 
//! beforehand for the program to work. And situation gets progressively worse with the increase in amount of generics.
//! 
//! So, with this library you can do this:
//! ```
//! # use bevy_ecs::prelude::*;
//! # use bevy_ecs::world::DeferredWorld;
//! # use bevy_app::Update;
//! use bevy_register_in_world::prelude::*;
//!
//! #[derive(ComponentAutoRegister)]
//! struct GenericComponent<A, B>(A, B)
//!     where A: Send + Sync + 'static, B: Send + Sync + 'static;
//!
//! impl<A, B> RegisterInWorld for GenericComponent<A, B> 
//!     where A: Send + Sync + 'static, B: Send + Sync + 'static
//! {
//!     fn register(mut world: DeferredWorld) {
//!         world.add_systems(Update, system_operating_on_generic_component::<A, B>);
//!     }
//! }
//! 
//! fn system_operating_on_generic_component<A, B>(query: Query<&GenericComponent<A, B>>) 
//!     where A: Send + Sync + 'static, B: Send + Sync + 'static
//! {
//!     // do_something ...
//! }
//! ```
//! And when component with unique combination of generics is added,
//! `register` is called during it's `on_add` hook.

extern crate self as bevy_register_in_world;

pub mod add_systems;
#[cfg(feature = "bevy_app")]
pub mod app;
pub mod component;
// unsure if this is the right thing to do
//pub mod system_param;

use bevy_ecs::{
    system::Resource,
    world::{DeferredWorld, World},
};
use bevy_utils::{hashbrown::HashSet, NoOpHash};
use std::any::TypeId;

pub mod prelude {
    //! Prelude module
    
    pub use crate::{
        RegisterExtension, RegisterInWorld,
        add_systems::{AddSystems, WorldAddSystems},
        component::ComponentAutoRegister,
    };

    #[cfg(feature = "bevy_app")]
    pub use crate::app::RegisterInWorldPlugin;
}


/// Types that can be registered to the world.
pub trait RegisterInWorld: 'static {
    /// Register type to the world.
    /// 
    /// Since this crate is primarily useful for 
    /// [automatic component registration](bevy_register_in_world::component::ComponentAutoRegister),
    /// which registers components during `on_add` hook, it was decided to use 
    /// [`DeferredWorld`] directly as an argument. You can still use [`DeferredWorld::commands`].
    /// Calling [`World::register`] will immediately flush commands after call to `register`.
    fn register(world: DeferredWorld);
}

type TypeIdSet = HashSet<TypeId, NoOpHash>;

/// Stores a `HashSet` of types that were registered into the world using [`RegisterInWorld`] trait.
#[derive(Resource, Default)]
pub struct RegisteredTypes {
    types: TypeIdSet,
}

impl RegisteredTypes {
    /// Returns wether the type is registered or not.
    #[inline]
    pub fn is_registered<T: RegisterInWorld>(&self) -> bool {
        self.types.contains(&TypeId::of::<T>())
    }

    /// If type should be registered, returns `true`.
    ///
    /// If type was already registered, returns `false`.
    #[inline]
    pub fn register<T: RegisterInWorld>(&mut self) -> bool {
        self.types.insert(TypeId::of::<T>())
    }
}

/// Trait that is implemented for world and app types for convenience of registering.
pub trait RegisterExtension {
    /// Register the specified type into the world using [`RegisterInWorld`].
    /// Won't register again if type was already registered to the world.
    fn register<T: RegisterInWorld>(&mut self);
}

impl RegisterExtension for DeferredWorld<'_> {
    fn register<T: RegisterInWorld>(&mut self) {
        let mut initialized = self.resource_mut::<RegisteredTypes>();

        if initialized.register::<T>() {
            T::register(self.reborrow());
        }
    }
}

impl RegisterExtension for World {
    fn register<T: RegisterInWorld>(&mut self) {
        let mut initialized = self.get_resource_or_insert_with::<RegisteredTypes>(Default::default);

        if initialized.register::<T>() {
            T::register(self.into());
            self.flush_commands();
        }
    }
}
