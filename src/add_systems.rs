//! Adding

use bevy_consumable_event::{ConsumableEventReader, ConsumableEvents};
use bevy_ecs::{
    event::Event,
    schedule::{InternedScheduleLabel, IntoSystemConfigs, ScheduleLabel, Schedules, SystemConfigs},
    system::ResMut,
    world::{DeferredWorld, World},
};

/// Schedule that is executed after [`Last`] schedule. 
/// During this schedule *only one system* should be called - [`add_requested_systems`].
/// It's not recommended to add any other systems to it, that reduces potential parallelism targets.
/// This schedule is only used for adding systems to other schedules, so adding systems to it
/// using [`AddSystems`] event is impossible.
#[derive(ScheduleLabel, Debug, PartialEq, Eq, Hash, Clone)]
pub struct AddingSystems;

/// Adds systems to the schedule during [`AddingSystems`] schedule.
#[derive(Event)]
pub struct AddSystems(InternedScheduleLabel, SystemConfigs);

impl AddSystems {
    /// Create instance of the event. Will add `systems` in `schedule` during the run of [`AddingSystems`] schedule
    /// # Panics
    /// If trying to use [`AddingSystems`] as label to add systems to. 
    pub fn new<M>(schedule: impl ScheduleLabel, systems: impl IntoSystemConfigs<M>) -> Self {
        let schedule = schedule.intern();
        assert!(!schedule.as_dyn_eq().dyn_eq(&AddingSystems), "Trying to add systems to `AddingSystems` schedule using `AddSystems` event. This is not allowed since `AddSystems` events are consumed during `AddingSystems` schedule.");
        AddSystems(schedule, systems.into_configs())
    }
}

/// Consumes all [`AddSystems`] events, and adds it to the needed schedules.
/// This should *only* run during [`AddingSystems`] schedules. 
/// If you're not using [`RegisterInWorldPlugin`](bevy_register_in_world::app::RegisterInWorldPlugin),
/// add this system to the [`AddingSystems`] schedule, and not 
/// 
/// Note that events should be sent using [`ConsumableEventWriter`](bevy_consumable_event::ConsumableEventWriter).
/// 
pub fn add_requested_systems(
    mut events: ConsumableEventReader<AddSystems>,
    mut schedules: ResMut<Schedules>,
) {
    for AddSystems(schedule, systems) in events.read_and_consume_all() {
        schedules.add_systems(schedule, systems);
    }
}

/// Convenience trait to add systems to the world.
pub trait WorldAddSystems {
    /// Sends [`AddSystems`] event.
    fn add_systems<M>(&mut self, schedule: impl ScheduleLabel, systems: impl IntoSystemConfigs<M>);
}

impl WorldAddSystems for DeferredWorld<'_> {
    fn add_systems<M>(&mut self, schedule: impl ScheduleLabel, systems: impl IntoSystemConfigs<M>) {
        self.resource_mut::<ConsumableEvents<AddSystems>>()
            .send(AddSystems::new(schedule, systems));
    }
}

impl WorldAddSystems for World {
    #[inline]
    fn add_systems<M>(&mut self, schedule: impl ScheduleLabel, systems: impl IntoSystemConfigs<M>) {
        Into::<DeferredWorld>::into(self).add_systems(schedule, systems)
    }
}
