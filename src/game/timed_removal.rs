use crate::game::components::DirectionalSpriteAnimationSpecial;
use bevy::app::App;
use bevy::core::{Time, Timer};
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::prelude::{Commands, Query, Res};
use bevy::prelude::Plugin;
use std::marker::PhantomData;
use std::time::Duration;

pub struct TimedRemovalPlugin;

impl Plugin for TimedRemovalPlugin {
    fn build(&self, app: &mut App) {
        // Building this plugin we add all the types we expect to used TimedRemoval for
        // It's generic but we need to add a system for every T
        app.add_system(timed_removal_system::<DirectionalSpriteAnimationSpecial>);
    }
}

#[derive(Component)]
pub struct TimedRemoval<T: Component> {
    pub timer: Timer,
    _phantom: PhantomData<T>,
}

impl<T: Component> TimedRemoval<T> {
    pub fn new(duration: Duration) -> Self {
        Self {
            timer: Timer::new(duration, false),
            _phantom: PhantomData,
        }
    }
}

/// This system allows use of a [TimedRemoval<T>] to have component T removed after a given time
/// Systems need to be added for every [T]
fn timed_removal_system<T: Component>(
    mut q: Query<(Entity, &mut TimedRemoval<T>)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (entity, mut timed_removal) in q.iter_mut() {
        timed_removal.timer.tick(time.delta());
        if timed_removal.timer.just_finished() {
            println!("Removing Component after timed removal");
            commands
                .entity(entity)
                .remove::<T>()
                .remove::<TimedRemoval<T>>();
        }
    }
}
