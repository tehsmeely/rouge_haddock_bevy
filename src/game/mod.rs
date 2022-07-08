pub mod components;
mod end_game;
mod enemy;
mod events;
mod game;
mod movement;
mod projectile;
mod snails;
mod tilemap;
mod timed_removal;
mod turn;
mod ui;
mod ui_overlay;

pub use game::GamePlugin as Plugin;
pub use ui_overlay::GameOverlayPlugin;
