use bevy::prelude::Component;

#[derive(Component)]
pub struct MenuOnly;

#[derive(Component)]
pub enum MenuButton {
    Play,
    Quit,
}
impl MenuButton {
    pub fn to_text(&self) -> &'static str {
        match self {
            Self::Play => "Play",
            Self::Quit => "Quit",
        }
    }
}
