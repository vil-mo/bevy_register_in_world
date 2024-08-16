Crate provides a way for tracking initialized to the world types,
automatically initializing components into the world on add
and conveniently adding system during runtime.

# Motivation
Sometimes it's impossible, or will be way too hard, to initialize everything on the
stage of building the app.
For example:
```rust
#[derive(Component)]
struct GenericComponent<A, B>(A, B)
    where A: Send + Sync + 'static, B: Send + Sync + 'static;

fn system_operating_on_generic_component<A, B>(query: Query<&GenericComponent<A, B>>)
    where A: Send + Sync + 'static, B: Send + Sync + 'static 
{
    // do_something ...
}
```

Usual way to use generic types in bevy ecs is to provide a method to register all
data related to the generic parameter to the world.
For example, [`add_event`](https://docs.rs/bevy/latest/bevy/app/struct.App.html#method.add_event)
registers all necessary data to the app so that it is possible to work with the registered event.
This way of handling generics should still be preferred, because it avoids unnecessary runtime checks.

But in the situation above, for it to be possible, user should register every possible combination of two generics 
beforehand for the program to work.

So, with this library you can do this:
```rust
use bevy_register_in_world::prelude::*;

#[derive(ComponentAutoRegister)]
struct GenericComponent<A, B>(A, B)
    where A: Send + Sync + 'static, B: Send + Sync + 'static;

impl<A, B> RegisterInWorld for GenericComponent<A, B> 
    where A: Send + Sync + 'static, B: Send + Sync + 'static
{
    fn register(mut world: DeferredWorld) {
        world.add_systems(Update, system_operating_on_generic_component::<A, B>);
    }
}

fn system_operating_on_generic_component<A, B>(query: Query<&GenericComponent<A, B>>) 
    where A: Send + Sync + 'static, B: Send + Sync + 'static
{
    // do_something ...
}
```
And when component with unique combination of generics is added,
`register` is called during it's `on_add` hook.
