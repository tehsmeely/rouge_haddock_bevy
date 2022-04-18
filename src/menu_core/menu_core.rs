use crate::menu_core::helpers::RectExt;
use bevy::asset::Handle;
use bevy::ecs::prelude::{Changed, Query, With};
use bevy::math::{Rect, Size};
use bevy::prelude::{
    AlignItems, BuildChildren, Button, ButtonBundle, ChildBuilder, Color, Component, Font,
    Interaction, JustifyContent, NodeBundle, Style, Text, TextBundle, TextStyle, UiColor, Val,
};

pub const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);
const TRANSPARENT: Color = Color::rgba(0.0, 0.0, 0.0, 0.0);

pub mod rect_consts {
    use bevy::math::Rect;
    use bevy::ui::Val;
    pub const CENTRED: Rect<Val> = Rect {
        left: Val::Auto,
        right: Val::Auto,
        top: Val::Percent(0.0),
        bottom: Val::Px(10.0),
    };
}

pub fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut UiColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                *color = PRESSED_BUTTON.into();
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}

pub trait ButtonComponent: Component {
    fn to_text(&self) -> &'static str;
}

pub fn make_button<C>(button_component: C, parent: &mut ChildBuilder, font: Handle<Font>)
where
    C: ButtonComponent,
{
    let button_size = Size::new(Val::Px(150.0), Val::Px(65.0));
    make_button_custom_size(button_component, button_size, parent, font)
}
pub fn make_button_custom_size<C>(
    button_component: C,
    button_size: Size<Val>,
    parent: &mut ChildBuilder,
    font: Handle<Font>,
) where
    C: ButtonComponent,
{
    let text = button_component.to_text();
    parent
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: button_size,
                // center button
                margin: rect_consts::CENTRED,
                padding: (Rect {
                    left: Val::Percent(0.0),
                    right: Val::Percent(0.0),
                    top: Val::Px(100.0),
                    bottom: Val::Px(100.0),
                }),
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                ..Default::default()
            },
            color: NORMAL_BUTTON.into(),
            ..Default::default()
        })
        .insert(button_component)
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle {
                text: Text::with_section(
                    text,
                    TextStyle {
                        font,
                        font_size: 40.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                    },
                    Default::default(),
                ),
                ..Default::default()
            });
        });
}

pub mod text {
    use crate::menu_core::helpers::RectExt;
    use bevy::prelude::*;

    pub fn standard_centred_text_custom(
        builder: &mut ChildBuilder,
        text: String,
        font: Handle<Font>,
        font_size: f32,
    ) {
        builder
            .spawn_bundle(NodeBundle {
                style: Style {
                    // center button
                    size: Size::new(Val::Auto, Val::Px(font_size + 2.0)),
                    margin: super::rect_consts::CENTRED,
                    padding: Rect::new_2(Val::Px(100.0), Val::Percent(0.0)),
                    // horizontally center child text
                    justify_content: JustifyContent::Center,
                    // vertically center child text
                    align_items: AlignItems::Center,
                    ..Default::default()
                },
                color: UiColor(super::TRANSPARENT),
                ..Default::default()
            })
            .with_children(|parent| {
                parent.spawn_bundle(TextBundle {
                    text: Text::with_section(
                        text,
                        TextStyle {
                            font,
                            font_size: 40.0,
                            color: Color::rgb(0.0, 0.0, 0.0),
                        },
                        Default::default(),
                    ),
                    ..Default::default()
                });
            });
    }
    pub fn standard_centred_text(builder: &mut ChildBuilder, text: String, font: Handle<Font>) {
        let font_size = 40.0;
        standard_centred_text_custom(builder, text, font, font_size);
    }
}