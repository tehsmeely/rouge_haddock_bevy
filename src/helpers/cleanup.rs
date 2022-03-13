use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::prelude::{Commands, Query, With};
use bevy::prelude::DespawnRecursiveExt;

pub fn recursive_cleanup<T: Component>(mut commands: Commands, query: Query<Entity, With<T>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
