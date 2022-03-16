use bevy::ecs::schedule::{IntoRunCriteria, RunCriteriaDescriptorOrLabel};
use bevy::ecs::system::QuerySingleError;
use bevy::prelude::*;
use bevy::reflect::Map;
use bevy_ecs_tilemap::{MapQuery, TilePos, TilemapPlugin};
use log::info;

use crate::helpers::error_handling::ResultOkLog;

use super::{
    components::*,
    enemy::{Enemy, Shark},
    events::{GameEvent, InputEvent},
    tilemap::{HasTileType, TilePosExt},
    timed_removal::{TimedRemoval, TimedRemovalPlugin},
    turn::{GamePhase, GlobalTurnCounter, TurnCounter},
};
use crate::game::events::{InfoEvent, PowerEvent};
use crate::game::movement::{AttackCriteria, MoveDecisions};
use crate::game::projectile::{spawn_projectile, ProjectileFate};
use crate::game::ui::GameUiPlugin;
use crate::helpers::cleanup::recursive_cleanup;
use crate::map_gen::cell_map::CellMap;
use bevy::input::gamepad::{gamepad_connection_system, gamepad_event_system};
use bevy::sprite::MaterialMesh2dBundle;
use bevy_kira_audio::Audio;
use std::io::Chain;
use std::marker::PhantomData;
use std::time::Duration;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        let state = crate::State::Game;
        app.add_system_set(
            SystemSet::on_enter(state)
                .with_system(setup)
                .with_system(add_test_mesh2d),
        )
        .add_system_set(
            SystemSet::on_update(state)
                .with_system(animate_sprite_system)
                .with_system(simple_animate_sprite_system)
                .with_system(input_handle_system.label("input"))
                .with_system(mouse_click_system.label("input"))
                .with_system(gamepad_input_handle_system.label("input"))
                .with_system(debug_print_input_system)
                .with_system(player_power_system)
                .with_system(player_movement_system.label("player_movement"))
                .with_system(camera_follow_system.after("player_movement"))
                .with_system(player_movement_watcher.after("player_movement"))
                .with_system(
                    enemy_system
                        .label("enemy_movement")
                        .after("player_movement"),
                )
                .with_system(animate_move_system.after("enemy_movement"))
                .with_system(global_turn_counter_system.after("enemy_movement"))
                .with_system(mouse_click_debug_system.after("input"))
                .with_system(input_event_debug_system.after("input"))
                .with_system(health_watcher_system.after("enemy_movement"))
                .with_system(player_damaged_effect_system.after("enemy_movement"))
                .with_system(sfx_system)
                .with_system(waggle_system)
                .with_system(player_death_system)
                .with_system(super::projectile::projectile_watcher_system)
                .with_system(super::projectile::projectile_system),
        )
        .add_system_set(
            SystemSet::on_exit(state)
                .with_system(recursive_cleanup::<GameOnly>)
                .with_system(super::tilemap::cleanup),
        )
        .add_plugin(TimedRemovalPlugin)
        .add_plugin(GameUiPlugin)
        .add_system(
            // TODO: when pre-loading is implemented we can do away with this (i think)
            crate::helpers::texture::set_texture_filters_to_nearest,
        )
        .add_event::<super::events::GameEvent>()
        .add_event::<super::events::InputEvent>()
        .add_event::<super::events::InfoEvent>()
        .add_event::<super::events::PowerEvent>()
        .add_event::<MouseClickEvent>()
        .insert_resource(GlobalTurnCounter::default());
    }
}

fn global_turn_counter_system(
    mut global_turn_counter: ResMut<GlobalTurnCounter>,
    mut game_event_reader: EventReader<GameEvent>,
) {
    for event in game_event_reader.iter() {
        match event {
            GameEvent::PhaseComplete(phase) => {
                global_turn_counter.step(&phase);
                info!("New Turn: {:?}", global_turn_counter);
            }
            GameEvent::PlayerDied => (),
        }
    }
}

fn player_death_system(
    mut game_event_reader: EventReader<GameEvent>,
    mut game_state: ResMut<State<crate::State>>,
) {
    for event in game_event_reader.iter() {
        match event {
            GameEvent::PhaseComplete(_) => (),
            GameEvent::PlayerDied => {
                info!("Player died");
                game_state.set(crate::State::MainMenu);
            }
        }
    }
}

fn waggle_system(
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut Waggle)>,
    mut commands: Commands,
) {
    for (entity, mut transform, mut waggle) in query.iter_mut() {
        waggle.update(&mut transform.rotation, &time.delta());
        if waggle.finished() {
            println!("Waggle Finished");
            transform.rotation = Quat::from_rotation_z(0.0);
            commands.entity(entity).remove::<Waggle>();
        }
    }
}

fn simple_animate_sprite_system(
    time: Res<Time>,
    mut query: Query<(
        &mut Timer,
        &mut TextureAtlasSprite,
        &mut SimpleSpriteAnimation,
    )>,
) {
    for (mut timer, mut sprite, mut animation) in query.iter_mut() {
        timer.tick(time.delta());
        if timer.finished() {
            animation.incr();
        }
        sprite.index = animation.frame_index;
    }
}

fn animate_sprite_system(
    time: Res<Time>,
    mut query: Query<(
        &mut Timer,
        &mut TextureAtlasSprite,
        &Facing,
        &mut DirectionalSpriteAnimation,
        Option<&DirectionalSpriteAnimationSpecial>,
    )>,
) {
    for (
        mut timer,
        mut sprite,
        facing,
        mut direction_animation,
        maybe_direction_animation_special,
    ) in query.iter_mut()
    {
        timer.tick(time.delta());
        if timer.finished() {
            direction_animation.incr();
        }
        if let Some(special_index) = maybe_direction_animation_special {
            sprite.index = direction_animation.special_index_safe(special_index.0, &facing.0)
        } else if direction_animation.dirty {
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
        QueryState<&mut Transform, With<GameCamera>>,
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
    camera_query: Query<&Transform, With<GameCamera>>,
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
        (Some(dir), false) => {
            input_events.send(InputEvent::MoveDirection(dir));
            return;
        }
        (Some(dir), true) => {
            input_events.send(InputEvent::TurnDirection(dir));
            return;
        }
        (None, _) => (),
    }

    if input.just_pressed(KeyCode::Space) {
        input_events.send(InputEvent::Wait);
        return;
    }

    if input.just_pressed(KeyCode::Q) {
        input_events.send(InputEvent::Power);
        return;
    }
}

fn gamepad_input_handle_system(
    input: Res<Input<GamepadButton>>,
    gamepads: Res<Gamepads>,
    mut input_events: EventWriter<InputEvent>,
) {
    // TODO: Flesh this out and add proper gamepad support eventually
    for gamepad in gamepads.iter().cloned() {
        let new_direction = {
            if input.just_pressed(GamepadButton(gamepad, GamepadButtonType::DPadLeft)) {
                Some(MapDirection::Left)
            } else if input.just_pressed(GamepadButton(gamepad, GamepadButtonType::DPadRight)) {
                Some(MapDirection::Right)
            } else if input.just_pressed(GamepadButton(gamepad, GamepadButtonType::DPadUp)) {
                Some(MapDirection::Up)
            } else if input.just_pressed(GamepadButton(gamepad, GamepadButtonType::DPadDown)) {
                Some(MapDirection::Down)
            } else {
                None
            }
        };
        if let Some(dir) = new_direction {
            input_events.send(InputEvent::MoveDirection(dir));
            return;
        }
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

fn player_damaged_effect_system(
    mut info_event_reader: EventReader<InfoEvent>,
    player_query: Query<Entity, With<Player>>,
    mut commands: Commands,
) {
    for event in info_event_reader.iter() {
        match event {
            InfoEvent::PlayerHurt => {
                let player_entity = player_query.single();
                let timed_removal: TimedRemoval<DirectionalSpriteAnimationSpecial> =
                    TimedRemoval::new(Duration::from_millis(220));
                let waggle = Waggle::new(8, 0.2, 0.2, 10.0);
                commands
                    .entity(player_entity)
                    .insert(DirectionalSpriteAnimationSpecial(0))
                    .insert(timed_removal)
                    .insert(waggle);
            }
            _ => (),
        }
    }
}

fn health_watcher_system(
    enemy_health: Query<(Entity, &Health), (With<Enemy>, Changed<Health>)>,
    player_health: Query<(Entity, &Health), (With<Player>, Changed<Health>)>,
    mut info_event_writer: EventWriter<InfoEvent>,
    mut commands: Commands,
    mut known_player_hp: Local<Option<usize>>,
    mut game_event_writer: EventWriter<GameEvent>,
) {
    for (entity, health) in enemy_health.iter() {
        if health.hp == 0 {
            info_event_writer.send(InfoEvent::EnemyKilled);
            println!("Enemy died {:?}", entity);
            commands.entity(entity).despawn()
        }
    }

    for (entity, health) in player_health.iter() {
        // There's a small chance this change triggers even if health aint changed - may need to
        // handle this if it becomes a problem
        match *known_player_hp {
            Some(known_hp) if known_hp != health.hp => {
                info_event_writer.send(InfoEvent::PlayerHurt);
            }
            _ => (),
        }
        *known_player_hp = Some(health.hp);
        if health.hp == 0 {
            game_event_writer.send(GameEvent::PlayerDied);
            println!("Player! died {:?}", entity);
        }
    }
}

fn sfx_system(
    mut info_event_reader: EventReader<InfoEvent>,
    audio: Res<Audio>,
    assets: Res<AssetServer>,
) {
    for event in info_event_reader.iter() {
        match event {
            InfoEvent::PlayerHurt => {
                debug!("Playing Audio for Player Hurt");
                let sound = assets.load("audio/342229__christopherderp__hurt-1-male.wav");
                audio.play(sound);
            }
            InfoEvent::EnemyKilled => {
                debug!("Playing Audio for Enemy Killed");
                let sound = assets.load("audio/carrotnom.wav");
                audio.play(sound);
            }
            InfoEvent::PlayerMoved => {
                debug!("Playing Audio for Player Moved");
                let sound = assets.load("audio/fish_slap.ogg");
                audio.play(sound);
            }
        }
    }
}

fn enemy_system(
    mut game_event_writer: EventWriter<GameEvent>,
    global_turn_counter: Res<GlobalTurnCounter>,
    mut local_turn_counter: Local<TurnCounter>,
    enemy_query: Query<Entity, With<Enemy>>,
    mut health_query: Query<&mut Health>,
    mut move_query: QuerySet<(
        QueryState<&TilePos, With<Player>>,
        QueryState<&TilePos, With<Enemy>>,
        QueryState<(Entity, &TilePos, Option<&Player>, Option<&Enemy>)>,
        QueryState<(&mut TilePos, &mut MovementAnimate, &Transform, &mut Facing)>,
    )>,
    mut map_query: MapQuery,
    tile_type_query: Query<&HasTileType>,
) {
    let player_position = move_query.q0().get_single().unwrap().clone();
    if global_turn_counter.can_take_turn(&local_turn_counter, GamePhase::EnemyMovement) {
        let attack_criteria = AttackCriteria::for_enemy();
        let mut move_decisions = MoveDecisions::new();
        let mut moved_to = Vec::new();
        for (entity) in enemy_query.iter() {
            let current_pos = move_query.q1().get(entity).unwrap().clone();
            let direction = MapDirection::weighted_rand_choice(&current_pos, &player_position);
            let decision = super::movement::decide_move(
                &current_pos,
                &direction,
                &attack_criteria,
                move_query.q2(),
                &mut map_query,
                &tile_type_query,
                &moved_to,
            );
            if let Some(tilepos) = decision.to_move_position() {
                moved_to.push(tilepos);
            }
            move_decisions.insert(entity, decision);
        }
        println!("Move Decisions: {:?}", move_decisions);

        super::movement::apply_move(move_decisions, move_query.q3(), health_query);
        local_turn_counter.incr();
        game_event_writer.send(GameEvent::PhaseComplete(GamePhase::EnemyMovement));
    }
}

fn player_movement_watcher(
    player_position_query: Query<&TilePos, (With<Player>, Changed<TilePos>)>,
    mut known_player_position: Local<Option<TilePos>>,
    mut info_event_writer: EventWriter<InfoEvent>,
) {
    if let Ok(player_tilepos) = player_position_query.get_single() {
        match *known_player_position {
            Some(pos) if pos != *player_tilepos => {
                info_event_writer.send(InfoEvent::PlayerMoved);
            }
            _ => (),
        }
        *known_player_position = Some(player_tilepos.clone());
    }
}

fn player_movement_system(
    mut game_event_writer: EventWriter<GameEvent>,
    mut power_event_writer: EventWriter<PowerEvent>,
    mut input_events: EventReader<InputEvent>,
    mut move_query: QuerySet<(
        QueryState<(Entity, &TilePos), With<Player>>,
        QueryState<(Entity, &TilePos, Option<&Player>, Option<&Enemy>)>,
        QueryState<(&mut TilePos, &mut MovementAnimate, &Transform, &mut Facing)>,
        QueryState<&mut Facing, With<Player>>,
    )>,
    mut power_query: Query<&mut PowerCharges, With<Player>>,
    mut health_query: Query<(&mut Health)>,
    tile_type_query: Query<(&HasTileType)>,
    mut map_query: MapQuery,
    global_turn_counter: Res<GlobalTurnCounter>,
    mut local_turn_counter: Local<TurnCounter>,
) {
    for event in input_events.iter() {
        match event {
            InputEvent::MoveDirection(direction) => {
                let can_take_turn = global_turn_counter
                    .can_take_turn(&local_turn_counter, GamePhase::PlayerMovement);
                if can_take_turn {
                    let (player_entity, current_pos) = move_query.q0().get_single().unwrap();
                    let current_pos = current_pos.clone();

                    let move_decision = super::movement::decide_move(
                        &current_pos,
                        &direction,
                        &AttackCriteria::for_player(),
                        move_query.q1(),
                        &mut map_query,
                        &tile_type_query,
                        &vec![],
                    );

                    super::movement::apply_move_single(
                        player_entity,
                        &move_decision,
                        &mut move_query.q2(),
                        &mut health_query,
                    );

                    local_turn_counter.incr();
                    game_event_writer.send(GameEvent::PhaseComplete(GamePhase::PlayerMovement));
                }
            }
            InputEvent::TurnDirection(dir) => {
                let can_take_turn = global_turn_counter
                    .can_take_turn(&local_turn_counter, GamePhase::PlayerMovement);
                if can_take_turn {
                    info!("Player Turning: {:?}", dir);
                    move_query.q3().single_mut().0 = dir.clone();
                    local_turn_counter.incr();
                    game_event_writer.send(GameEvent::PhaseComplete(GamePhase::PlayerMovement));
                }
            }
            InputEvent::Wait => {
                let can_take_turn = global_turn_counter
                    .can_take_turn(&local_turn_counter, GamePhase::PlayerMovement);
                if can_take_turn {
                    info!("Player Waiting");
                    local_turn_counter.incr();
                    game_event_writer.send(GameEvent::PhaseComplete(GamePhase::PlayerMovement));
                }
            }
            InputEvent::Power => {
                let can_take_turn = global_turn_counter
                    .can_take_turn(&local_turn_counter, GamePhase::PlayerMovement);
                if can_take_turn {
                    let mut power_charges = power_query.single_mut();
                    if power_charges.charges > 0 {
                        power_event_writer.send(PowerEvent::PowerFired);
                        local_turn_counter.incr();
                        game_event_writer.send(GameEvent::PhaseComplete(GamePhase::PlayerMovement));
                        power_charges.use_charge();
                    }
                }
            }
        }
    }
}

fn player_power_system(
    mut query: QuerySet<(
        QueryState<(&Transform, &TilePos, &Facing), With<Player>>,
        QueryState<(Entity, &TilePos), With<Enemy>>,
    )>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut power_event_reader: EventReader<PowerEvent>,
    mut map_query: MapQuery,
    tile_type_query: Query<&HasTileType>,
) {
    for event in power_event_reader.iter() {
        match event {
            PowerEvent::PowerFired => {
                let (start_pos, tilepos, direction): (Vec3, TilePos, MapDirection) = {
                    let (transform, tilepos, facing) = query.q0().single();
                    (
                        (*transform).translation.clone(),
                        tilepos.clone(),
                        facing.0.clone(),
                    )
                };
                let fate = super::projectile::scan_to_endpoint(
                    &tilepos,
                    &direction,
                    &query.q1(),
                    &mut map_query,
                    &tile_type_query,
                );
                let end_point = fate.tile_pos().clone();
                let end_target_entity = fate.entity();
                super::projectile::spawn_projectile(
                    &mut commands,
                    &asset_server,
                    &mut texture_atlases,
                    direction,
                    start_pos,
                    end_point,
                    end_target_entity,
                );
            }
        }
    }
}

fn debug_print_input_system(
    mut query: QuerySet<(
        QueryState<(&Transform, &GlobalTransform)>,
        QueryState<Entity, With<Player>>,
        QueryState<(&TilePos, &Transform), With<Player>>,
        QueryState<&TilePos, With<Enemy>>,
    )>,
    input: Res<Input<KeyCode>>,
    mut cell_map: ResMut<CellMap<i32>>,
    mut commands: Commands,
    mut asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut info_event_writer: EventWriter<InfoEvent>,
) {
    if input.just_pressed(KeyCode::P) {
        for (trans, global_trans) in query.q0().iter() {
            println!("{:?} (Global: {:?}", trans, global_trans)
        }
    }

    if input.just_pressed(KeyCode::G) {
        let player_entity = query.q1().single();
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
            .q3()
            .iter()
            .map(|tilepos: &TilePos| tilepos.as_i32s())
            .collect::<Vec<(i32, i32)>>();
        let start_point = {
            let (TilePos(x, y), trans) = query.q2().single();
            (*x as i32, *y as i32)
        };
        let recalculated_map = cell_map.recalculate(start_point);
        add_sharks(
            &mut commands,
            &mut asset_server,
            &mut texture_atlases,
            4,
            &recalculated_map,
            Some(&exclude_positions),
        );
        *cell_map = recalculated_map;
    }
}
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    map_query: MapQuery,
) {
    let border_size = 20usize;
    let cell_map: CellMap<i32> = {
        let normalised = crate::map_gen::get_cell_map(50, 50);
        normalised.offset((border_size as i32, border_size as i32))
    };
    println!("Final CellMap: {:?}", cell_map);
    super::tilemap::init_tilemap(
        &mut commands,
        &asset_server,
        map_query,
        &cell_map,
        border_size,
    );
    commands
        .spawn_bundle(OrthographicCameraBundle::new_2d())
        .insert(GameOnly)
        .insert(GameCamera);
    let texture_handle = asset_server.load("sprites/haddock_spritesheet.png");
    let atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(64.0, 64.0), 5, 4);
    let atlas_handle = texture_atlases.add(atlas);
    let start_point = {
        let start_point = cell_map.start_point().unwrap_or((1, 1));
        TilePos(start_point.0 as u32, start_point.1 as u32)
    };
    commands
        .spawn_bundle(TileResidentBundle::new(3, start_point, atlas_handle, 1))
        .insert(CameraFollow {
            x_threshold: 300.0,
            y_threshold: 200.0,
        })
        .insert(PowerCharges::new(3))
        .insert(Player);
    add_sharks(
        &mut commands,
        &asset_server,
        &mut texture_atlases,
        7,
        &cell_map,
        None,
    );
    commands.insert_resource(cell_map);
}

fn add_test_mesh2d(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut commands: Commands,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    asset_server: Res<AssetServer>,
) {
    let start_pos = TilePos(22, 22).to_world_pos(20.0);
    let height = 6000.0;
    let offset_start_pos = Vec3::new(
        start_pos.x + 14.0,
        start_pos.y + 32.0 + (height / 2.0),
        start_pos.z,
    );
    let material = materials.add(ColorMaterial::from(Color::rgb(
        34.0 / 255.0,
        32.0 / 255.0,
        52.0 / 255.0,
    )));
    let mesh = meshes
        .add(Mesh::from(shape::Quad::new(Vec2::new(3.0, height))))
        .into();
    commands.spawn_bundle(MaterialMesh2dBundle {
        mesh,
        transform: Transform::from_translation(offset_start_pos),
        material,
        ..Default::default()
    });
    let texture_handle = asset_server.load("sprites/hook_spritesheet.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(64.0, 64.0), 4, 1);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: texture_atlas_handle,
            transform: Transform::from_translation(start_pos),
            ..Default::default()
        })
        .insert(GameOnly)
        .insert(Timer::from_seconds(0.250, true))
        .insert(SimpleSpriteAnimation::new(4));
}

fn add_sharks(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    texture_atlases: &mut ResMut<Assets<TextureAtlas>>,
    num_sharks: usize,
    cell_map: &CellMap<i32>,
    exclude_positions: Option<&Vec<(i32, i32)>>,
) {
    let texture_handle = asset_server.load("sprites/shark_spritesheet.png");
    let atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(64.0, 64.0), 4, 4);
    let atlas_handle = texture_atlases.add(atlas);
    let spawn_positions = cell_map.distribute_points_by_cost(num_sharks, exclude_positions);
    for (x, y) in spawn_positions.into_iter() {
        let tile_pos = TilePos(x as u32, y as u32);
        commands
            .spawn_bundle(TileResidentBundle::new(
                1,
                tile_pos,
                atlas_handle.clone(),
                0,
            ))
            .insert(Enemy {})
            .insert(Shark {});
    }
}
