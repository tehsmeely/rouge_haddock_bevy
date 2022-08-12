use crate::asset_handling::{ImageAssetStore, TextureAtlasStore};
use crate::game::components::*;
use crate::game::enemy::Enemy;
use crate::game::events::{InfoEvent, InputEvent};

use crate::game::tilemap::{HasTileType, TilePosExt, TileStorageQuery};
use crate::game::turn::{GamePhase, GlobalTurnCounter, TurnCounter};
use crate::map_gen::cell_map::CellMap;
use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TilePos;

pub struct GameDebugPlugin;

impl Plugin for GameDebugPlugin {
    fn build(&self, app: &mut App) {
        //This plugin is empty unless "debug_assetions" is enabled, i.e. it is in dev
        // so no systems below will be run in release builds
        if cfg!(debug_assertions) {
            app.add_system_set(
                SystemSet::on_update(crate::CoreState::GameLevel)
                    .with_system(debug_print_input_system)
                    .with_system(input_event_debug_system)
                    .with_system(mouse_click_debug_system),
            );
        }
    }
}

fn debug_print_input_system(
    mut query: ParamSet<(
        Query<(&Transform, &GlobalTransform)>,
        Query<Entity, With<Player>>,
        Query<(&TilePos, &Transform), With<Player>>,
        Query<&TilePos, With<Enemy>>,
    )>,
    _player_health_q: Query<&mut Health, With<Player>>,
    _player_charges_q: Query<&mut PowerCharges, With<Player>>,
    input: Res<Input<KeyCode>>,
    mut cell_map: ResMut<CellMap<i32>>,
    mut commands: Commands,
    _asset_server: Res<AssetServer>,
    _texture_atlases: ResMut<Assets<TextureAtlas>>,
    atlases: Res<TextureAtlasStore>,
    mut info_event_writer: EventWriter<InfoEvent>,
    _image_assets: Res<ImageAssetStore>,
) {
    if input.just_pressed(KeyCode::P) {
        for (trans, global_trans) in query.p0().iter() {
            println!("{:?} (Global: {:?}", trans, global_trans)
        }
    }

    if input.just_pressed(KeyCode::G) {
        let player_entity = query.p1().single();
        commands
            .entity(player_entity)
            .insert(Waggle::new(5, -0.4, 0.4, 10.0));
    }
    if input.just_pressed(KeyCode::H) {
        info_event_writer.send(InfoEvent::PlayerHurt);
    }

    if input.just_pressed(KeyCode::O) {
        info!("Spawning more sharks");
        let exclude_positions = query
            .p3()
            .iter()
            .map(|tilepos: &TilePos| tilepos.as_i32s())
            .collect::<Vec<(i32, i32)>>();
        let start_point = {
            let q = query.p2();
            let (TilePos { x, y }, _trans) = q.single();
            (*x as i32, *y as i32)
        };
        let recalculated_map = cell_map.recalculate(start_point);
        let _: Vec<(i32, i32)> = super::enemy::add_sharks(
            &mut commands,
            &atlases,
            4,
            &recalculated_map,
            Some(&exclude_positions),
        );
        *cell_map = recalculated_map;
    }
}

fn input_event_debug_system(
    mut input_events: EventReader<InputEvent>,
    global_turn_counter: Res<GlobalTurnCounter>,
    mut local_turn_counter: Local<TurnCounter>,
) {
    for event in input_events.iter() {
        let event: &InputEvent = event;
        info!("Input Event: {:?}", event);
        if global_turn_counter.can_take_turn(&mut local_turn_counter, GamePhase::PlayerMovement) {
            info!("Can take turn");
        } else {
            info!(
                "Can't take turn. {:?}, {:?})",
                global_turn_counter, local_turn_counter
            );
        }
    }
}

fn mouse_click_debug_system(
    mut mouse_event_reader: EventReader<MouseClickEvent>,
    tile_type_query: Query<&HasTileType>,
    tile_storage_query: TileStorageQuery,
) {
    for MouseClickEvent {
        button,
        world_position,
    } in mouse_event_reader.iter()
    {
        if button == &MouseButton::Left {
            let tile_pos = TilePos::from_world_pos(world_position.x, world_position.y);
            let tile_entity = tile_storage_query.single().get(&tile_pos).unwrap();
            if let Ok(tile_type) = tile_type_query.get(tile_entity) {
                println!("Clicked {:?} ({:?})", tile_pos, tile_type);
            }
        }
    }
}
