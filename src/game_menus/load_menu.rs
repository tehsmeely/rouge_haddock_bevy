use bevy::app::AppExit;
use bevy::prelude::*;
use bevy::reflect::erased_serde::private::serde::Serialize;

use crate::asset_handling::asset::ImageAsset;
use crate::asset_handling::ImageAssetStore;
use crate::game_menus::components::{LoadButton, LoadMenuOnly};
use crate::menu_core::menu_core;
use crate::menu_core::menu_core::make_button;
use crate::menu_core::menu_core::rect_consts::CENTRED;
use crate::menu_core::menu_core::text::{standard_centred_text, standard_centred_text_custom};
use crate::profiles::profiles::{load_profiles_blocking, LoadedUserProfile, UserProfile};

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
    mut commands: Commands,
) {
    for (interaction, button) in interaction_query.iter() {
        if *interaction == Interaction::Clicked {
            match button {
                LoadButton::Back => {
                    app_state.set(crate::CoreState::MainMenu).unwrap();
                }
                LoadButton::Load => {
                    // TODO: Actually load the right thing!
                    commands.insert_resource(LoadedUserProfile::new(UserProfile::default(), 0));
                    app_state.set(crate::CoreState::GameHub).unwrap();
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
        .spawn_bundle(crate::menu_core::nodes::full_width())
        .insert(LoadMenuOnly {})
        .with_children(|parent| {
            make_button(LoadButton::Back, parent, font.clone());
            make_button(LoadButton::Load, parent, font.clone());
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

struct ProfilePicker {}

impl ProfilePicker {
    fn create(builder: &mut ChildBuilder) -> Self {
        builder
            .spawn_bundle(crate::menu_core::nodes::full_width())
            .with_children(|parent| {
                parent.spawn_bundle(crate::menu_core::nodes::half_width());
                parent.spawn_bundle(crate::menu_core::nodes::half_width());
            });
        ProfilePicker {}
    }
}
