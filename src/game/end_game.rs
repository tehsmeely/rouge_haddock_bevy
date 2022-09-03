use crate::asset_handling::asset::ImageAsset;
use crate::asset_handling::ImageAssetStore;
use crate::game::components::{
    AnimationTimer, CameraFollow, GameOnly, MovementAnimate, Player, Rotating, Shrinking,
    SimpleSpriteAnimation,
};
use crate::game::events::GameEvent;
use crate::game::tilemap::TilePosExt;
use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use bevy_ecs_tilemap::tiles::TilePos;
use std::time::Duration;

#[derive(Default, Component)]
pub struct InVortex;
#[derive(Default, Component)]
pub struct InHook;
#[derive(Default, Component)]
pub struct EndGameVortex;
#[derive(Default, Component)]
pub struct EndGameHook;
#[derive(Default, Component)]
pub struct EndGameHookLine;

pub struct VortexSpawnEvent;

#[derive(Default, Component, Clone)]
pub struct HookedAnimation {
    timer: Timer,
    speed: f32,
}

const DEFAULT_VORTEX_ROTATION_SPEED: f32 = 2f32;

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

/// Watches for shrinking player (in vortex) and emits event when hits zero size
pub fn vortex_animation_system(
    query: Query<&Transform, (Changed<Transform>, With<Player>, With<InVortex>)>,
    mut game_event_writer: EventWriter<GameEvent>,
) {
    for transform in query.iter() {
        if transform.scale == Vec3::ZERO {
            game_event_writer.send(GameEvent::VortexCompleted);
        }
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
                game_event_writer.send(GameEvent::HookCompleted);
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
    player_query: Query<(Entity, &TilePos), (With<Player>, Changed<TilePos>)>,
    mut game_event_writer: EventWriter<GameEvent>,
    mut commands: Commands,
) {
    if let Ok((hook_entity, hook_tilepos)) = hook_query.get_single() {
        if let Ok((player_entity, player_tilepos)) = player_query.get_single() {
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
                    .insert(InHook)
                    .insert(hooked_animation.clone());
                commands
                    .entity(hook_entity)
                    .insert(hooked_animation.clone());
                commands.entity(hook_line_entity).insert(hooked_animation);
            }
        }
    }
}
pub fn end_game_vortex_system(
    vortex_query: Query<(Entity, &TilePos), With<EndGameVortex>>,
    player_query: Query<(Entity, &TilePos), (With<Player>, Changed<TilePos>)>,
    mut game_event_writer: EventWriter<GameEvent>,
    mut commands: Commands,
) {
    for (_vortex_entity, hook_tilepos) in vortex_query.iter() {
        if let Ok((player_entity, player_tilepos)) = player_query.get_single() {
            if hook_tilepos.eq(player_tilepos) {
                game_event_writer.send(GameEvent::PlayerEnteredVortex);
                println!("Player entered vortex");

                let rotating = Rotating::new(DEFAULT_VORTEX_ROTATION_SPEED * 2.0);
                let shrinking = Shrinking { factor: 1.0 };
                commands
                    .entity(player_entity)
                    .remove::<CameraFollow>()
                    .insert(InVortex)
                    .insert(shrinking)
                    .insert(rotating);
            }
        }
    }
}
pub fn spawn_vortex(
    commands: &mut Commands,
    spawn_pos: TilePos,
    image_store: &Res<ImageAssetStore>,
) {
    let start_pos = spawn_pos.to_world_pos(2.0);
    let transform = Transform::from_translation(start_pos);
    println!("Vortex Spawned");

    let rotating = Rotating::new(2f32);
    commands
        .spawn_bundle(SpriteBundle {
            texture: image_store.get(&ImageAsset::VortexSprite),
            ..Default::default()
        })
        .insert(GameOnly)
        .insert(EndGameVortex)
        .insert(transform)
        .insert(rotating)
        .insert(spawn_pos);
}

pub fn spawn_hook(
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    commands: &mut Commands,
    texture_atlases: &mut ResMut<Assets<TextureAtlas>>,
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
        .insert(GameOnly)
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
        .insert(AnimationTimer(Timer::from_seconds(0.250, true)))
        .insert(EndGameHook)
        .insert(spawn_pos)
        .insert(SimpleSpriteAnimation::new(0, 4));
}
