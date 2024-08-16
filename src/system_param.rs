use std::ops::{Deref, DerefMut};

use bevy_ecs::{
    archetype::Archetype,
    component::Tick,
    system::{ReadOnlySystemParam, SystemMeta, SystemParam, SystemParamItem},
    world::{unsafe_world_cell::UnsafeWorldCell, DeferredWorld, World},
};

use crate::{InitInWorld, WorldInit};

pub struct Init<'w, 's, T: SystemParam + InitInWorld>(SystemParamItem<'w, 's, T>);

impl<'w, 's, T: SystemParam + InitInWorld> Deref for Init<'w, 's, T> {
    type Target = SystemParamItem<'w, 's, T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: SystemParam + InitInWorld> DerefMut for Init<'_, '_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'w, 's, T: SystemParam + InitInWorld> Init<'w, 's, T> {
    /// Get the value of the parameter
    pub fn into_inner(self) -> SystemParamItem<'w, 's, T> {
        self.0
    }
}

// SAFETY: This doesn't add any more reads
unsafe impl<T: SystemParam + InitInWorld> ReadOnlySystemParam for Init<'_, '_, T> where
    T: ReadOnlySystemParam
{
}

// SAFETY: all methods are just delegated to `T`'s `SystemParam` implementation
// except `init_state` that also calls [`WorldInit::init`], that doesn't add any access
unsafe impl<T: SystemParam + InitInWorld> SystemParam for Init<'_, '_, T> {
    type State = T::State;

    type Item<'world, 'state> = Init<'world, 'state, T>;

    fn init_state(world: &mut World, system_meta: &mut SystemMeta) -> Self::State {
        world.init::<T>();
        T::init_state(world, system_meta)
    }

    /// For the specified [`Archetype`], registers the components accessed by this [`SystemParam`] (if applicable).a
    ///
    /// # Safety
    /// `archetype` must be from the [`World`] used to initialize `state` in `init_state`.
    #[inline]
    #[allow(unused_variables)]
    unsafe fn new_archetype(
        state: &mut Self::State,
        archetype: &Archetype,
        system_meta: &mut SystemMeta,
    ) {
        T::new_archetype(state, archetype, system_meta);
    }

    /// Applies any deferred mutations stored in this [`SystemParam`]'s state.
    /// This is used to apply [`Commands`] during [`apply_deferred`](crate::prelude::apply_deferred).
    ///
    /// [`Commands`]: crate::prelude::Commands
    #[inline]
    #[allow(unused_variables)]
    fn apply(state: &mut Self::State, system_meta: &SystemMeta, world: &mut World) {
        T::apply(state, system_meta, world);
    }

    /// Queues any deferred mutations to be applied at the next [`apply_deferred`](crate::prelude::apply_deferred).
    #[inline]
    #[allow(unused_variables)]
    fn queue(state: &mut Self::State, system_meta: &SystemMeta, world: DeferredWorld) {
        T::queue(state, system_meta, world);
    }

    /// Creates a parameter to be passed into a [`SystemParamFunction`].
    ///
    /// [`SystemParamFunction`]: super::SystemParamFunction
    ///
    /// # Safety
    ///
    /// - The passed [`UnsafeWorldCell`] must have access to any world data
    ///   registered in [`init_state`](SystemParam::init_state).
    /// - `world` must be the same `World` that was used to initialize [`state`](SystemParam::init_state).
    unsafe fn get_param<'world, 'state>(
        state: &'state mut Self::State,
        system_meta: &SystemMeta,
        world: UnsafeWorldCell<'world>,
        change_tick: Tick,
    ) -> Self::Item<'world, 'state> {
        Init(T::get_param(state, system_meta, world, change_tick))
    }
}
