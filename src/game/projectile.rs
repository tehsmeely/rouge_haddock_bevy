use super::tilemap::TilePosExt;
use crate::game::components::{DirectionalSpriteAnimation, Facing, MapDirection};
use bevy::asset::{AssetServer, Assets};
use bevy::core::{Time, Timer};
use bevy::ecs::change_detection::ResMut;
use bevy::ecs::entity::Entity;
use bevy::ecs::prelude::{Commands, Query, Res};
use bevy::math::{Vec2, Vec3};
use bevy::prelude::{Component, SpriteBundle, TextureAtlas};
use bevy::prelude::{SpriteSheetBundle, Transform};
use bevy_ecs_tilemap::TilePos;

#[derive(Component)]
pub struct Projectile {
    end_point: TilePos,
    speed: f32,
}

pub fn projectile_system(
    mut query: Query<(Entity, &mut Transform, &mut Projectile)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (entity, mut transform, mut projectile) in query.iter_mut() {
        let target_pos = projectile.end_point.to_world_pos(1f32).truncate();
        let direction: Vec2 =
            (Vec2::from(target_pos) - transform.translation.truncate()).normalize();

        transform.translation +=
            (direction * projectile.speed * time.delta().as_secs_f32()).extend(0f32);

        let remaining_distance = (Vec2::from(target_pos) - transform.translation.truncate());

        //We know we've overshot when the signs of remaining_distance are opposite
        let overshot = remaining_distance.x.signum() != direction.x.signum()
            && remaining_distance.y.signum() != direction.y.signum();
        if overshot {
            commands.entity(entity).despawn();
        }
    }
}

pub fn spawn_projectile(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    start_pos: Vec3,
    end_point: TilePos,
) {
    let texture_handle = asset_server.load("sprites/projectile_spritesheet.png");
    let atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(20.0, 20.0), 4, 4);
    let atlas_handle = texture_atlases.add(atlas);
    commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: atlas_handle.clone(),
            transform: Transform::from_translation(start_pos),
            ..Default::default()
        })
        .insert(Timer::from_seconds(0.1, true))
        .insert((Facing(MapDirection::Down)))
        .insert(DirectionalSpriteAnimation::new(4, 0))
        .insert(Projectile {
            end_point,
            speed: 150.,
        });
}
