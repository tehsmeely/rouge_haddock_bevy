use bevy::app::AppExit;
use bevy::prelude::*;

use crate::main_menu::components::{MenuButton, MenuOnly};
use crate::menu_core::menu_core;
use crate::menu_core::menu_core::NORMAL_BUTTON;
use crate::profiles::profiles::UserProfile;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        let state = crate::CoreState::MainMenu;
        app.add_system_set(SystemSet::on_enter(state).with_system(menu_setup))
            .add_system_set(
                SystemSet::on_update(state)
                    .with_system(menu_core::button_system)
                    .with_system(button_click_system),
            )
            .add_system_set(SystemSet::on_exit(state).with_system(menu_cleanup));
    }
}

fn button_click_system(
    interaction_query: Query<(&Interaction, &MenuButton), (With<Button>, Changed<Interaction>)>,
    mut app_state: ResMut<State<crate::CoreState>>,
    mut app_exit_events: EventWriter<AppExit>,
    mut commands: Commands,
) {
    for (interaction, button) in interaction_query.iter() {
        if *interaction == Interaction::Clicked {
            match button {
                MenuButton::Play => {
                    //TODO: Play should not direct to game hub, resource loading to be done
                    // somewhere else!
                    commands.insert_resource(UserProfile::default());
                    app_state.set(crate::CoreState::LoadMenu).unwrap();
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
            menu_core::make_button(MenuButton::Quit, parent, font.clone());
            menu_core::make_button(MenuButton::Play, parent, font.clone());
        });
}

fn menu_cleanup(q: Query<Entity, With<MenuOnly>>, mut commands: Commands) {
    for entity in q.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
