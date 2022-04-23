use super::tilemap::TilePosExt;
use crate::asset_handling::asset::{ImageAsset, TextureAtlasAsset};
use crate::asset_handling::{ImageAssetStore, TextureAtlasStore};
use crate::game::components::{
    AnimationTimer, DirectionalSpriteAnimation, Facing, Health, MapDirection, TileType,
};
use crate::game::events::GameEvent;
use crate::game::tilemap::HasTileType;
use crate::game::turn::{GamePhase, GlobalTurnCounter, TurnCounter};
use bevy::asset::{AssetServer, Assets};
use bevy::core::{Time, Timer};
use bevy::ecs::change_detection::ResMut;
use bevy::ecs::entity::Entity;
use bevy::ecs::event::EventWriter;
use bevy::ecs::prelude::{Commands, Local, Query, Res, With};
use bevy::math::{Vec2, Vec3};
use bevy::prelude::{Component, TextureAtlas};
use bevy::prelude::{SpriteSheetBundle, Transform};
use bevy_ecs_tilemap::{MapQuery, TilePos};
use log::debug;
use num::Signed;
use std::collections::HashMap;

pub enum ProjectileEvent {
    ProjectileLaunched,
    ProjectileHit(Entity),
}

#[derive(Component)]
pub struct Projectile {
    end_point: TilePos,
    speed: f32,
    finish_point_threshold: f32,
    damage: usize,
    end_target_entity: Option<Entity>,
}

impl Projectile {
    fn new(end_point: TilePos, speed: f32, end_target_entity: Option<Entity>) -> Self {
        Self {
            end_point,
            speed,
            finish_point_threshold: 32.0,
            damage: 1usize,
            end_target_entity,
        }
    }
}

/// This system only progresses turn phase if all projectiles have ceased to exist
pub fn projectile_watcher_system(
    projectile_query: Query<Entity, With<Projectile>>,
    global_turn_counter: Res<GlobalTurnCounter>,
    mut local_turn_counter: Local<TurnCounter>,
    mut game_event_writer: EventWriter<GameEvent>,
    mut frame_delay: Local<usize>,
) {
    // [frame_delay] protects against the system running when we enter the PlayerPowerEffect phase but before
    // the stage has spawned the projectile - because spawns from [Commands] happen in a later stage
    // waiting to see no projectiles twice will cause one frame cycle for cases where we don't fire a projectile
    // If this is a problem, this system would be run in a stage AFTER the spawning happens
    if global_turn_counter.can_take_turn(&local_turn_counter, GamePhase::PlayerPowerEffect) {
        if projectile_query.is_empty() {
            if *frame_delay > 1 {
                game_event_writer.send(GameEvent::PhaseComplete(GamePhase::PlayerPowerEffect));
                local_turn_counter.incr();
            } else {
                *frame_delay += 1;
            }
        } else {
            *frame_delay = 0;
        }
    }
}

pub fn projectile_system(
    mut query: Query<(Entity, &mut Transform, &mut Projectile)>,
    mut health_query: Query<&mut Health>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (entity, mut transform, projectile) in query.iter_mut() {
        let target_pos = projectile.end_point.to_world_pos(1f32).truncate();
        let distance_to_travel = target_pos - transform.translation.truncate();
        let direction: Vec2 = distance_to_travel.normalize();

        let distance_this_step = direction * projectile.speed * time.delta().as_secs_f32();

        transform.translation += distance_this_step.extend(0f32);

        if (transform.translation.truncate() - target_pos).length()
            < projectile.finish_point_threshold
        {
            debug!("Despawning projectile: {:?}", entity);
            commands.entity(entity).despawn();
            if let Some(damage_entity) = projectile.end_target_entity {
                if let Ok(mut health) = health_query.get_mut(damage_entity) {
                    health.decr_by(projectile.damage);
                }
            }
        }
    }
}

// TODO: Move me
fn get_tiletype(t: &TilePos, q: &Query<&HasTileType>, map_query: &mut MapQuery) -> TileType {
    let tile_entity = map_query.get_tile_entity(*t, 0, 0);
    match tile_entity {
        Ok(entity) => {
            let type_ = q.get(entity);
            match type_ {
                Ok(tt) => tt.0.clone(),
                Err(_) => TileType::WALL,
            }
        }
        Err(_) => TileType::WALL,
    }
}

pub enum ProjectileFate {
    EndNoTarget(TilePos),
    EndHitTarget((TilePos, Entity)),
}

impl ProjectileFate {
    pub fn tile_pos(&self) -> &TilePos {
        match self {
            Self::EndNoTarget(tp) => tp,
            Self::EndHitTarget((tp, _entity)) => tp,
        }
    }
    pub fn entity(&self) -> Option<Entity> {
        match self {
            Self::EndNoTarget(_tp) => None,
            Self::EndHitTarget((_tp, entity)) => Some(*entity),
        }
    }
}

pub fn scan_to_endpoint<T: Component>(
    from: &TilePos,
    direction: &MapDirection,
    query: &Query<(Entity, &TilePos), With<T>>,
    map_query: &mut MapQuery,
    tiletype_query: &Query<&HasTileType>,
) -> ProjectileFate {
    let enemies_on_same_row_or_column: HashMap<TilePos, Entity> = {
        let mut enemies = HashMap::with_capacity(5);
        for (entity, tilepos) in query.iter() {
            if tilepos.0 == from.0 || tilepos.1 == from.1 {
                enemies.insert(*tilepos, entity);
            }
        }
        enemies
    };
    let step = direction.to_unit_translation().truncate();
    let mut test_pos = *from;
    let mut i = 0;
    println!(
        "Calculating projectile from: {:?} in direction {:?}",
        from, step
    );
    loop {
        i += 1;
        if i > 50 {
            panic!("Projectile loop did not terminate");
        }
        tilepos_add_vec(&mut test_pos, &step);
        println!("Testing pos: {:?}", test_pos);
        let tile_type = get_tiletype(&test_pos, tiletype_query, map_query);
        if tile_type.can_enter() {
            match enemies_on_same_row_or_column.get(&test_pos) {
                Some(entity) => return ProjectileFate::EndHitTarget((test_pos, *entity)),
                None => (),
            }
        } else {
            return ProjectileFate::EndNoTarget(test_pos);
        }
    }
}

fn tilepos_add_vec(tilepos: &mut TilePos, vec: &Vec2) {
    //TODO use the i32 add function defined somewhere else for each
    if vec.x.is_negative() {
        tilepos.0 -= vec.x.abs() as i32 as u32;
    } else {
        tilepos.0 += vec.x as i32 as u32;
    }
    if vec.y.is_negative() {
        tilepos.1 -= vec.y.abs() as i32 as u32;
    } else {
        tilepos.1 += vec.y as i32 as u32;
    }
}

pub fn spawn_projectile(
    commands: &mut Commands,
    atlases: &Res<TextureAtlasStore>,
    direction: MapDirection,
    start_pos: Vec3,
    end_point: TilePos,
    end_target_entity: Option<Entity>,
) {
    let atlas_handle = atlases.get(&TextureAtlasAsset::ProjectileSpritesheet);
    commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: atlas_handle,
            transform: Transform::from_translation(start_pos),
            ..Default::default()
        })
        .insert(AnimationTimer(Timer::from_seconds(0.1, true)))
        .insert(Facing(direction))
        .insert(DirectionalSpriteAnimation::new(4, 0, 0))
        .insert(Projectile::new(end_point, 500., end_target_entity));
}
