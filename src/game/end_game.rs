use crate::game::components::{
    CameraFollow, GameOnly, MovementAnimate, Player, SimpleSpriteAnimation,
};
use crate::game::events::GameEvent;
use crate::game::tilemap::TilePosExt;
use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use bevy_ecs_tilemap::TilePos;
use std::time::Duration;

#[derive(Default, Component)]
pub struct EndGameHook;
#[derive(Default, Component)]
pub struct EndGameHookLine;

#[derive(Default, Component, Clone)]
pub struct HookedAnimation {
    timer: Timer,
    speed: f32,
}

impl HookedAnimation {
    fn new(duration_s: f32, speed: f32) -> Self {
        Self {
            timer: Timer::from_seconds(duration_s, false),
            speed,
        }
    }
    fn tick(&mut self, delta: Duration) -> bool {
        self.timer.tick(delta);
        self.timer.finished()
    }
}

pub fn hooked_animation_system(
    mut query: Query<(
        Entity,
        &mut Transform,
        &mut HookedAnimation,
        Option<&Player>,
    )>,
    time: Res<Time>,
    mut commands: Commands,
    mut game_event_writer: EventWriter<GameEvent>,
) {
    for (entity, mut transform, mut hooked_animation, maybe_player) in query.iter_mut() {
        let finished = hooked_animation.tick(time.delta());
        if finished {
            commands.entity(entity).remove::<HookedAnimation>();
            if maybe_player.is_some() {
                game_event_writer.send(GameEvent::EndOfLevel);
            }
        } else {
            let distance = {
                let y = -1.0 * hooked_animation.speed * time.delta().as_secs_f32();
                Vec3::new(0.0, y, 0.0)
            };
            transform.translation += distance;
        }
    }
}

pub fn end_game_hook_system(
    hook_query: Query<(Entity, &TilePos), With<EndGameHook>>,
    hook_line_query: Query<Entity, With<EndGameHookLine>>,
    player_query: Query<(Entity, &TilePos), With<Player>>,
    mut game_event_writer: EventWriter<GameEvent>,
    mut already_triggered: Local<bool>,
    mut commands: Commands,
) {
    if let Ok((hook_entity, hook_tilepos)) = hook_query.get_single() {
        if !*already_triggered {
            let (player_entity, player_tilepos) = player_query.single();

            if hook_tilepos.eq(player_tilepos) {
                game_event_writer.send(GameEvent::PlayerHooked);
                println!("Player hook hooked");
                let hook_line_entity = hook_line_query.single();
                // TODO: Unstable player state when triggering this animation
                // Immediately triggering this might not factor in player movement since
                // it's animated and the player arrives at TilePos after the component is set
                // Maybe use some "InsertAfterDelay" component/system or handle elsewhere
                let hooked_animation = HookedAnimation::new(1.0, -900.0);
                commands
                    .entity(player_entity)
                    .remove::<MovementAnimate>()
                    .remove::<CameraFollow>()
                    .insert(hooked_animation.clone());
                commands
                    .entity(hook_entity)
                    .insert(hooked_animation.clone());
                commands.entity(hook_line_entity).insert(hooked_animation);

                *already_triggered = true;
            }
        }
    }
}

pub fn spawn(
    mut meshes: &mut ResMut<Assets<Mesh>>,
    mut materials: &mut ResMut<Assets<ColorMaterial>>,
    mut commands: &mut Commands,
    mut texture_atlases: &mut ResMut<Assets<TextureAtlas>>,
    asset_server: &Res<AssetServer>,
    spawn_pos: TilePos,
) {
    let start_pos = spawn_pos.to_world_pos(20.0);
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
    commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh,
            transform: Transform::from_translation(offset_start_pos),
            material,
            ..Default::default()
        })
        .insert(EndGameHookLine);
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
        .insert(EndGameHook)
        .insert(spawn_pos)
        .insert(SimpleSpriteAnimation::new(4));
}
