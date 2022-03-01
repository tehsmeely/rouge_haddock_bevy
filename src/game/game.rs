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
use crate::game::events::InfoEvent;
use crate::game::movement::{AttackCriteria, MoveDecisions};
use crate::game::projectile::spawn_projectile;
use crate::map_gen::cell_map::CellMap;
use bevy::sprite::MaterialMesh2dBundle;
use bevy_kira_audio::Audio;
use std::io::Chain;
use std::marker::PhantomData;
use std::time::Duration;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup)
            .add_startup_system(add_test_mesh2d)
            .add_system(animate_sprite_system)
            .add_system(input_handle_system.label("input"))
            .add_system(mouse_click_system.label("input"))
            .add_system(debug_print_input_system)
            .add_system(player_movement_system.label("player_movement"))
            .add_system(camera_follow_system.after("player_movement"))
            .add_system(player_movement_watcher.after("player_movement"))
            .add_system(
                enemy_system
                    .label("enemy_movement")
                    .after("player_movement"),
            )
            .add_system(animate_move_system.after("enemy_movement"))
            .add_system(global_turn_counter_system.after("enemy_movement"))
            .add_system_set(
                SystemSet::new()
                    .with_system(mouse_click_debug_system.after("input"))
                    .with_system(input_event_debug_system.after("input")),
            )
            .add_system(health_watcher_system.after("enemy_movement"))
            .add_system(player_damaged_effect_system.after("enemy_movement"))
            .add_system(sfx_system)
            .add_system(waggle_system)
            .add_system(super::projectile::projectile_system)
            .add_plugin(TimedRemovalPlugin)
            .add_event::<super::events::GameEvent>()
            .add_event::<super::events::InputEvent>()
            .add_event::<super::events::InfoEvent>()
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

fn waggle_system(mut query: Query<(Entity, &mut Transform, &mut Waggle)>, mut commands: Commands) {
    for (entity, mut transform, mut waggle) in query.iter_mut() {
        waggle.update(&mut transform.rotation);
        if waggle.finished() {
            println!("Waggle Finished");
            commands.entity(entity).remove::<Waggle>();
        }
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
                    TimedRemoval::new(Duration::from_millis(500));
                commands
                    .entity(player_entity)
                    .insert(DirectionalSpriteAnimationSpecial(0))
                    .insert(timed_removal);
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
    mut input_events: EventReader<InputEvent>,
    mut move_query: QuerySet<(
        QueryState<(Entity, &TilePos), With<Player>>,
        QueryState<(Entity, &TilePos, Option<&Player>, Option<&Enemy>)>,
        QueryState<(&mut TilePos, &mut MovementAnimate, &Transform, &mut Facing)>,
    )>,
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
            InputEvent::TurnDirection(_dir) => (),
            InputEvent::Wait => {
                let can_take_turn = global_turn_counter
                    .can_take_turn(&local_turn_counter, GamePhase::PlayerMovement);
                if can_take_turn {
                    println!("Player Waiting");
                    local_turn_counter.incr();
                    game_event_writer.send(GameEvent::PhaseComplete(GamePhase::PlayerMovement));
                }
            }
            InputEvent::Power => (),
        }
    }
}

fn debug_print_input_system(
    mut query: QuerySet<(
        QueryState<(&Transform, &GlobalTransform)>,
        QueryState<(&Player)>,
        QueryState<(&TilePos, &Transform), With<Player>>,
    )>,
    input: Res<Input<KeyCode>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    if input.just_pressed(KeyCode::P) {
        for (trans, global_trans) in query.q0().iter() {
            println!("{:?} (Global: {:?}", trans, global_trans)
        }
    }

    if input.just_pressed(KeyCode::T) {
        let (tilepos, player_position) = query.q2().single();
        let end_point = TilePos(tilepos.0, tilepos.1 - 5);
        spawn_projectile(
            commands,
            asset_server,
            texture_atlases,
            player_position.translation.clone(),
            end_point,
        );
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
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    let texture_handle = asset_server.load("sprites/haddock_spritesheet.png");
    let atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(64.0, 64.0), 5, 4);
    let atlas_handle = texture_atlases.add(atlas);
    let start_point = {
        let start_point = cell_map.start_point().unwrap_or((1, 1));
        TilePos(start_point.0 as u32, start_point.1 as u32)
    };
    commands
        .spawn_bundle(TileResidentBundle::new(3, start_point, atlas_handle, 1))
        .insert(Waggle::new(100, -0.4, 0.4, 0.01))
        .insert(CameraFollow {
            x_threshold: 300.0,
            y_threshold: 200.0,
        })
        .insert(Player);

    add_sharks(
        &mut commands,
        &asset_server,
        &mut texture_atlases,
        &cell_map,
    );
}

fn add_test_mesh2d(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut commands: Commands,
) {
    let start_pos = TilePos(22, 22).to_world_pos(20.0);
    commands.spawn_bundle(MaterialMesh2dBundle {
        mesh: meshes.add(Mesh::from(shape::Quad::default())).into(),
        transform: Transform::from_translation(start_pos).with_scale(Vec3::splat(64.)),
        material: materials.add(ColorMaterial::from(Color::PURPLE)),
        ..Default::default()
    });
}

fn add_sharks(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    texture_atlases: &mut ResMut<Assets<TextureAtlas>>,
    cell_map: &CellMap<i32>,
) {
    let texture_handle = asset_server.load("sprites/shark_spritesheet.png");
    let atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(64.0, 64.0), 4, 4);
    let atlas_handle = texture_atlases.add(atlas);
    let spawn_positions = cell_map.distribute_points_by_cost(7);
    //for (x, y) in [(8, 9), (12, 12), (3, 10)].into_iter() {
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
