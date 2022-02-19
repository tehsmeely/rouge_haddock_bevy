use bevy::ecs::schedule::{IntoRunCriteria, RunCriteriaDescriptorOrLabel};
use bevy::ecs::system::QuerySingleError;
use bevy::prelude::*;
use bevy::reflect::Map;
use bevy_ecs_tilemap::{MapQuery, TilePos, TilemapPlugin};
use log::info;

use crate::game::components::TileType;
use crate::helpers::error_handling::ResultOkLog;

use super::{
    components::*,
    enemy::{Enemy, Shark},
    events::{GameEvent, InputEvent},
    tilemap::{HasTileType, TilePosExt},
    turn::{GamePhase, GlobalTurnCounter, TurnCounter},
};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup)
            .add_startup_system(add_sharks)
            .add_system(animate_sprite_system)
            .add_system(input_handle_system.label("input"))
            .add_system(camera_follow_system.after("movement"))
            .add_system(debug_print_input_system)
            .add_system(player_movement_system.label("movement"))
            .add_system(global_turn_counter_system.after("movement"))
            .add_system(enemy_system.after("movement"))
            .add_system(mouse_click_system.label("input"))
            .add_system(animate_move_system.after("movement"))
            .add_system_set(
                SystemSet::new()
                    .with_system(mouse_click_debug_system.after("input"))
                    .with_system(input_event_debug_system.after("input")),
            )
            .add_event::<super::events::GameEvent>()
            .add_event::<super::events::InputEvent>()
            .add_event::<MouseClickEvent>()
            .insert_resource(GlobalTurnCounter::default());
        super::tilemap::build(app);
    }
}

fn global_turn_counter_system(
    mut global_turn_counter: ResMut<GlobalTurnCounter>,
    mut game_event_reader: EventReader<GameEvent>,
) {
    for event in game_event_reader.iter() {
        match event {
            super::events::GameEvent::PhaseComplete(phase) => {
                global_turn_counter.step(&phase);
                info!("New Turn: {:?}", global_turn_counter);
            }
        }
    }
}

fn animate_sprite_system(
    time: Res<Time>,
    mut query: Query<(
        &mut Timer,
        &mut TextureAtlasSprite,
        &Facing,
        &mut DirectionalAnimation,
    )>,
) {
    for (mut timer, mut sprite, facing, mut direction_animation) in query.iter_mut() {
        timer.tick(time.delta());
        if timer.finished() {
            direction_animation.incr();
        }
        if direction_animation.dirty {
            sprite.index = direction_animation.index(&facing.0);
        }
    }
}

fn animate_move_system(mut query: Query<(&mut Transform, &mut MovementAnimate)>) {
    for (mut transform, mut movement_animate) in query.iter_mut() {
        if movement_animate.active {
            transform.translation = movement_animate.lerp(&transform.translation);

            if movement_animate.finished(&transform.translation) {
                movement_animate.active = false;
            }
        }
    }
}

fn camera_follow_system(
    mut query: QuerySet<(
        QueryState<(&Transform, &CameraFollow)>,
        QueryState<&mut Transform, With<Camera>>,
    )>,
) {
    let mut pos = query.q0().get_single().ok_log().map(|(transform, follow)| {
        (
            transform.translation.x,
            transform.translation.y,
            follow.x_threshold,
            follow.y_threshold,
        )
    });

    if let Some((x, y, x_threshold, y_threshold)) = pos {
        if let Some(mut camera_transform) = query.q1().get_single_mut().ok_log() {
            if (x - camera_transform.translation.x).abs() > x_threshold {
                camera_transform.translation.x = x
            }
            if (y - camera_transform.translation.y).abs() > y_threshold {
                camera_transform.translation.y = y
            }
        }
    }
}

fn mouse_click_system(
    input: Res<Input<MouseButton>>,
    windows: Res<Windows>,
    camera_query: Query<&Transform, With<Camera>>,
    mut mouse_event_writer: EventWriter<MouseClickEvent>,
) {
    let mouse_button = {
        if input.just_pressed(MouseButton::Left) {
            Some(MouseButton::Left)
        } else if input.just_pressed(MouseButton::Right) {
            Some(MouseButton::Right)
        } else if input.just_pressed(MouseButton::Middle) {
            Some(MouseButton::Middle)
        } else {
            None
        }
    };

    if let Some(mouse_button) = mouse_button {
        let window = windows.get_primary().unwrap();
        if let Some(pos) = window.cursor_position() {
            let size = Vec2::new(window.width() as f32, window.height() as f32);
            // the default orthographic projection is in pixels from the center;
            // just undo the translation
            let pos = pos - size / 2.0;

            if let Some(camera_transform) = camera_query.get_single().ok_log() {
                // apply the camera transform
                let world_position =
                    camera_transform.compute_matrix() * pos.extend(0.0).extend(1.0);

                debug!("Click at world pos: {:?}", world_position);
                mouse_event_writer.send(MouseClickEvent {
                    button: mouse_button,
                    world_position: world_position.truncate(),
                })
            }
        }
    }
}

fn input_handle_system(input: Res<Input<KeyCode>>, mut input_events: EventWriter<InputEvent>) {
    let new_direction = {
        if input.just_pressed(KeyCode::A) {
            Some(MapDirection::Left)
        } else if input.just_pressed(KeyCode::D) {
            Some(MapDirection::Right)
        } else if input.just_pressed(KeyCode::W) {
            Some(MapDirection::Up)
        } else if input.just_pressed(KeyCode::S) {
            Some(MapDirection::Down)
        } else {
            None
        }
    };
    let shift_held = input.pressed(KeyCode::LShift);
    match (new_direction, shift_held) {
        (Some(dir), false) => input_events.send(InputEvent::MoveDirection(dir)),
        (Some(dir), true) => input_events.send(InputEvent::TurnDirection(dir)),
        (None, _) => (),
    }
}

fn input_event_debug_system(mut input_events: EventReader<InputEvent>) {
    for event in input_events.iter() {
        let event: &InputEvent = event;
        println!("Input Event: {:?}", event);
    }
}

fn mouse_click_debug_system(
    mut mouse_event_reader: EventReader<MouseClickEvent>,
    tile_type_query: Query<&HasTileType>,
    mut map_query: MapQuery,
) {
    for MouseClickEvent {
        button,
        world_position,
    } in mouse_event_reader.iter()
    {
        if button == &MouseButton::Left {
            let tile_pos = TilePos::from_world_pos(world_position.x, world_position.y);
            let tile_entity = map_query.get_tile_entity(tile_pos, 0, 0).unwrap();
            let tile_type = tile_type_query.get(tile_entity).unwrap();
            println!("Clicked {:?} ({:?})", tile_pos, tile_type);
        }
    }
}

fn enemy_system(
    mut game_event_writer: EventWriter<GameEvent>,
    global_turn_counter: Res<GlobalTurnCounter>,
    mut local_turn_counter: Local<TurnCounter>,
    mut query: Query<
        (
            &mut TilePos,
            &mut Transform,
            &mut Facing,
            &mut MovementAnimate,
        ),
        With<Enemy>,
    >,
    mut map_query: MapQuery,
    tile_type_query: Query<&HasTileType>,
) {
    if global_turn_counter.can_take_turn(&local_turn_counter, GamePhase::EnemyMovement) {
        for (mut tile_pos, mut transform, mut facing, mut move_animation) in query.iter_mut() {
            let direction = MapDirection::rand_choice();

            move_map_object(
                &mut tile_pos,
                &direction,
                &mut map_query,
                &mut facing,
                &mut transform,
                &mut move_animation,
                &tile_type_query,
            );
        }
        local_turn_counter.incr();
        game_event_writer.send(GameEvent::PhaseComplete(GamePhase::EnemyMovement));
    }
}

fn player_movement_system(
    mut game_event_writer: EventWriter<GameEvent>,
    mut input_events: EventReader<InputEvent>,
    mut query: Query<(
        &mut Facing,
        &Controlled,
        &mut Transform,
        &mut TilePos,
        &mut MovementAnimate,
    )>,
    tile_type_query: Query<(&HasTileType)>,
    mut map_query: MapQuery,
    global_turn_counter: Res<GlobalTurnCounter>,
    mut local_turn_counter: Local<TurnCounter>,
) {
    for event in input_events.iter() {
        match event {
            InputEvent::MoveDirection(direction) => {
                for (mut facing, controlled, mut transform, mut tile_pos, mut movement_animate) in
                    query.iter_mut()
                {
                    let can_take_turn = global_turn_counter
                        .can_take_turn(&local_turn_counter, GamePhase::PlayerMovement);
                    if can_take_turn && controlled.0 {
                        move_map_object(
                            &mut tile_pos,
                            &direction,
                            &mut map_query,
                            &mut facing,
                            &mut transform,
                            &mut movement_animate,
                            &tile_type_query,
                        );
                        local_turn_counter.incr();
                        game_event_writer.send(GameEvent::PhaseComplete(GamePhase::PlayerMovement));
                    }
                }
            }
            InputEvent::TurnDirection(_dir) => (),
            InputEvent::Wait => (),
            InputEvent::Power => (),
        }
    }
}

fn move_map_object(
    mut current_tile_pos: &mut TilePos,
    move_direction: &MapDirection,
    map_query: &mut MapQuery,
    facing: &mut Facing,
    transform: &mut Transform,
    move_animation: &mut MovementAnimate,
    tile_type_query: &Query<(&HasTileType)>,
) {
    let new_tilepos = current_tile_pos.add(move_direction.to_pos_move());
    println!("New Tile Pos: {:?}", new_tilepos);
    let new_tile_entity = map_query.get_tile_entity(new_tilepos, 0, 0).unwrap();
    let can_move = match tile_type_query.get(new_tile_entity) {
        Ok(HasTileType(tt)) => tt.can_enter(),
        Err(_) => false,
    };
    facing.0 = move_direction.clone();
    if can_move {
        *current_tile_pos = new_tilepos;
        //let z = transform.translation.z;
        //transform.translation = new_tilepos.to_world_pos();
        //transform.translation.z = z;
        move_animation.set(new_tilepos.to_world_pos(transform.translation.z));
        facing.0 = move_direction.clone();
        println!("Now at {:?}", current_tile_pos);
    }
}

fn debug_print_input_system(
    mut query: QuerySet<(
        QueryState<(&Transform, &GlobalTransform)>,
        QueryState<(&Controlled)>,
    )>,
    //query: Query<(&Transform, &GlobalTransform)>,
    input: Res<Input<KeyCode>>,
) {
    if input.just_pressed(KeyCode::P) {
        for (trans, global_trans) in query.q0().iter() {
            println!("{:?} (Global: {:?}", trans, global_trans)
        }
    } else if input.just_pressed(KeyCode::C) {
        for (controlled) in query.q1().iter() {
            println!("{:?}", controlled)
        }
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    map_query: MapQuery,
) {
    let cell_map = super::tilemap::init_tilemap(&mut commands, &asset_server, map_query);
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    let texture_handle = asset_server.load("sprites/haddock_spritesheet.png");
    let atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(64.0, 64.0), 4, 4);
    let atlas_handle = texture_atlases.add(atlas);
    let start_point = {
        let start_point = cell_map.start_point().unwrap_or((1, 1));
        TilePos(start_point.0 as u32, start_point.1 as u32)
    };
    let start_pos = start_point.to_world_pos(10.0);
    commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: atlas_handle.clone(),
            transform: Transform::from_translation(start_pos), //from_xyz(transform.x, transform.y, 10.0),
            ..Default::default()
        })
        .insert(Timer::from_seconds(0.1, true))
        .insert(Facing::default())
        .insert(DirectionalAnimation::default())
        .insert(CameraFollow {
            x_threshold: 300.0,
            y_threshold: 200.0,
        })
        .insert(start_point)
        .insert(Controlled(true))
        .insert(MovementAnimate::default());
}

fn add_sharks(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let texture_handle = asset_server.load("sprites/shark_spritesheet.png");
    let atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(64.0, 64.0), 4, 4);
    let atlas_handle = texture_atlases.add(atlas);
    for (x, y) in [(8, 9), (12, 12), (3, 10)].into_iter() {
        let tile_pos = TilePos(x, y);
        let start_pos = tile_pos.to_world_pos(10.0);
        commands
            .spawn_bundle(SpriteSheetBundle {
                texture_atlas: atlas_handle.clone(),
                transform: Transform::from_translation(start_pos),
                ..Default::default()
            })
            .insert(Timer::from_seconds(0.1, true))
            .insert(Facing::default())
            .insert(DirectionalAnimation::default())
            .insert(tile_pos)
            .insert(Enemy {})
            .insert(Shark {})
            .insert(MovementAnimate::default());
    }
}
