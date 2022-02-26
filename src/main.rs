use bevy::prelude::*;
use bevy::render::render_resource::internal::bytemuck::cast_ref;
use bevy::utils::Duration;
use bevy::window::WindowId;
use bevy::winit::WinitWindows;
use bevy_ecs_tilemap::{MapQuery, TilePos, TilemapPlugin};
use bevy_kira_audio::AudioPlugin;
use log::info;
use simple_logger::SimpleLogger;
use winit::window::Icon;

mod game;
mod helpers;
mod map_gen;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(TilemapPlugin)
        .add_plugin(AudioPlugin)
        .add_plugin(crate::game::Plugin)
        .add_system(setup_window_title)
        .run();
}

struct ActiveSystem(bool);
impl Default for ActiveSystem {
    fn default() -> Self {
        Self(true)
    }
}

fn setup_window_title(
    time: Res<Time>,
    mut windows: ResMut<Windows>,
    mut active_system: Local<ActiveSystem>,
) {
    // If you set title too soon, it causes the window to hang ...
    // TODO: Merge this with some setup/loading phase so the system does not run all the time
    if active_system.0 {
        if time.time_since_startup() > Duration::from_secs(1) {
            let primary = windows.get_primary_mut().unwrap();
            info!("Setting Title");
            let version = env!("CARGO_PKG_VERSION");
            primary.set_title(format!("Rouge Haddock . {}", version));
            active_system.0 = false;
        }
    }
}

// As of right now this causes the program to hang. This is possibly some issue with calling
// winit windows directly instead of via bevy
// When Bevy supports this first-class, will use that instead
fn _setup_window_icon(
    time: Res<Time>,
    windows: Res<WinitWindows>,
    mut active_system: Local<ActiveSystem>,
) {
    if active_system.0 {
        if time.time_since_startup() > Duration::from_secs(10) {
            let primary = windows.get_window(WindowId::primary()).unwrap();
            info!("Setting Icon");
            let (icon_rgba, icon_width, icon_height) = {
                let image = image::open("icon.png")
                    .expect("Failed to open icon path")
                    .into_rgba8();
                let (width, height) = image.dimensions();
                let rgba = image.into_raw();
                (rgba, width, height)
            };

            let icon = Icon::from_rgba(icon_rgba, icon_width, icon_height).unwrap();

            primary.set_window_icon(Some(icon));
            active_system.0 = false;
        }
    }
}
