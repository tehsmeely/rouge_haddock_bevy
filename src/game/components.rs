use bevy::prelude::*;

#[derive(Debug, Component)]
pub struct Controlled(pub bool);

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
