use crate::menu_core::menu_core::ButtonComponent;
use bevy::prelude::Component;

#[derive(Component)]
pub struct HubMenuOnly;
#[derive(Component)]
pub struct StoreMenuOnly;
#[derive(Component)]
pub struct LoadMenuOnly;
#[derive(Component)]
pub struct NewGameMenuOnly;

#[derive(Component)]
pub enum HubButton {
    Run,
    Store,
    Quit,
}
#[derive(Component)]
pub enum StoreButton {
    LevelUp,
    Back,
}
#[derive(Component)]
pub enum LoadButton {
    LoadOrNew,
    Back,
}
#[derive(Component)]
pub enum NewGameButton {
    NewGame,
    Back,
}

impl ButtonComponent for HubButton {
    fn to_text(&self) -> &'static str {
        match self {
            Self::Run => "Start Run",
            Self::Store => "Store",
            Self::Quit => "Quit",
        }
    }
}
impl ButtonComponent for StoreButton {
    fn to_text(&self) -> &'static str {
        match self {
            Self::Back => "Back",
            Self::LevelUp => "Level Up",
        }
    }
}
impl ButtonComponent for LoadButton {
    fn to_text(&self) -> &'static str {
        match self {
            Self::LoadOrNew => "Load",
            Self::Back => "Back",
        }
    }
}
impl ButtonComponent for NewGameButton {
    fn to_text(&self) -> &'static str {
        match self {
            Self::Back => "Back",
            Self::NewGame => "Create New Profile",
        }
    }
}
