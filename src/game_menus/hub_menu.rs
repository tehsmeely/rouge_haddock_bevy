use bevy::prelude::*;

use crate::asset_handling::asset::ImageAsset;
use crate::asset_handling::ImageAssetStore;
use crate::game_menus::components::{HubButton, HubMenuOnly};
use crate::menu_core::menu_core;
use crate::menu_core::menu_core::rect_consts::CENTRED;
use crate::menu_core::menu_core::text::{standard_centred_text, standard_centred_text_custom};
use crate::profiles::profiles::{LoadedUserProfile, UserProfile};
use bevy_ui_nodes::{HeightOrWidth, Property};

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

fn menu_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    image_assets: Res<ImageAssetStore>,
    loaded_profile: Res<LoadedUserProfile>,
) {
    let font = asset_server.load("fonts/bigfish/Bigfish.ttf");

    // Always save on loading in
    loaded_profile.save();

    commands
        .spawn_bundle(bevy_ui_nodes::new(bevy_ui_nodes::defaults::full(
            FlexDirection::Row,
            Some(vec![
                Property::Colour(Color::rgba(1., 1., 1., 1.0)),
                Property::Image(image_assets.get(&ImageAsset::Background)),
            ]),
        )))
        //.spawn_bundle(bevy_ui_nodes::defaults::full_horizontal())
        .insert(HubMenuOnly {})
        .with_children(|parent| {
            left_bar_stats_bundle(
                parent,
                font.clone(),
                &image_assets,
                &loaded_profile.user_profile,
            );
            right_bar_button_bundle(parent, font.clone());
        });
}

fn left_bar_stats_bundle(
    parent: &mut ChildBuilder,
    font: Handle<Font>,
    image_assets: &Res<ImageAssetStore>,
    user_profile: &UserProfile,
) {
    let image = image_assets.get(&user_profile.haddock_variant.to_image_asset());
    parent
        .spawn_bundle(bevy_ui_nodes::default_node::half(
            HeightOrWidth::Width,
            FlexDirection::Column,
            Some(vec![bevy_ui_nodes::Property::Justify(
                JustifyContent::Center,
            )]),
        ))
        .with_children(|parent| {
            standard_centred_text_custom(
                parent,
                user_profile.name.clone(),
                font.clone(),
                60.0,
                Color::BLACK,
            );
            parent.spawn_bundle(ImageBundle {
                style: Style {
                    size: Size::new(Val::Px(128.0), Val::Px(128.0)),
                    margin: CENTRED,
                    ..Default::default()
                },
                image: UiImage(image),
                ..Default::default()
            });
            standard_centred_text(
                parent,
                format!("Shells: {}", user_profile.snail_shells),
                font.clone(),
            );
            standard_centred_text(
                parent,
                format!("Level: {}", user_profile.level),
                font.clone(),
            );
        });
}

fn right_bar_button_bundle(parent: &mut ChildBuilder, font: Handle<Font>) {
    parent
        .spawn_bundle(bevy_ui_nodes::default_node::half(
            HeightOrWidth::Width,
            FlexDirection::Column,
            Some(vec![bevy_ui_nodes::Property::Justify(
                JustifyContent::Center,
            )]),
        ))
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
