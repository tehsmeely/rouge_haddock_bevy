use crate::game::tilemap::TilePosExt;
use bevy::prelude::*;
use bevy_ecs_tilemap::{Tile, TilePos};

#[derive(Debug, Component)]
pub struct Player;

#[derive(Debug, Component)]
pub struct CameraFollow {
    pub x_threshold: f32,
    pub y_threshold: f32,
}

// Not called "Direction" as to not smash with the Direction in bevy prelude
#[derive(Debug, Clone)]
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
}

#[derive(Debug, Component)]
pub struct Facing(pub MapDirection);

impl Default for Facing {
    fn default() -> Self {
        Self(MapDirection::Left)
    }
}

#[derive(Debug, Component)]
pub struct DirectionalAnimation {
    pub frames_per_direction: usize,
    pub frame_index: usize,
    pub dirty: bool,
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
    pub fn incr(&mut self) {
        self.frame_index = (self.frame_index + 1) % self.frames_per_direction;
        self.dirty = true;
    }

    pub fn index(&self, direction: &MapDirection) -> usize {
        (Self::direction_to_order_index(direction) * self.frames_per_direction) + self.frame_index
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
        self.destination_position = destination_pos;
        self.active = true
    }

    pub fn finished(&self, from: &Vec3) -> bool {
        self.destination_position.eq(from)
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
    directional_animation: DirectionalAnimation,
    tile_pos: TilePos,
    movement_animate: MovementAnimate,
    health: Health,
}

impl TileResidentBundle {
    pub fn new(initial_hp: usize, tile_pos: TilePos, atlas_handle: Handle<TextureAtlas>) -> Self {
        let start_pos = tile_pos.to_world_pos(10.0);
        Self {
            sprite_sheet_bundle: SpriteSheetBundle {
                texture_atlas: atlas_handle.clone(),
                transform: Transform::from_translation(start_pos),
                ..Default::default()
            },
            timer: Timer::from_seconds(0.1, true),
            facing: (Facing::default()),
            directional_animation: DirectionalAnimation::default(),
            tile_pos: (tile_pos),
            movement_animate: (MovementAnimate::default()),
            health: Health { hp: initial_hp },
        }
    }
}
