use crate::game::tilemap::TilePosExt;
use bevy::prelude::*;
use bevy::utils::Duration;
use bevy_ecs_tilemap::{Tile, TilePos};
use interpolation::Lerp;
use std::collections::HashMap;

#[derive(Debug, Component)]
pub struct Player;

#[derive(Debug, Component)]
pub struct CameraFollow {
    pub x_threshold: f32,
    pub y_threshold: f32,
}

// Not called "Direction" as to not smash with the Direction in bevy prelude
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum MapDirection {
    Up,
    Right,
    Down,
    Left,
}

impl MapDirection {
    pub fn to_unit_translation(&self) -> Vec3 {
        match self {
            Self::Up => Vec3::Y,
            Self::Down => -Vec3::Y,
            Self::Right => Vec3::X,
            Self::Left => -Vec3::X,
        }
    }

    pub fn to_pos_move(&self) -> (i32, i32) {
        match self {
            Self::Up => (0, 1),
            Self::Down => (0, (-1)),
            Self::Right => (1, 0),
            Self::Left => ((-1), 0),
        }
    }

    const ALL: [MapDirection; 4] = [Self::Up, Self::Right, Self::Down, Self::Left];

    pub fn rand_choice() -> Self {
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        Self::ALL.choose(&mut rng).unwrap().clone()
    }

    pub fn weighted_rand_choice(from_pos: &TilePos, target_pos: &TilePos) -> Self {
        let dx = target_pos.0 as isize - from_pos.0 as isize;
        let dy = target_pos.1 as isize - from_pos.1 as isize;
        let mut costs = HashMap::new();
        if dx.abs() > dy.abs() {
            pick_left_right(dx, 4, 1, &mut costs);
            pick_up_down(dy, 3, 2, &mut costs);
        } else {
            pick_up_down(dy, 4, 1, &mut costs);
            pick_left_right(dx, 3, 2, &mut costs);
        }

        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        let weights = move |map_dir: &MapDirection| costs.get(map_dir).cloned().unwrap_or(0);
        Self::ALL
            .choose_weighted(&mut rng, weights)
            .unwrap()
            .clone()
    }
}

fn pick_up_down(
    dy: isize,
    high_cost: usize,
    low_cost: usize,
    costs: &mut HashMap<MapDirection, usize>,
) {
    if dy > 0 {
        costs.insert(MapDirection::Up, high_cost);
        costs.insert(MapDirection::Down, low_cost);
    } else {
        costs.insert(MapDirection::Down, high_cost);
        costs.insert(MapDirection::Up, low_cost);
    }
}
fn pick_left_right(
    dx: isize,
    high_cost: usize,
    low_cost: usize,
    costs: &mut HashMap<MapDirection, usize>,
) {
    if dx > 0 {
        costs.insert(MapDirection::Right, high_cost);
        costs.insert(MapDirection::Left, low_cost);
    } else {
        costs.insert(MapDirection::Left, high_cost);
        costs.insert(MapDirection::Right, low_cost);
    }
}

#[derive(Debug, Component)]
pub struct Facing(pub MapDirection);

impl Default for Facing {
    fn default() -> Self {
        Self(MapDirection::Left)
    }
}

/// Struct for handling animated sprite frames from a spritesheet where frames depend on direction
#[derive(Debug, Component)]
pub struct DirectionalSpriteAnimation {
    pub regular_frames_per_direction: usize,
    pub special_frames_per_direction: usize,
    pub frame_index: usize,
    pub dirty: bool,
}
impl Default for DirectionalSpriteAnimation {
    fn default() -> Self {
        Self {
            regular_frames_per_direction: 4,
            special_frames_per_direction: 0,
            frame_index: 0,
            dirty: true,
        }
    }
}
impl DirectionalSpriteAnimation {
    // Example of 4 regular frames, 1 special frame layout
    // [0, 1, 2, 3], 4
    // [5, 6, 7, 8], 9
    // [10, 11, 12, 13], 14
    // [15, 16, 17, 18], 19
    pub fn new(regular_frames_per_direction: usize, special_frames_per_direction: usize) -> Self {
        Self {
            regular_frames_per_direction,
            special_frames_per_direction,
            ..Default::default()
        }
    }
    pub fn incr(&mut self) {
        self.frame_index = (self.frame_index + 1) % self.regular_frames_per_direction;
        self.dirty = true;
    }

    fn total_frames_per_direction(&self) -> usize {
        self.regular_frames_per_direction + self.special_frames_per_direction
    }

    pub fn index(&self, direction: &MapDirection) -> usize {
        (Self::direction_to_order_index(direction) * self.total_frames_per_direction())
            + self.frame_index
    }

    pub fn special_index_safe(&self, special_index: usize, direction: &MapDirection) -> usize {
        let offset = if special_index < self.special_frames_per_direction {
            self.regular_frames_per_direction + special_index
        } else {
            warn!("Special Index is greater than expected for this DirectionalSpriteAnimations: {}, available: {}", special_index, self.special_frames_per_direction);
            self.regular_frames_per_direction - 1
        };
        (Self::direction_to_order_index(direction) * self.total_frames_per_direction()) + offset
    }

    pub fn direction_to_order_index(direction: &MapDirection) -> usize {
        match direction {
            MapDirection::Left => 0,
            MapDirection::Right => 1,
            MapDirection::Down => 2,
            MapDirection::Up => 3,
        }
    }
}

/// Component to override the frame index in a [DirectionalSpriteAnimation] with a special frame
#[derive(Component)]
pub struct DirectionalSpriteAnimationSpecial(pub usize);

#[derive(Debug)]
pub struct MouseClickEvent {
    pub button: MouseButton,
    pub world_position: Vec3,
}

#[derive(Component, Debug)]
pub struct MovementAnimate {
    destination_position: Vec3,
    factor: f32, //Per bevy lerp doc: values 0.0-1.0, is ratio of mix from a to b. 1.0 would result in immediate b result
    pub active: bool,
}

impl Default for MovementAnimate {
    fn default() -> Self {
        Self {
            destination_position: Vec3::ZERO,
            factor: 0.5,
            active: false,
        }
    }
}

impl MovementAnimate {
    pub fn lerp(&self, from: &Vec3) -> Vec3 {
        from.lerp(self.destination_position, self.factor)
    }

    pub fn set(&mut self, destination_pos: Vec3) {
        if self.active {
            debug!("Movement animate set to new position whilst still active (This isn't a problem really)");
        }
        self.destination_position = destination_pos;
        self.active = true
    }

    pub fn finished(&self, from: &Vec3) -> bool {
        self.destination_position.eq(from)
    }
}

#[derive(Component, Debug)]
pub struct Waggle {
    count: usize,
    rotation_anticlockwise: f32,
    rotation_clockwise: f32,
    factor: f32,
    current_rotation: f32,
}

impl Waggle {
    pub fn new(
        count: usize,
        rotation_anticlockwise: f32,
        rotation_clockwise: f32,
        factor: f32,
    ) -> Self {
        Self {
            count,
            rotation_anticlockwise,
            rotation_clockwise,
            factor,
            current_rotation: 0f32,
        }
    }
    pub fn update(&mut self, current: &mut Quat, delta: &Duration) {
        if self.count > 0 {
            // TODO this needs some work: The lerp used in this way never completes.
            let target_rotation = if self.count == 1 {
                0f32
            } else if self.count % 2 == 0 {
                self.rotation_anticlockwise
            } else {
                self.rotation_clockwise
            };

            let rotation_direction = (target_rotation - self.current_rotation).signum();
            let rotation_this_step = rotation_direction * self.factor * delta.as_secs_f32();

            let new_rotation = self.current_rotation + rotation_this_step;
            *current = Quat::from_rotation_z(new_rotation);
            self.current_rotation = new_rotation;
            // Target: 30
            // old: 29
            // direction = 30-29 = 1 = 1.0
            // new = 31
            // new_dir = 30-31 = -1 = -1.0
            let overshot = (target_rotation - new_rotation).signum() != rotation_direction;

            if overshot {
                debug!("Waggle reducing: {}->{}", self.count, self.count - 1);
                self.count -= 1;
            }
        } else {
            ()
        }
    }
    pub fn finished(&self) -> bool {
        self.count == 0
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TileType {
    WALL,
    WATER,
}

impl TileType {
    pub fn can_enter(&self) -> bool {
        match self {
            Self::WALL => false,
            Self::WATER => true,
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            Self::WALL => "X",
            Self::WATER => " ",
        }
    }

    pub fn to_raw_tile(&self) -> Tile {
        match self {
            Self::WATER => Tile {
                texture_index: 0,
                ..Default::default()
            },
            Self::WALL => Tile {
                texture_index: 1,
                ..Default::default()
            },
        }
    }
}

#[derive(Component, Debug)]
pub struct Health {
    pub hp: usize,
}

impl Default for Health {
    fn default() -> Self {
        Self { hp: 1 }
    }
}

impl Health {
    pub fn decr_by(&mut self, d: usize) {
        if self.hp >= d {
            self.hp = self.hp.overflowing_sub(d).0;
        } else {
            self.hp = 0
        }
    }
}

#[derive(Bundle, Default)]
pub struct TileResidentBundle {
    #[bundle]
    sprite_sheet_bundle: SpriteSheetBundle,
    timer: Timer,
    facing: Facing,
    directional_animation: DirectionalSpriteAnimation,
    tile_pos: TilePos,
    movement_animate: MovementAnimate,
    health: Health,
}

impl TileResidentBundle {
    pub fn new(
        initial_hp: usize,
        tile_pos: TilePos,
        atlas_handle: Handle<TextureAtlas>,
        special_frames: usize,
    ) -> Self {
        let start_pos = tile_pos.to_world_pos(10.0);
        Self {
            sprite_sheet_bundle: SpriteSheetBundle {
                texture_atlas: atlas_handle.clone(),
                transform: Transform::from_translation(start_pos),
                ..Default::default()
            },
            timer: Timer::from_seconds(0.1, true),
            facing: (Facing::default()),
            directional_animation: DirectionalSpriteAnimation::new(4, special_frames),
            tile_pos: (tile_pos),
            movement_animate: (MovementAnimate::default()),
            health: Health { hp: initial_hp },
        }
    }
}
