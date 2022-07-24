use crate::asset_handling::asset::{TextureAtlasAsset};
use crate::asset_handling::{TextureAtlasStore};
use crate::game::components::{
    CanMoveDistance, GameOnly, MapDirection, MoveWeighting, Player, SimpleTileResidentBundle,
    TileResidentBundle,
};
use crate::game::projectile::ProjectileFate;
use crate::game::tilemap::{HasTileType, TilePosExt};
use crate::game::timed_removal::TimedDespawn;
use crate::map_gen::cell_map::CellMap;
use bevy::prelude::*;
use bevy_ecs_tilemap::{MapQuery, TilePos};
use std::time::Duration;

#[derive(Debug, Component)]
pub struct Enemy {
    pub can_attack_directly: bool,
}

#[derive(Debug, Component)]
pub struct Shark;

#[derive(Debug, Component)]
pub struct Crab;

#[derive(Debug, Component, Default)]
pub struct Jellyfish {
    pub state: JellyfishState,
}

#[derive(Debug, Component)]
pub struct JellyfishLightningTile;

#[derive(Debug)]
pub enum JellyfishState {
    Normal,
    Charging(MapDirection),
    Recharging(usize),
}

impl Default for JellyfishState {
    fn default() -> Self {
        Self::Recharging(3)
    }
}

pub fn add_sharks(
    commands: &mut Commands,
    atlases: &Res<TextureAtlasStore>,
    num_sharks: usize,
    cell_map: &CellMap<i32>,
    exclude_positions: Option<&Vec<(i32, i32)>>,
) -> Vec<(i32, i32)> {
    let atlas_handle = atlases.get(&TextureAtlasAsset::SharkSpritesheet);
    let spawn_positions = cell_map.distribute_points_by_cost(num_sharks, exclude_positions);
    for (x, y) in spawn_positions.iter() {
        let tile_pos = TilePos(*x as u32, *y as u32);
        commands
            .spawn_bundle(TileResidentBundle::new(
                1,
                tile_pos,
                atlas_handle.clone(),
                0,
            ))
            .insert(Enemy {
                can_attack_directly: true,
            })
            .insert(CanMoveDistance::all(1))
            .insert(MoveWeighting::all(1.0))
            .insert(Shark);
    }
    spawn_positions
}

pub fn add_crabs(
    commands: &mut Commands,
    atlases: &Res<TextureAtlasStore>,
    num_crabs: usize,
    cell_map: &CellMap<i32>,
    exclude_positions: Option<&Vec<(i32, i32)>>,
) -> Vec<(i32, i32)> {
    let atlas_handle = atlases.get(&TextureAtlasAsset::CrabSpritesheet);
    let spawn_positions = cell_map.distribute_points_by_cost(num_crabs, exclude_positions);
    for (x, y) in spawn_positions.iter() {
        let tile_pos = TilePos(*x as u32, *y as u32);
        commands
            .spawn_bundle(SimpleTileResidentBundle::new(
                1,
                tile_pos,
                atlas_handle.clone(),
                4,
                None,
            ))
            .insert(Enemy {
                can_attack_directly: true,
            })
            .insert(CanMoveDistance::updown_leftright(1, 2))
            .insert(MoveWeighting::updown_leftright(0.1, 1.0))
            .insert(Crab);
    }
    spawn_positions
}

pub fn add_jellyfish(
    commands: &mut Commands,
    atlases: &Res<TextureAtlasStore>,
    num_jellies: usize,
    cell_map: &CellMap<i32>,
    exclude_positions: Option<&Vec<(i32, i32)>>,
) -> Vec<(i32, i32)> {
    let atlas_handle = atlases.get(&TextureAtlasAsset::JellySpritesheet);
    let spawn_positions = cell_map.distribute_points_by_cost(num_jellies, exclude_positions);
    for (x, y) in spawn_positions.iter() {
        let tile_pos = TilePos(*x as u32, *y as u32);
        commands
            .spawn_bundle(SimpleTileResidentBundle::new(
                1,
                tile_pos,
                atlas_handle.clone(),
                4,
                Some(Timer::from_seconds(0.2, true)),
            ))
            .insert(Enemy {
                can_attack_directly: false,
            })
            .insert(CanMoveDistance::updown_leftright(1, 1))
            .insert(MoveWeighting::updown_leftright(1.0, 1.0))
            .insert(Jellyfish::default());
    }
    spawn_positions
}

pub fn jelly_lightning_projection(
    jelly_position: &TilePos,
    firing_direction: &MapDirection,
    player_query: &Query<(Entity, &TilePos), With<Player>>,
    map_query: &mut MapQuery,
    tiletype_query: &Query<&HasTileType>,
) -> (usize, Option<Entity>) {
    let projectile_fate = super::projectile::scan_to_endpoint(
        jelly_position,
        firing_direction,
        player_query,
        map_query,
        tiletype_query,
        false,
    );
    let (final_tilepos, hit_player) = match projectile_fate {
        ProjectileFate::EndNoTarget(last_tile_pos) => (last_tile_pos, None),
        ProjectileFate::EndHitTarget((last_tile_pos, player_entity)) => {
            (last_tile_pos, Some(player_entity))
        }
    };
    (jelly_position.distance_to(&final_tilepos) - 1, hit_player)
}
pub fn spawn_jelly_lightning(
    commands: &mut Commands,
    atlases: &TextureAtlasStore,
    start_pos: TilePos,
    length: usize,
    direction: MapDirection,
) -> Vec<Entity> {
    let rotation = direction.to_rotation_from_right_zero();

    let mut entities = Vec::new();
    let mut tilepos = start_pos;

    for i in 0..length {
        let index = if i == 0 {
            0
        } else if i == length - 1 {
            2
        } else {
            1
        };
        let mut transform = Transform::from_translation(tilepos.to_world_pos(11.0));
        transform.rotate(Quat::from_rotation_z(rotation));
        let entity = commands
            .spawn_bundle(SpriteSheetBundle {
                sprite: TextureAtlasSprite {
                    index,
                    ..Default::default()
                },
                texture_atlas: atlases.get(&TextureAtlasAsset::JellyLightning),
                transform,
                ..Default::default()
            })
            .insert(GameOnly)
            .insert(JellyfishLightningTile)
            .insert(TimedDespawn::new(Duration::from_millis(400)))
            .id();
        entities.push(entity);
        tilepos = tilepos.add(direction.to_pos_move());
    }
    entities
}

impl Jellyfish {
    pub const CHARGE_CHANCE: f64 = 0.5;
    pub const RECHARGE_TURNS: usize = 1;

    pub fn can_move(&self) -> bool {
        match self.state {
            JellyfishState::Normal => true,
            JellyfishState::Recharging(remaining) => remaining < Jellyfish::RECHARGE_TURNS,
            JellyfishState::Charging(_) => false,
        }
    }
}
