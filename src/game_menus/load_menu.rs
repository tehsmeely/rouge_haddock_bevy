use bevy::app::AppExit;
use bevy::prelude::*;

use crate::asset_handling::asset::ImageAsset;
use crate::asset_handling::ImageAssetStore;
use crate::game_menus::components::{LoadButton, LoadMenuOnly};
use crate::menu_core::menu_core;
use crate::menu_core::menu_core::make_button;
use crate::menu_core::menu_core::rect_consts::CENTRED;
use crate::menu_core::menu_core::text::{standard_centred_text, standard_centred_text_custom};
use crate::profiles::profiles::{load_profiles_blocking, LoadedUserProfile, UserProfile};
use bevy::reflect::erased_serde::private::serde::Serialize;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        let state = crate::CoreState::LoadMenu;
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
    interaction_query: Query<(&Interaction, &LoadButton), (With<Button>, Changed<Interaction>)>,
    mut app_state: ResMut<State<crate::CoreState>>,
) {
    for (interaction, button) in interaction_query.iter() {
        if *interaction == Interaction::Clicked {
            match button {
                LoadButton::Back => {
                    app_state.set(crate::CoreState::MainMenu).unwrap();
                }
            }
        }
    }
}

fn menu_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    image_assets: Res<ImageAssetStore>,
) {
    println!("LoadMenu Setup Start");
    let profile = UserProfile::default();
    let loaded = LoadedUserProfile::new(profile, 0);
    loaded.save();
    let loaded_profiles = load_profiles_blocking();
    let font = asset_server.load("fonts/bigfish/Bigfish.ttf");
    // ui camera
    commands
        .spawn_bundle(UiCameraBundle::default())
        .insert(LoadMenuOnly);

    println!("LoadMenu Setup Middle");
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                margin: Rect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Row,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(LoadMenuOnly {})
        .with_children(|parent| {
            make_button(LoadButton::Back, parent, font.clone());
            for loaded_profile in loaded_profiles.iter() {
                let text = format!("{:?}", loaded_profile.user_profile);
                standard_centred_text(parent, text, font.clone())
            }
        });
    println!("LoadMenu Setup Done");
}
fn menu_cleanup(q: Query<Entity, With<LoadMenuOnly>>, mut commands: Commands) {
    for entity in q.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
