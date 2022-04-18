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
    Back,
}
#[derive(Component)]
pub enum LoadButton {
    Back,
}
#[derive(Component)]
pub enum NewGameButton {
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
        }
    }
}
impl ButtonComponent for LoadButton {
    fn to_text(&self) -> &'static str {
        match self {
            Self::Back => "Back",
        }
    }
}
impl ButtonComponent for NewGameButton {
    fn to_text(&self) -> &'static str {
        match self {
            Self::Back => "Back",
        }
    }
}
