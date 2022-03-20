use crate::menu::components::{MenuButton, MenuOnly};
use bevy::app::AppExit;
use bevy::prelude::*;

pub struct MenuPlugin;

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        let state = crate::CoreState::MainMenu;
        app.add_system_set(SystemSet::on_enter(state).with_system(menu_setup))
            .add_system_set(
                SystemSet::on_update(state)
                    .with_system(button_system)
                    .with_system(button_click_system),
            )
            .add_system_set(SystemSet::on_exit(state).with_system(menu_cleanup));
    }
}

fn button_system(
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

fn button_click_system(
    interaction_query: Query<(&Interaction, &MenuButton), (With<Button>, Changed<Interaction>)>,
    mut app_state: ResMut<State<crate::CoreState>>,
    mut app_exit_events: EventWriter<AppExit>,
) {
    for (interaction, button) in interaction_query.iter() {
        if *interaction == Interaction::Clicked {
            match button {
                MenuButton::Play => {
                    app_state.set(crate::CoreState::Game).unwrap();
                }
                MenuButton::Quit => app_exit_events.send(AppExit),
            }
        }
    }
}

fn menu_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/bigfish/Bigfish.ttf");
    // ui camera
    commands
        .spawn_bundle(UiCameraBundle::default())
        .insert(MenuOnly);

    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                margin: Rect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(MenuOnly {})
        .with_children(|parent| {
            make_button(MenuButton::Quit, parent, font.clone());
            make_button(MenuButton::Play, parent, font.clone());
        });
}

fn make_button(menu_button: MenuButton, parent: &mut ChildBuilder, font: Handle<Font>) {
    let text = menu_button.to_text();
    parent
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(150.0), Val::Px(65.0)),
                // center button
                margin: (Rect {
                    left: Val::Auto,
                    right: Val::Auto,
                    top: Val::Percent(0.0),
                    bottom: Val::Px(10.0),
                }),
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
        .insert(menu_button)
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

fn menu_cleanup(q: Query<Entity, With<MenuOnly>>, mut commands: Commands) {
    for entity in q.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
