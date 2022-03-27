use crate::asset_handling::asset::ImageAsset;
use crate::asset_handling::ImageAssetStore;
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
    image_assets: &Res<ImageAssetStore>,
    texture_atlases: &mut ResMut<Assets<TextureAtlas>>,
    num_sharks: usize,
    cell_map: &CellMap<i32>,
    exclude_positions: Option<&Vec<(i32, i32)>>,
) -> Vec<(i32, i32)> {
    //let texture_handle = asset_server.load("sprites/shark_spritesheet.png");
    let texture_handle = image_assets.get(&ImageAsset::SharkSpritesheet);
    let atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(64.0, 64.0), 4, 4);
    let atlas_handle = texture_atlases.add(atlas);
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
    image_assets: &Res<ImageAssetStore>,
    texture_atlases: &mut ResMut<Assets<TextureAtlas>>,
    num_crabs: usize,
    cell_map: &CellMap<i32>,
    exclude_positions: Option<&Vec<(i32, i32)>>,
) {
    //let texture_handle = asset_server.load("sprites/crab_spritesheet.png");
    let texture_handle = image_assets.get(&ImageAsset::CrabSpritesheet);
    let atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(64.0, 64.0), 4, 1);
    let atlas_handle = texture_atlases.add(atlas);
    let spawn_positions = cell_map.distribute_points_by_cost(num_crabs, exclude_positions);
    for (x, y) in spawn_positions.into_iter() {
        let tile_pos = TilePos(x as u32, y as u32);
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
}
