use bevy::prelude::*;
use bevy::render::render_resource::internal::bytemuck::cast_ref;
use bevy_ecs_tilemap::{MapQuery, TilePos, TilemapPlugin};
use simple_logger::SimpleLogger;

mod game;
mod helpers;

fn main() {
    //SimpleLogger::new().init().unwrap();
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(TilemapPlugin)
        .add_plugin(crate::game::Plugin)
        .run();
}
