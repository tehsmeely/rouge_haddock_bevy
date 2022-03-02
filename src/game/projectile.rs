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
use log::debug;

pub enum ProjectileEvent {
    ProjectileLaunched,
    ProjectileHit(Entity),
}

#[derive(Component)]
pub struct Projectile {
    end_point: TilePos,
    speed: f32,
    finish_point_threshold: f32,
}

impl Projectile {
    fn new(end_point: TilePos, speed: f32) -> Self {
        Self {
            end_point,
            speed,
            finish_point_threshold: 64f32 / 100.0,
        }
    }
}

pub fn projectile_system(
    mut query: Query<(Entity, &mut Transform, &mut Projectile)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (entity, mut transform, mut projectile) in query.iter_mut() {
        let target_pos = projectile.end_point.to_world_pos(1f32).truncate();
        let distance_to_travel = (Vec2::from(target_pos) - transform.translation.truncate());
        let direction: Vec2 = distance_to_travel.normalize();

        let distance_this_step = direction * projectile.speed * time.delta().as_secs_f32();

        transform.translation += distance_this_step.extend(0f32);

        if (transform.translation.truncate() - target_pos).length()
            < projectile.finish_point_threshold
        {
            debug!("Despawning projectile: {:?}", entity);
            commands.entity(entity).despawn();
        }

        if distance_this_step.x.abs() > distance_to_travel.x.abs()
            && distance_this_step.y.abs() > distance_to_travel.y.abs()
        {
            debug!("(2) Despawning projectile: {:?}", entity);
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
        .insert(Projectile::new(end_point, 150.));
}
