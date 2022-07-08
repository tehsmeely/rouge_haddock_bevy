use bevy::prelude::*;
use bevy::reflect::Map;
use bevy_ecs_tilemap::{MapQuery, TilePos};
use code_location::code_location;
use log::info;

use crate::helpers::error_handling::ResultOkLog;

use super::{
    components::*,
    enemy::Enemy,
    events::{GameEvent, InputEvent},
    tilemap::{HasTileType, TilePosExt},
    timed_removal::{TimedRemoval, TimedRemovalPlugin},
    turn::{GamePhase, GlobalTurnCounter, TurnCounter},
};
use crate::game::events::{InfoEvent, PowerEvent};
use crate::game::movement::{AttackCriteria, MoveDecisions};

use crate::game::ui::GameUiPlugin;
use crate::helpers::cleanup::recursive_cleanup;
use crate::map_gen::cell_map::CellMap;

use bevy_kira_audio::Audio;

use crate::asset_handling::asset::{ImageAsset, TextureAtlasAsset};
use crate::asset_handling::{ImageAssetStore, TextureAtlasStore};
use crate::game::end_game::{EndGameHook, EndGameVortex};
use crate::game::turn::GlobalLevelCounter;
use crate::profiles::profiles::LoadedUserProfile;
use bevy::render::render_resource::Texture;
use std::time::Duration;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        let state = crate::CoreState::GameLevel;
        app.add_system_set(SystemSet::on_enter(state).with_system(setup))
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
                    .with_system(player_death_animation_system.after("enemy_movement"))
                    .with_system(sfx_system)
                    .with_system(waggle_system)
                    .with_system(rotate_system)
                    .with_system(shrinking_system)
                    .with_system(end_of_game_watcher_system)
                    .with_system(vortex_spawner_system)
                    .with_system(end_of_level_event_system)
                    .with_system(regular_game_enable_watcher)
                    .with_system(super::end_game::end_game_hook_system)
                    .with_system(super::end_game::end_game_vortex_system)
                    .with_system(super::end_game::hooked_animation_system)
                    .with_system(super::end_game::vortex_animation_system)
                    .with_system(super::projectile::projectile_watcher_system)
                    .with_system(super::projectile::projectile_system)
                    .with_system(super::snails::snail_pickup_system),
            )
            .add_system_set(
                SystemSet::on_exit(state)
                    .with_system(recursive_cleanup::<GameOnly>)
                    .with_system(state_cleanup)
                    .with_system(super::tilemap::cleanup),
            )
            .add_system_set(
                SystemSet::on_enter(crate::CoreState::GameLevelTransition)
                    .with_system(game_level_transition),
            )
            .add_plugin(TimedRemovalPlugin)
            .add_plugin(GameUiPlugin)
            .add_event::<super::events::GameEvent>()
            .add_event::<super::events::InputEvent>()
            .add_event::<super::events::InfoEvent>()
            .add_event::<super::events::PowerEvent>()
            .add_event::<MouseClickEvent>()
            .insert_resource(GlobalTurnCounter::default())
            .insert_resource(GlobalLevelCounter::default())
            .insert_resource(SnailsCollectedThisRun(0_usize))
            .insert_resource(RegularGameEnable {
                enabled: false,
                disable_cycle_count: 1,
            });
    }
}

/// Resource to indicate regular game process. Serves to be disabled at edges like when animating
/// end of game so some things can skip processing or ignore changes
pub struct RegularGameEnable {
    pub enabled: bool,
    pub disable_cycle_count: usize,
}

/// Resource indicates snails collected this run (persists across levels but is processed when
/// exiting (via death or hook)
pub struct SnailsCollectedThisRun(pub usize);

fn regular_game_enable_watcher(mut regular_game_enable: ResMut<RegularGameEnable>) {
    if regular_game_enable.disable_cycle_count > 0 {
        regular_game_enable.disable_cycle_count -= 1;
        if regular_game_enable.disable_cycle_count == 0 {
            regular_game_enable.enabled = true;
        }
    }
}

fn state_cleanup(mut global_turn_counter: ResMut<GlobalTurnCounter>) {
    global_turn_counter.reset();
}

fn game_level_transition(
    mut state: ResMut<State<crate::CoreState>>,
    mut global_level_counter: ResMut<GlobalLevelCounter>,
) {
    info!("Game Level Transition!");
    global_level_counter.increment();
    state.set(crate::CoreState::GameLevel).unwrap();
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
            GameEvent::PlayerDied
            | GameEvent::PlayerHooked
            | GameEvent::HookCompleted
            | GameEvent::PlayerEnteredVortex
            | GameEvent::VortexCompleted => (),
        }
    }
}

fn set_state_handle_error(state: &mut State<crate::CoreState>, new_state: crate::CoreState) {
    let result = state.set(new_state);
    if let Err(e) = result {
        warn!("Error setting state, not considering it a problem: {:?}", e);
    }
}

///
fn end_of_run(
    state: &mut State<crate::CoreState>,
    died: bool,
    global_level_counter: &mut GlobalLevelCounter,
    snail_shells_collected_this_run: &mut SnailsCollectedThisRun,
    loaded_profile: &mut LoadedUserProfile,
) {
    global_level_counter.reset();

    if !died {
        //Only get to keep eggs if didn't die
        loaded_profile.user_profile.snail_shells += snail_shells_collected_this_run.0
    }
    snail_shells_collected_this_run.0 = 0;
    set_state_handle_error(state, crate::CoreState::GameHub);
}

fn end_of_level_event_system(
    mut state: ResMut<State<crate::CoreState>>,
    mut game_event_reader: EventReader<GameEvent>,
    mut global_level_counter: ResMut<GlobalLevelCounter>,
    mut snails_collected_this_run: ResMut<SnailsCollectedThisRun>,
    mut loaded_profile: ResMut<LoadedUserProfile>,
) {
    for event in game_event_reader.iter() {
        match event {
            GameEvent::HookCompleted => end_of_run(
                &mut state,
                false,
                &mut global_level_counter,
                &mut snails_collected_this_run,
                &mut loaded_profile,
            ),
            GameEvent::PlayerDied => end_of_run(
                &mut state,
                true,
                &mut global_level_counter,
                &mut snails_collected_this_run,
                &mut loaded_profile,
            ),
            GameEvent::VortexCompleted => {
                set_state_handle_error(&mut state, crate::CoreState::GameLevelTransition);
            }
            GameEvent::PlayerHooked
            | GameEvent::PhaseComplete(_)
            | GameEvent::PlayerEnteredVortex => (),
        }
    }
}
fn vortex_spawner_system(
    mut commands: Commands,
    image_store: Res<ImageAssetStore>,
    cell_map: ResMut<CellMap<i32>>,
    player_query: Query<&TilePos, With<Player>>,
    enemy_query: Query<Entity, With<Enemy>>,
    global_turn_counter: Res<GlobalTurnCounter>,
    existing_vortex_query: Query<Entity, With<EndGameVortex>>,
) {
    let no_vortex_exists = existing_vortex_query.is_empty();
    let ready_to_spawn = {
        let enemy_count = enemy_query.iter().count();
        let turn_past_threshold = global_turn_counter.turn_count > 34;
        let not_too_many_enemies = enemy_count < 4;

        // Late spawn is dependent on being many turns in and killed *some* enemies
        let can_late_spawn = (turn_past_threshold || not_too_many_enemies);

        // Early spawn is if all enemies are killed. Turn count stops this accidentally triggering
        // before enemies spawn at start
        let can_early_spawn = enemy_count == 0 && global_turn_counter.turn_count > 2;
        can_late_spawn || can_early_spawn
    };
    if ready_to_spawn && no_vortex_exists {
        info!(
            "Spawning phase vortex! Turn: {}",
            global_turn_counter.turn_count
        );
        let player_pos = player_query.single().as_i32s();
        let new_cell_map = cell_map.recalculate(player_pos);
        let spawn_pos = {
            let (x, y) = new_cell_map
                .distribute_points_by_cost(1, None)
                .first()
                .unwrap()
                .to_owned();
            TilePos(x as u32, y as u32)
        };
        super::end_game::spawn_vortex(&mut commands, spawn_pos, &image_store);
        // TODO: Trigger sound
    }
}

fn end_of_game_watcher_system(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut commands: Commands,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut input_event_reader: EventReader<InputEvent>,
    asset_server: Res<AssetServer>,
    cell_map: ResMut<CellMap<i32>>,
    player_query: Query<&TilePos, With<Player>>,
    enemy_query: Query<Entity, With<Enemy>>,
    global_turn_counter: Res<GlobalTurnCounter>,
    hook_query: Query<Entity, With<EndGameHook>>,
) {
    let no_hook_exists = hook_query.is_empty();
    let end_of_game = {
        let mut hook_input = false;
        for event in input_event_reader.iter() {
            if let InputEvent::Hook = event {
                hook_input = true;
            }
        }
        hook_input
    };
    if end_of_game && no_hook_exists {
        info!(
            "Spawning end of game hook! Turn: {}",
            global_turn_counter.turn_count
        );
        let player_pos = player_query.single().as_i32s();
        let new_cell_map = cell_map.recalculate(player_pos);
        let spawn_pos = {
            let (x, y) = new_cell_map
                .distribute_points_by_cost(1, None)
                .first()
                .unwrap()
                .to_owned();
            TilePos(x as u32, y as u32)
        };
        super::end_game::spawn_hook(
            &mut meshes,
            &mut materials,
            &mut commands,
            &mut texture_atlases,
            &asset_server,
            spawn_pos,
        );
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

fn rotate_system(time: Res<Time>, mut query: Query<(&mut Transform, &mut Rotating)>) {
    for (mut transform, mut rotating) in query.iter_mut() {
        rotating.update(&mut transform.rotation, &time.delta());
    }
}

fn shrinking_system(time: Res<Time>, mut query: Query<(&mut Transform, &Shrinking)>) {
    for (mut transform, shrinking) in query.iter_mut() {
        let new_scale = transform.scale - (shrinking.factor * time.delta_seconds());
        transform.scale = new_scale.clamp(Vec3::ZERO, Vec3::ONE);
    }
}

fn player_death_animation_system(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut PlayerDeathAnimation)>,
    mut event_writer: EventWriter<GameEvent>,
) {
    for (entity, mut transform, mut player_death_animation) in query.iter_mut() {
        let finished = player_death_animation.update(&mut transform, &time.delta());
        println!("PlayerDeathAnim: {:?}", player_death_animation);
        if finished {
            commands.entity(entity).remove::<PlayerDeathAnimation>();
            event_writer.send(GameEvent::PlayerDied);
        }
    }
}

fn simple_animate_sprite_system(
    time: Res<Time>,
    mut query: Query<(
        &mut AnimationTimer,
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
        &mut AnimationTimer,
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
    mut query: ParamSet<(
        Query<(&Transform, &CameraFollow)>,
        Query<&mut Transform, With<GameCamera>>,
    )>,
) {
    let pos = query
        .p0()
        .get_single()
        .ok_log(code_location!())
        .map(|(transform, follow)| {
            (
                transform.translation.x,
                transform.translation.y,
                follow.x_threshold,
                follow.y_threshold,
            )
        });

    if let Some((x, y, x_threshold, y_threshold)) = pos {
        if let Some(mut camera_transform) = query.p1().get_single_mut().ok_log(code_location!()) {
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

            if let Some(camera_transform) = camera_query.get_single().ok_log(code_location!()) {
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

fn input_handle_system(
    input: Res<Input<KeyCode>>,
    mut input_events: EventWriter<InputEvent>,
    regular_game_enable: Res<RegularGameEnable>,
    mut app_state: ResMut<State<crate::CoreState>>,
) {
    if input.just_pressed(KeyCode::Escape) {
        println!("Starting GameOverlay");
        app_state.push(crate::CoreState::GameOverlay).unwrap();
        return;
    }
    fn input_to_event(input: &Input<KeyCode>) -> Option<InputEvent> {
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
                return Some(InputEvent::MoveDirection(dir));
            }
            (Some(dir), true) => {
                return Some(InputEvent::TurnDirection(dir));
            }
            (None, _) => (),
        }

        if input.just_pressed(KeyCode::Space) {
            return Some(InputEvent::Wait);
        }

        if input.just_pressed(KeyCode::Q) {
            return Some(InputEvent::Power);
        }

        if input.just_pressed(KeyCode::R) {
            return Some(InputEvent::Hook);
        }

        None
    }
    if let Some(event) = input_to_event(&input) {
        if regular_game_enable.enabled {
            input_events.send(event);
        }
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
    mut regular_game_enable: ResMut<RegularGameEnable>,
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
                if regular_game_enable.enabled {
                    info_event_writer.send(InfoEvent::PlayerHurt);
                } else {
                    info!("Not emitting PlayerHurt event as regular game not enabled")
                }
            }
            _ => (),
        }
        *known_player_hp = Some(health.hp);
        if health.hp == 0 {
            println!("Player! died {:?}", entity);
            let delay = Duration::from_millis(500);
            commands
                .entity(entity)
                .insert(PlayerDeathAnimation::new(delay, 100f32));
            info_event_writer.send(InfoEvent::PlayerKilled);
            regular_game_enable.enabled = false;
        }
    }
}

fn sfx_system(
    mut info_event_reader: EventReader<InfoEvent>,
    audio: Res<Audio>,
    assets: Res<AssetServer>,
) {
    // TODO: Move audio to asset system
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
            InfoEvent::PlayerKilled => {
                debug!("Playing Audio for Player Died");
                let sound = assets.load("audio/398068__happyparakeet__pixel-death.wav");
                audio.play(sound);
            }
            InfoEvent::PlayerPickedUpSnail => {
                debug!("Playing Audio for Player Picked Up Snail");
                let sound = assets.load("audio/608431__plasterbrain__shiny-coin-pickup.flac");
                audio.play(sound);
            }
        }
    }
}
fn enemy_system(
    mut game_event_writer: EventWriter<GameEvent>,
    global_turn_counter: Res<GlobalTurnCounter>,
    mut local_turn_counter: Local<TurnCounter>,
    enemy_query: Query<(Entity, &CanMoveDistance, &MoveWeighting), With<Enemy>>,
    health_query: Query<&mut Health>,
    mut move_query: ParamSet<(
        Query<&TilePos, With<Player>>,
        Query<&TilePos, With<Enemy>>,
        Query<(Entity, &TilePos, Option<&Player>, Option<&Enemy>)>,
        Query<(&mut TilePos, &mut MovementAnimate, &Transform, &mut Facing)>,
    )>,
    mut map_query: MapQuery,
    tile_type_query: Query<&HasTileType>,
) {
    let player_position = *move_query.p0().get_single().unwrap();
    if global_turn_counter.can_take_turn(&mut local_turn_counter, GamePhase::EnemyMovement) {
        let attack_criteria = AttackCriteria::for_enemy();
        let mut move_decisions = MoveDecisions::new();
        let mut moved_to = Vec::new();
        for (entity, can_move_distance, move_weights) in enemy_query.iter() {
            let current_pos = *move_query.p1().get(entity).unwrap();
            let direction =
                MapDirection::weighted_rand_choice(&current_pos, &player_position, move_weights);
            let decision = super::movement::decide_move(
                &current_pos,
                &direction,
                can_move_distance.get(&direction),
                &attack_criteria,
                move_query.p2(),
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

        super::movement::apply_move(move_decisions, move_query.p3(), health_query);
        local_turn_counter.incr();
        game_event_writer.send(GameEvent::PhaseComplete(GamePhase::EnemyMovement));
    }
}

fn player_movement_watcher(
    player_position_query: Query<&TilePos, (With<Player>, Changed<TilePos>)>,
    mut known_player_position: Local<Option<TilePos>>,
    mut info_event_writer: EventWriter<InfoEvent>,
    regular_game_enable: Res<RegularGameEnable>,
) {
    if let Ok(player_tilepos) = player_position_query.get_single() {
        match *known_player_position {
            Some(pos) if pos != *player_tilepos => {
                if regular_game_enable.enabled {
                    info_event_writer.send(InfoEvent::PlayerMoved);
                } else {
                    info!("Not emitting PlayerMoved as regular game not enabled");
                }
            }
            _ => (),
        }
        *known_player_position = Some(*player_tilepos);
    }
}

fn player_movement_system(
    mut game_event_writer: EventWriter<GameEvent>,
    mut power_event_writer: EventWriter<PowerEvent>,
    mut input_events: EventReader<InputEvent>,
    mut move_query: ParamSet<(
        Query<(Entity, &TilePos), With<Player>>,
        Query<(Entity, &TilePos, Option<&Player>, Option<&Enemy>)>,
        Query<(&mut TilePos, &mut MovementAnimate, &Transform, &mut Facing)>,
        Query<&mut Facing, With<Player>>,
    )>,
    mut power_query: Query<&mut PowerCharges, With<Player>>,
    mut health_query: Query<&mut Health>,
    tile_type_query: Query<&HasTileType>,
    mut map_query: MapQuery,
    global_turn_counter: Res<GlobalTurnCounter>,
    mut local_turn_counter: Local<TurnCounter>,
) {
    for event in input_events.iter() {
        match event {
            InputEvent::MoveDirection(direction) => {
                let can_take_turn = global_turn_counter
                    .can_take_turn(&mut local_turn_counter, GamePhase::PlayerMovement);
                if can_take_turn {
                    let (player_entity, current_pos) = {
                        let q = move_query.p0();
                        let (player_entity, current_pos) = q.get_single().unwrap();
                        (player_entity.clone(), current_pos.clone())
                    };

                    let move_decision = super::movement::decide_move(
                        &current_pos,
                        direction,
                        1,
                        &AttackCriteria::for_player(),
                        move_query.p1(),
                        &mut map_query,
                        &tile_type_query,
                        &vec![],
                    );

                    super::movement::apply_move_single(
                        player_entity,
                        &move_decision,
                        &mut move_query.p2(),
                        &mut health_query,
                    );

                    local_turn_counter.incr();
                    game_event_writer.send(GameEvent::PhaseComplete(GamePhase::PlayerMovement));
                } else {
                    info!(
                        "Can't take turn: {:?} {:?}",
                        global_turn_counter, local_turn_counter
                    );
                }
            }
            InputEvent::TurnDirection(dir) => {
                let can_take_turn = global_turn_counter
                    .can_take_turn(&mut local_turn_counter, GamePhase::PlayerMovement);
                if can_take_turn {
                    info!("Player Turning: {:?}", dir);
                    move_query.p3().single_mut().0 = dir.clone();
                    local_turn_counter.incr();
                    game_event_writer.send(GameEvent::PhaseComplete(GamePhase::PlayerMovement));
                }
            }
            InputEvent::Wait => {
                let can_take_turn = global_turn_counter
                    .can_take_turn(&mut local_turn_counter, GamePhase::PlayerMovement);
                if can_take_turn {
                    info!("Player Waiting");
                    local_turn_counter.incr();
                    game_event_writer.send(GameEvent::PhaseComplete(GamePhase::PlayerMovement));
                }
            }
            InputEvent::Power => {
                let can_take_turn = global_turn_counter
                    .can_take_turn(&mut local_turn_counter, GamePhase::PlayerMovement);
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
            InputEvent::Hook => {
                //Do nothing here, handled in hook spawner system
            }
        }
    }
}

fn player_power_system(
    mut query: ParamSet<(
        Query<(&Transform, &TilePos, &Facing), With<Player>>,
        Query<(Entity, &TilePos), With<Enemy>>,
    )>,
    mut commands: Commands,
    atlases: Res<TextureAtlasStore>,
    mut power_event_reader: EventReader<PowerEvent>,
    mut map_query: MapQuery,
    tile_type_query: Query<&HasTileType>,
) {
    for event in power_event_reader.iter() {
        match event {
            PowerEvent::PowerFired => {
                let (start_pos, tilepos, direction): (Vec3, TilePos, MapDirection) = {
                    let q = query.p0();
                    let (transform, tilepos, facing) = q.single();
                    ((*transform).translation, *tilepos, facing.0.clone())
                };
                let fate = super::projectile::scan_to_endpoint(
                    &tilepos,
                    &direction,
                    &query.p1(),
                    &mut map_query,
                    &tile_type_query,
                );
                let end_point = *fate.tile_pos();
                let end_target_entity = fate.entity();
                super::projectile::spawn_projectile(
                    &mut commands,
                    &atlases,
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
    mut query: ParamSet<(
        Query<(&Transform, &GlobalTransform)>,
        Query<Entity, With<Player>>,
        Query<(&TilePos, &Transform), With<Player>>,
        Query<&TilePos, With<Enemy>>,
    )>,
    mut player_health_q: Query<&mut Health, With<Player>>,
    mut player_charges_q: Query<&mut PowerCharges, With<Player>>,
    input: Res<Input<KeyCode>>,
    mut cell_map: ResMut<CellMap<i32>>,
    mut commands: Commands,
    mut asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    atlases: Res<TextureAtlasStore>,
    mut info_event_writer: EventWriter<InfoEvent>,
    image_assets: Res<ImageAssetStore>,
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
            let (TilePos(x, y), _trans) = q.single();
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

    if input.just_pressed(KeyCode::Key6) {
        let mut health = player_health_q.single_mut();
        health.hp += 3;
    }
    if input.just_pressed(KeyCode::Key7) {
        let mut charges = player_charges_q.single_mut();
        charges.charges += 3;
    }

    if input.just_pressed(KeyCode::Key8) {
        info!("Spawning Vortex");
        let spawn_pos = {
            let q = query.p2();
            let (TilePos(x, y), _trans) = q.single();
            TilePos(*x + 1, *y)
        };
        super::end_game::spawn_vortex(&mut commands, spawn_pos, &image_assets)
    }
}
fn setup(
    mut commands: Commands,
    image_assets: Res<ImageAssetStore>,
    texture_atlas_store: Res<TextureAtlasStore>,
    map_query: MapQuery,
    global_level_counter: Res<GlobalLevelCounter>,
) {
    let border_size = 20usize;
    let cell_map: CellMap<i32> = {
        let normalised = crate::map_gen::get_cell_map(50, 50);
        normalised.offset((border_size as i32, border_size as i32))
    };
    println!("Final CellMap: {:?}", cell_map);
    super::tilemap::init_tilemap(
        &mut commands,
        &image_assets,
        map_query,
        &cell_map,
        border_size,
    );
    commands
        .spawn_bundle(OrthographicCameraBundle::new_2d())
        .insert(GameOnly)
        .insert(GameCamera);
    let atlas_handle = texture_atlas_store.get(&TextureAtlasAsset::HaddockSpritesheet);
    let start_point = {
        let start_point = cell_map.start_point().unwrap_or((1, 1));
        TilePos(start_point.0 as u32, start_point.1 as u32)
    };
    commands
        .spawn_bundle(TileResidentBundle::new(
            3,
            start_point.clone(),
            atlas_handle,
            1,
        ))
        .insert(CameraFollow {
            x_threshold: 300.0,
            y_threshold: 200.0,
        })
        .insert(PowerCharges::new(3))
        .insert(Player);
    let mut spawned_positions = Vec::new();
    let shark_positions =
        super::enemy::add_sharks(&mut commands, &texture_atlas_store, 7, &cell_map, None);
    spawned_positions.extend_from_slice(&shark_positions[..]);
    let crab_positions = super::enemy::add_crabs(
        &mut commands,
        &texture_atlas_store,
        3,
        &cell_map,
        Some(&spawned_positions),
    );
    spawned_positions.extend_from_slice(&crab_positions[..]);
    let (snail_num, snail_positions) = super::snails::choose_number_of_and_spawn_snails(
        &mut commands,
        &texture_atlas_store,
        &cell_map,
        Some(&spawned_positions),
    );
    info!("Spawned {}", snail_num);
    spawned_positions.extend_from_slice(&crab_positions[..]);
    commands.insert_resource(cell_map);
    let regular_game_enable = RegularGameEnable {
        enabled: false,
        disable_cycle_count: 2,
    };
    commands.insert_resource(regular_game_enable);
    info!(
        "Completed Setup for level :{}",
        global_level_counter.level()
    );
}
