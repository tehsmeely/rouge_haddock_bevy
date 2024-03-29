use bevy::app::AppExit;
use bevy::prelude::*;

use crate::asset_handling::asset::ImageAsset;
use crate::asset_handling::ImageAssetStore;
use crate::main_menu::components::{MenuButton, MenuOnly};
use crate::menu_core::menu_core;

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
    _commands: Commands,
) {
    for (interaction, button) in interaction_query.iter() {
        if *interaction == Interaction::Clicked {
            match button {
                MenuButton::Play => {
                    app_state.set(crate::CoreState::LoadMenu).unwrap();
                }
                MenuButton::Quit => app_exit_events.send(AppExit),
            }
        }
    }
}

fn menu_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    image_assets: Res<ImageAssetStore>,
) {
    let font = asset_server.load("fonts/bigfish/Bigfish.ttf");

    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                margin: UiRect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..Default::default()
            },
            image: UiImage(image_assets.get(&ImageAsset::Background)),
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
