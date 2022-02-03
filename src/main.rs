use bevy::prelude::*;
use bevy_ecs_tilemap::TilemapPlugin;

mod game;
mod helpers;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(TilemapPlugin)
        .add_plugin(crate::game::tilemap::GameTilemapPlugin)
        .add_startup_system(setup)
        .add_system(animate_sprite_system)
        .add_system(input_handle_system.label("input"))
        .add_system(camera_follow_system.after("input"))
        .run();
}

#[derive(Debug, Component)]
struct Controlled(bool);

#[derive(Debug, Clone)]
enum Direction {
    Up,
    Right,
    Down,
    Left,
}

#[derive(Debug, Component)]
struct CameraFollow;

impl Direction {
    fn to_unit_translation(&self) -> Vec3 {
        match self {
            Self::Up => Vec3::Y,
            Self::Down => -Vec3::Y,
            Self::Right => Vec3::X,
            Self::Left => -Vec3::X,
        }
    }
}

#[derive(Debug, Component)]
struct Facing(Direction);

impl Default for Facing {
    fn default() -> Self {
        Self(Direction::Left)
    }
}

#[derive(Debug, Component)]
struct DirectionalAnimation {
    frames_per_direction: usize,
    frame_index: usize,
    dirty: bool,
}
impl Default for DirectionalAnimation {
    fn default() -> Self {
        Self {
            frames_per_direction: 4,
            frame_index: 0,
            dirty: false,
        }
    }
}
impl DirectionalAnimation {
    fn incr(&mut self) {
        self.frame_index = (self.frame_index + 1) % self.frames_per_direction;
        self.dirty = true;
    }

    fn index(&self, direction: &Direction) -> usize {
        (Self::direction_to_order_index(direction) * self.frames_per_direction) + self.frame_index
    }

    fn direction_to_order_index(direction: &Direction) -> usize {
        match direction {
            Direction::Left => 0,
            Direction::Right => 1,
            Direction::Down => 2,
            Direction::Up => 3,
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

fn camera_follow_system(
    mut query: QuerySet<(
        QueryState<(&Transform), With<CameraFollow>>,
        QueryState<(&mut Transform), With<Camera>>,
    )>,
) {
    let mut pos = None;
    for (transform) in query.q0().iter() {
        pos = Some((transform.translation.x, transform.translation.y));
    }

    if let Some((x, y)) = pos {
        for (mut transform) in query.q1().iter_mut() {
            transform.translation.x = x;
            transform.translation.y = y;
        }
    }
}

fn input_handle_system(
    input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Facing, &Controlled, &mut Transform, &mut Timer)>,
) {
    let new_direction = {
        if input.just_pressed(KeyCode::A) {
            Some(Direction::Left)
        } else if input.just_pressed(KeyCode::D) {
            Some(Direction::Right)
        } else if input.just_pressed(KeyCode::W) {
            Some(Direction::Up)
        } else if input.just_pressed(KeyCode::S) {
            Some(Direction::Down)
        } else {
            None
        }
    };
    if let Some(direction) = &new_direction {
        for (mut facing, controlled, mut transform, mut timer) in query.iter_mut() {
            if controlled.0 {
                let dur = timer.duration();
                timer.tick(dur);
                facing.0 = direction.clone();
                let speed = 64.0;
                transform.translation += direction.to_unit_translation() * speed
            }
        }
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    let texture_handle = asset_server.load("sprites/haddock_spritesheet.png");
    let atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(64.0, 64.0), 4, 4);
    let atlas_handle = texture_atlases.add(atlas);
    commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: atlas_handle.clone(),
            transform: Transform::from_xyz(32., 32., 10.),
            ..Default::default()
        })
        .insert(Timer::from_seconds(0.1, true))
        .insert(Facing::default())
        .insert(DirectionalAnimation::default())
        .insert(CameraFollow {})
        .insert(Controlled(true));
}
