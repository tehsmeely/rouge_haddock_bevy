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
    if i >= DEBUG_COLOURS.len() {
        i = 0;
    }
    DEBUG_COLOUR_I.store(i, Ordering::Relaxed);

    let mut c = DEBUG_COLOURS[i].clone();
    c.set_a(0.1);
    c
}

pub mod general {
    use crate::menu_core::nodes::debug_get_colour;
    use bevy::prelude::*;

    #[derive(Debug)]
    pub enum Property {
        Colour(Color),
        Height(Val),
        Width(Val),
        Margin(Val),
        Image(Handle<Image>),
        Justify(JustifyContent),
        Direction(FlexDirection),
    }

    pub mod defaults {
        use super::*;

        pub fn full(direction: FlexDirection, extra: Option<Vec<Property>>) -> Vec<Property> {
            let mut props = vec![
                Property::Height(Val::Percent(100.0)),
                Property::Width(Val::Percent(100.0)),
                Property::Margin(Val::Auto),
                Property::Direction(direction),
            ];
            if let Some(mut extra_props) = extra {
                props.append(&mut extra_props);
            }
            props
        }
    }

    #[derive(Debug)]
    struct Properties {
        colour: Color,
        height: Val,
        width: Val,
        margin: Val,
        image: Handle<Image>,
        justify: JustifyContent,
        direction: FlexDirection,
    }

    impl Default for Properties {
        fn default() -> Self {
            Self {
                colour: debug_get_colour(),
                height: Val::default(),
                width: Val::default(),
                margin: Val::default(),
                image: Default::default(),
                justify: JustifyContent::default(),
                direction: FlexDirection::default(),
            }
        }
    }

    impl Properties {
        fn set(&mut self, property: Property) {
            match property {
                Property::Colour(color) => self.colour = color,
                Property::Height(val) => self.height = val,
                Property::Width(val) => self.width = val,
                Property::Margin(val) => self.margin = val,
                Property::Image(image) => self.image = image,
                Property::Justify(justify_content) => self.justify = justify_content,
                Property::Direction(flex_direction) => self.direction = flex_direction,
            }
        }
    }

    /// Create default node bundle with values overridden by passed properties.
    /// A given [Property] enum value can exist multiple times in the vec, the latest one will
    /// be applied.
    pub fn new(properties: Vec<Property>) -> NodeBundle {
        println!("Making new general node. Props: {:?}", properties);
        let mut prop = Properties::default();
        println!(".");
        for property in properties.into_iter() {
            prop.set(property);
        }

        println!("Prop set: {:?}", prop);

        NodeBundle {
            style: Style {
                size: Size::new(prop.width, prop.height),
                margin: Rect::all(prop.margin),
                justify_content: prop.justify,
                flex_direction: prop.direction,
                ..Default::default()
            },
            color: UiColor(prop.colour),
            image: UiImage(prop.image),
            ..Default::default()
        }
    }
}

pub mod horizontal {
    use super::debug_get_colour;
    use bevy::prelude::*;
    pub fn full() -> NodeBundle {
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
    pub fn half() -> NodeBundle {
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
    pub fn empty() -> NodeBundle {
        NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(0.0), Val::Percent(100.0)),
                margin: Rect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                flex_grow: 0_f32,
                ..Default::default()
            },
            color: UiColor(debug_get_colour()),
            ..Default::default()
        }
    }
}

pub mod vertical {
    use super::debug_get_colour;
    use bevy::prelude::*;

    pub fn full() -> NodeBundle {
        let background: Handle<Image> = Default::default();
        full_with_background_(background, true)
    }
    pub fn full_with_background(background: Handle<Image>) -> NodeBundle {
        full_with_background_(background, false)
    }
    pub fn full_with_background_(background: Handle<Image>, use_colour: bool) -> NodeBundle {
        let color = if use_colour {
            UiColor(debug_get_colour())
        } else {
            Default::default()
        };
        NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                margin: Rect::all(Val::Auto),
                justify_content: JustifyContent::FlexStart,
                flex_direction: FlexDirection::Column,
                ..Default::default()
            },
            color,
            image: UiImage(background),
            ..Default::default()
        }
    }
    pub fn half() -> NodeBundle {
        NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(50.0)),
                margin: Rect::all(Val::Auto),
                justify_content: JustifyContent::FlexStart,
                flex_direction: FlexDirection::Row,
                ..Default::default()
            },
            color: UiColor(debug_get_colour()),
            ..Default::default()
        }
    }
    pub fn empty() -> NodeBundle {
        NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(0.0)),
                margin: Rect::all(Val::Auto),
                justify_content: JustifyContent::FlexStart,
                flex_direction: FlexDirection::Row,
                flex_grow: 0_f32,
                ..Default::default()
            },
            color: UiColor(debug_get_colour()),
            ..Default::default()
        }
    }
}
