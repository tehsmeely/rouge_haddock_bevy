use bevy::prelude::*;

use bevy::utils::Duration;
use bevy::window::WindowId;
use bevy::winit::WinitWindows;
use bevy_ecs_tilemap::TilemapPlugin;
use bevy_kira_audio::AudioPlugin;
use log::info;

use winit::window::Icon;

mod asset_handling;
mod game;
mod game_menus;
mod helpers;
mod main_menu;
mod map_gen;
mod menu_core;
mod profiles;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
enum CoreState {
    Loading,
    MainMenu,
    GameLevel,
    GameLevelTransition,
    GameOverlay,
    GameHub,
    GameStore,
    LoadMenu,
    NewGameMenu,
}

pub fn main() {
    let initial_state = CoreState::Loading;
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(TilemapPlugin)
        .add_plugin(AudioPlugin)
        .add_plugin(crate::game::Plugin)
        .add_plugin(crate::game::GameOverlayPlugin)
        .add_plugin(crate::main_menu::Plugin)
        .add_plugin(crate::asset_handling::Plugin)
        .add_plugin(crate::game_menus::HubMenuPlugin)
        .add_plugin(crate::game_menus::StoreMenuPlugin)
        .add_plugin(crate::game_menus::LoadMenuPlugin)
        .add_plugin(crate::game_menus::NewGameMenuPlugin)
        .add_state(initial_state)
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
    if active_system.0 && time.time_since_startup() > Duration::from_secs(1) {
        let primary = windows.get_primary_mut().unwrap();
        info!("Setting Title");
        let version = env!("CARGO_PKG_VERSION");
        primary.set_title(format!("Rouge Haddock . {}", version));
        active_system.0 = false;
    }
}
