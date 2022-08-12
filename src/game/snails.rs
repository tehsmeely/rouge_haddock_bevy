use crate::asset_handling::asset::TextureAtlasAsset;
use crate::asset_handling::TextureAtlasStore;
use crate::game::components::{AnimationTimer, GameOnly, Player, SimpleSpriteAnimation};
use crate::game::events::InfoEvent;
use crate::game::game::SnailsCollectedThisRun;
use crate::game::tilemap::TilePosExt;
use crate::map_gen::cell_map::CellMap;
use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TilePos;

/// Snails serve as the collectable resource, are not a moving tile resident/enemy
#[derive(Debug, Component)]
pub struct Snail;

fn add_snails(
    num_snails: usize,
    commands: &mut Commands,
    atlases: &Res<TextureAtlasStore>,
    cell_map: &CellMap<i32>,
    exclude_positions: Option<&Vec<(i32, i32)>>,
) -> Vec<(i32, i32)> {
    let atlas_handle = atlases.get(&TextureAtlasAsset::SnailSpritesheet);
    let spawn_positions = cell_map.distribute_points_by_cost(num_snails, exclude_positions);
    for (x, y) in spawn_positions.iter() {
        let tile_pos = TilePos {
            x: *x as u32,
            y: *y as u32,
        };
        let start_pos = tile_pos.to_world_pos(9.0);
        let mut transform = Transform::from_translation(start_pos);
        transform.scale = Vec3::splat(0.7);
        commands
            .spawn_bundle(SpriteSheetBundle {
                texture_atlas: atlas_handle.clone(),
                transform,
                ..Default::default()
            })
            .insert(tile_pos)
            .insert(AnimationTimer(Timer::from_seconds(0.1, true)))
            .insert(SimpleSpriteAnimation::new(0, 4))
            .insert(GameOnly {})
            .insert(Snail {});
    }
    spawn_positions
}

pub fn choose_number_of_and_spawn_snails(
    commands: &mut Commands,
    texture_atlases: &Res<TextureAtlasStore>,
    cell_map: &CellMap<i32>,
    exclude_positions: Option<&Vec<(i32, i32)>>,
) -> (usize, Vec<(i32, i32)>) {
    let num_snails = 2;

    let spawned_positions = if num_snails > 0 {
        add_snails(
            num_snails,
            commands,
            texture_atlases,
            cell_map,
            exclude_positions,
        )
    } else {
        Vec::new()
    };
    (num_snails, spawned_positions)
}

pub fn snail_pickup_system(
    mut commands: Commands,
    snail_query: Query<(Entity, &TilePos), With<Snail>>,
    player_query: Query<&TilePos, With<Player>>,
    mut snail_shells_collected_this_run: ResMut<SnailsCollectedThisRun>,
    mut info_event_writer: EventWriter<InfoEvent>,
) {
    for player_pos in player_query.iter() {
        for (snail_entity, snail_pos) in snail_query.iter() {
            if snail_pos == player_pos {
                snail_shells_collected_this_run.0 += 1;
                commands.entity(snail_entity).despawn();
                info_event_writer.send(InfoEvent::PlayerPickedUpSnail);
            }
        }
    }
}
