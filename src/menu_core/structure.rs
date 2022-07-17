use crate::menu_core::nodes::debug_get_colour;
/// Another experimental interface for making notes
use bevy::prelude::*;
use std::ops::Not;

#[cfg(debug_assertions)]
fn get_colour() -> Color {
    debug_get_colour()
}
#[cfg(not(debug_assertions))]
fn get_colour() -> Color {
    Color::hsla(0f32, 0f32, 0f32, 0f32)
}

#[derive(Debug, Clone, Copy)]
pub enum SplitWay {
    Horizontal,
    Vertical,
}

impl Into<FlexDirection> for SplitWay {
    fn into(self) -> FlexDirection {
        match self {
            Self::Horizontal => FlexDirection::RowReverse,
            Self::Vertical => FlexDirection::Column,
        }
    }
}

impl Not for SplitWay {
    type Output = Self;
    fn not(self) -> Self::Output {
        match self {
            Self::Horizontal => Self::Vertical,
            Self::Vertical => Self::Horizontal,
        }
    }
}

pub fn split<F, G>(parent: &mut ChildBuilder, split_way: SplitWay, first: F, second: G)
where
    F: FnOnce(&mut ChildBuilder),
    G: FnOnce(&mut ChildBuilder),
{
    split_unequal(parent, split_way, first, second, 50.0);
}

// First is left or top, depending on direction
pub fn split_unequal<F, G>(
    parent: &mut ChildBuilder,
    split_way: SplitWay,
    first: F,
    second: G,
    first_pct: f32,
) where
    F: FnOnce(&mut ChildBuilder),
    G: FnOnce(&mut ChildBuilder),
{
    let (w1, w2, h1, h2) = {
        let hundred_pct = Val::Percent(100f32);
        match split_way {
            SplitWay::Vertical => (
                hundred_pct,
                hundred_pct,
                Val::Percent(100f32 - first_pct),
                Val::Percent(first_pct),
            ),
            SplitWay::Horizontal => (
                Val::Percent(100f32 - first_pct),
                Val::Percent(first_pct),
                hundred_pct,
                hundred_pct,
            ),
        }
    };
    parent
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                margin: Rect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                flex_direction: split_way.into(),
                ..Default::default()
            },
            color: UiColor(get_colour()),
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(w1, h1),
                        margin: Rect::all(Val::Auto),
                        justify_content: JustifyContent::Center,
                        flex_direction: (!split_way).into(),
                        ..Default::default()
                    },
                    color: UiColor(get_colour()),
                    ..Default::default()
                })
                .with_children(second);
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(w2, h2),
                        margin: Rect::all(Val::Auto),
                        justify_content: JustifyContent::Center,
                        flex_direction: (!split_way).into(),
                        ..Default::default()
                    },
                    color: UiColor(get_colour()),
                    ..Default::default()
                })
                .with_children(first);
        });
}
