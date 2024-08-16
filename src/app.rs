//! Logic for app

use bevy_app::{App, Last, MainScheduleOrder, Plugin, SubApp};
use bevy_consumable_event::ConsumableEventApp;

use crate::{
    add_systems::{add_requested_systems, AddSystems, AddingSystems},
    RegisterExtension, RegisteredTypes,
};

/// Adds functionality to be able to register types into the world 
/// and add system during runtime.
pub struct RegisterInWorldPlugin;

impl Plugin for RegisterInWorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RegisteredTypes>();

        // Adding systems
        app.add_persistent_consumable_event::<AddSystems>();

        app.init_schedule(AddingSystems);
        app.world_mut()
            .resource_mut::<MainScheduleOrder>()
            .insert_after(Last, AddingSystems);
        app.add_systems(AddingSystems, add_requested_systems);
    }
}

impl RegisterExtension for App {
    fn register<T: crate::RegisterInWorld>(&mut self) {
        self.world_mut().register::<T>();
    }
}

impl RegisterExtension for SubApp {
    fn register<T: crate::RegisterInWorld>(&mut self) {
        self.world_mut().register::<T>();
    }
}
