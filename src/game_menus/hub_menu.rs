use bevy::app::AppExit;
use bevy::prelude::*;

use crate::asset_handling::asset::ImageAsset;
use crate::asset_handling::ImageAssetStore;
use crate::game_menus::components::{HubButton, HubMenuOnly};
use crate::menu_core::menu_core;
use crate::menu_core::menu_core::rect_consts::CENTRED;
use crate::menu_core::menu_core::text::{standard_centred_text, standard_centred_text_custom};
use crate::profiles::profiles::UserProfile;
use bevy::reflect::erased_serde::private::serde::Serialize;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        let state = crate::CoreState::GameHub;
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
    interaction_query: Query<(&Interaction, &HubButton), (With<Button>, Changed<Interaction>)>,
    mut app_state: ResMut<State<crate::CoreState>>,
) {
    for (interaction, button) in interaction_query.iter() {
        if *interaction == Interaction::Clicked {
            match button {
                HubButton::Run => {
                    app_state.set(crate::CoreState::GameLevel).unwrap();
                }
                HubButton::Quit => {
                    app_state.set(crate::CoreState::MainMenu).unwrap();
                }
                HubButton::Store => {
                    app_state.set(crate::CoreState::GameStore).unwrap();
                }
            }
        }
    }
}

fn debug_insert_user_profile(mut commands: Commands, user_profile: Res<Option<UserProfile>>) {
    if user_profile.is_none() {
        warn!("User profile not populated prior to hub");
        let new_user_profile = UserProfile::default();
        commands.insert_resource(new_user_profile);
    }
}

fn menu_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    image_assets: Res<ImageAssetStore>,
    user_profile: Res<UserProfile>,
) {
    let font = asset_server.load("fonts/bigfish/Bigfish.ttf");
    // ui camera
    commands
        .spawn_bundle(UiCameraBundle::default())
        .insert(HubMenuOnly);

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
        .insert(HubMenuOnly {})
        .with_children(|parent| {
            left_bar_stats_bundle(parent, font.clone(), &image_assets, &user_profile);
            right_bar_button_bundle(parent, font.clone());
        });
}

fn left_bar_stats_bundle(
    parent: &mut ChildBuilder,
    font: Handle<Font>,
    image_assets: &Res<ImageAssetStore>,
    user_profile: &UserProfile,
) {
    let image = image_assets.get(&ImageAsset::HaddockSprite);
    parent
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(50.0), Val::Percent(100.0)),
                margin: Rect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::ColumnReverse,
                ..Default::default()
            },
            color: UiColor(Color::RED),
            ..Default::default()
        })
        .with_children(|parent| {
            // TODO: Load this data from state - and prepare to update it ofc
            standard_centred_text_custom(parent, user_profile.name.clone(), font.clone(), 60.0);
            parent.spawn_bundle(ImageBundle {
                style: Style {
                    size: Size::new(Val::Px(128.0), Val::Px(128.0)),
                    margin: CENTRED,
                    ..Default::default()
                },
                image: UiImage(image),
                ..Default::default()
            });
            standard_centred_text(parent, format!("Eggs: {}", user_profile.eggs), font.clone());
            standard_centred_text(
                parent,
                format!("Level: {}", user_profile.level),
                font.clone(),
            );
        });
}

fn right_bar_button_bundle(parent: &mut ChildBuilder, font: Handle<Font>) {
    parent
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(50.0), Val::Percent(100.0)),
                margin: Rect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..Default::default()
            },
            color: UiColor(Color::GREEN),
            ..Default::default()
        })
        .with_children(|parent| {
            menu_core::make_button(HubButton::Quit, parent, font.clone());
            menu_core::make_button(HubButton::Store, parent, font.clone());
            menu_core::make_button_custom_size(
                HubButton::Run,
                Size::new(Val::Px(300.0), Val::Px(65.0)),
                parent,
                font.clone(),
            );
        });
}

fn menu_cleanup(q: Query<Entity, With<HubMenuOnly>>, mut commands: Commands) {
    for entity in q.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
