use std::sync::atomic::{AtomicUsize, Ordering};

use bevy::prelude::{Color, FlexDirection, JustifyContent, NodeBundle, Rect, Size, Style, Val};
use bevy::ui::UiColor;

static DEBUG_COLOUR_I: AtomicUsize = AtomicUsize::new(0);
const DEBUG_COLOURS: [Color; 5] = [
    Color::RED,
    Color::BLUE,
    Color::GREEN,
    Color::PINK,
    Color::ORANGE,
];

fn debug_get_colour() -> Color {
    let mut i = DEBUG_COLOUR_I.load(Ordering::Relaxed) + 1;
    if i > DEBUG_COLOURS.len() {
        i = 0;
    }
    DEBUG_COLOUR_I.store(i, Ordering::Relaxed);
    DEBUG_COLOURS[i].clone()
}

pub fn full_width() -> NodeBundle {
    NodeBundle {
        style: Style {
            size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
            margin: Rect::all(Val::Auto),
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Row,
            ..Default::default()
        },
        color: UiColor(debug_get_colour()),
        ..Default::default()
    }
}
pub fn half_width() -> NodeBundle {
    NodeBundle {
        style: Style {
            size: Size::new(Val::Percent(50.0), Val::Percent(100.0)),
            margin: Rect::all(Val::Auto),
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            ..Default::default()
        },
        color: UiColor(debug_get_colour()),
        ..Default::default()
    }
}
