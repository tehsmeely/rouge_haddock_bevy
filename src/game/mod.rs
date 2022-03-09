pub mod components;
mod enemy;
mod events;
mod game;
mod movement;
mod projectile;
mod tilemap;
mod timed_removal;
mod turn;
mod ui;

pub use game::GamePlugin as Plugin;
