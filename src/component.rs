//! Component stuff

use bevy_ecs::{
    component::Component,
    world::DeferredWorld,
};
use crate::{RegisterExtension, RegisterInWorld};

pub use bevy_register_in_world_macros::ComponentAutoRegister;

/// Implemented for components that are automatically registered to the world.
/// 
/// You can derive this trait and use the same attributes as the regular [`Component`] derive.
/// In other words, you can still specify storage type and different hooks. 
/// [`on_add`] hook will be called after the registration.
pub trait ComponentAutoRegister: Component + RegisterInWorld {}

/// Should be called during [`on_add`] hook for every component that should be 
/// automatically registered to the world when added.
pub fn register_on_add<T: ComponentAutoRegister>(
    mut world: DeferredWorld,
) {
    world.register::<T>();
}

// macro_rules! wrapper_init {
//     ($t:ty, $($c:path),*) => {
//         impl<T: bevy_init_in_world::InitInWorld $(+ $c)*> bevy_init_in_world::InitInWorld for $t {
//             fn to_init_id() -> std::any::TypeId {
//                 <T as bevy_init_in_world::InitInWorld>::to_init_id()
//             }

//             fn init(world: &mut bevy_ecs::world::World) {
//                 <T as bevy_init_in_world::InitInWorld>::init(world);
//             }
//         }
//     };
// }

// wrapper_init!(Init<'_, '_, T>, SystemParam);
// wrapper_init!(Ref<'_, T>,);
// wrapper_init!(Mut<'_, T>,);
// wrapper_init!(Option<T>,);
// wrapper_init!(PhantomData<T>,);
// wrapper_init!(NonSend<'_, T>,);
// wrapper_init!(NonSendMut<'_, T>,);
// wrapper_init!(Res<'_, T>, Resource);
// wrapper_init!(ResMut<'_, T>, Resource);
// wrapper_init!(EventReader<'_, '_, T>, Event);
// wrapper_init!(EventWriter<'_, T>, Event);
// wrapper_init!(Deferred<'_, T>, SystemBuffer);
