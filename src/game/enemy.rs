use crate::asset_handling::asset::{ImageAsset, TextureAtlasAsset};
use crate::asset_handling::{ImageAssetStore, TextureAtlasStore};
use crate::game::components::{
    CanMoveDistance, MoveWeighting, SimpleTileResidentBundle, TileResidentBundle,
};
use crate::map_gen::cell_map::CellMap;
use bevy::prelude::*;
use bevy_ecs_tilemap::TilePos;

#[derive(Debug, Component)]
pub struct Enemy;

#[derive(Debug, Component)]
pub struct Shark;

#[derive(Debug, Component)]
pub struct Crab;

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
            .insert(Enemy {})
            .insert(CanMoveDistance::all(1))
            .insert(MoveWeighting::all(1.0))
            .insert(Shark {});
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
            ))
            .insert(Enemy {})
            .insert(CanMoveDistance::updown_leftright(1, 2))
            .insert(MoveWeighting::updown_leftright(0.1, 1.0))
            .insert(Crab {});
    }
    spawn_positions
}
