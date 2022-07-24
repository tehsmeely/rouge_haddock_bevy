
use bevy::prelude::*;

use crate::asset_handling::asset::ImageAsset;
use crate::asset_handling::ImageAssetStore;
use crate::game_menus::components::{StoreButton, StoreMenuOnly};
use crate::menu_core::menu_core;
use crate::menu_core::menu_core::text::{standard_centred_text, standard_centred_text_custom};
use crate::menu_core::structure::SplitWay;
use crate::profiles::profiles::{LoadedUserProfile, UserProfile};
use bevy::prelude::{FlexDirection, JustifyContent};

pub struct MenuPlugin;

#[derive(Debug)]
enum StoreMenuDisplayTextType {
    Shells,
    Stats,
    Cost,
}
#[derive(Component, Debug)]
struct StoreMenuDisplayText(StoreMenuDisplayTextType, Entity, usize);

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        let state = crate::CoreState::GameStore;
        app.add_system_set(SystemSet::on_enter(state).with_system(menu_setup))
            .add_system_set(
                SystemSet::on_update(state)
                    .with_system(menu_core::button_system)
                    .with_system(text_update_system)
                    .with_system(button_click_system),
            )
            .add_system_set(SystemSet::on_exit(state).with_system(menu_cleanup));
    }
}

fn cost_to_level_up(next_level: usize) -> usize {
    use num::Integer;
    next_level.div_ceil(&10usize) * 10usize
}

fn maybe_level_up(profile: &mut UserProfile) -> bool {
    // Subtract shell cost
    // Increase level
    let level_shell_cost = cost_to_level_up(profile.level + 1);
    if level_shell_cost <= profile.snail_shells {
        println!("Levelling up!");
        profile.level += 1;
        profile.snail_shells -= level_shell_cost;
        true
    } else {
        println!("Can't afford to level up!");
        false
    }
}

fn text_update_system(
    mut text_query: Query<&mut Text>,
    text_entity_query: Query<&StoreMenuDisplayText, Changed<StoreMenuDisplayText>>,
    user_profile: Res<LoadedUserProfile>,
) {
    for display_text in text_entity_query.iter() {
        println!("updating text: {:?}", display_text);
        if let Ok(mut text) = text_query.get_mut(display_text.1) {
            text.sections[0].value = match display_text.0 {
                StoreMenuDisplayTextType::Shells => {
                    format!("Shells: {}", user_profile.user_profile.snail_shells)
                }
                StoreMenuDisplayTextType::Stats => {
                    format!(
                        "Level: {}\n\nHealth: {}\nPower Charges: {}",
                        user_profile.user_profile.level,
                        user_profile.user_profile.max_health(),
                        user_profile.user_profile.max_power_charges()
                    )
                }
                StoreMenuDisplayTextType::Cost => {
                    format!(
                        "Cost: {}",
                        cost_to_level_up(user_profile.user_profile.level + 1)
                    )
                }
            };
        }
    }
}

fn trigger_change_on_text_entities(q: &mut Query<&mut StoreMenuDisplayText>) {
    println!("Triggering change");
    for mut store_menu_display_text in q.iter_mut() {
        store_menu_display_text.2 += 1;
    }
}

fn button_click_system(
    interaction_query: Query<(&Interaction, &StoreButton), (With<Button>, Changed<Interaction>)>,
    mut text_entity_query: Query<&mut StoreMenuDisplayText>,
    mut app_state: ResMut<State<crate::CoreState>>,
    mut loaded_profile: ResMut<LoadedUserProfile>,
) {
    for (interaction, button) in interaction_query.iter() {
        if *interaction == Interaction::Clicked {
            match button {
                StoreButton::Back => {
                    app_state.set(crate::CoreState::GameHub).unwrap();
                }
                StoreButton::LevelUp => {
                    println!("Level up!");
                    if maybe_level_up(&mut loaded_profile.user_profile) {
                        loaded_profile.save();
                    }
                    trigger_change_on_text_entities(&mut text_entity_query);
                }
            }
        }
    }
}

fn menu_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    image_asset_store: Res<ImageAssetStore>,
) {
    let font = asset_server.load("fonts/bigfish/Bigfish.ttf");
    // ui camera
    commands
        .spawn_bundle(UiCameraBundle::default())
        .insert(StoreMenuOnly);

    let mut display_text_cost = None;
    let mut display_text_shells = None;
    let mut display_text_stats = None;
    commands
        .spawn_bundle(crate::menu_core::nodes::vertical::full_with_background(
            image_asset_store.get(&ImageAsset::Background),
        ))
        .insert(StoreMenuOnly {})
        .with_children(|parent| {
            crate::menu_core::structure::split_unequal(
                parent,
                SplitWay::Vertical,
                |parent| {
                    crate::menu_core::structure::split(
                        parent,
                        SplitWay::Horizontal,
                        |parent| {
                            let text_nodes = standard_centred_text(
                                parent,
                                "Cost To Level".to_string(),
                                font.clone(),
                            );
                            display_text_cost = Some(StoreMenuDisplayText(
                                StoreMenuDisplayTextType::Cost,
                                text_nodes.text,
                                0,
                            ));
                            let button_size = Size::new(Val::Px(200.0), Val::Px(65.0));
                            menu_core::make_button_custom_size(
                                StoreButton::LevelUp,
                                button_size,
                                parent,
                                font.clone(),
                            );
                        },
                        |parent| {
                            crate::menu_core::structure::split_unequal(
                                parent,
                                SplitWay::Vertical,
                                |parent| {
                                    let text_nodes = standard_centred_text_custom(
                                        parent,
                                        "Shells".to_string(),
                                        font.clone(),
                                        40.0,
                                        Color::WHITE,
                                    );
                                    display_text_shells = Some(StoreMenuDisplayText(
                                        StoreMenuDisplayTextType::Shells,
                                        text_nodes.text,
                                        0,
                                    ));
                                },
                                |parent| {
                                    parent
                                        .spawn_bundle({
                                            use crate::menu_core::nodes::general::*;
                                            new(defaults::full(
                                                FlexDirection::Column,
                                                Some(vec![Property::Justify(
                                                    JustifyContent::Center,
                                                )]),
                                            ))
                                        })
                                        .with_children(|parent| {
                                            let text_nodes = standard_centred_text(
                                                parent,
                                                "Stats".to_string(),
                                                font.clone(),
                                            );
                                            display_text_stats = Some(StoreMenuDisplayText(
                                                StoreMenuDisplayTextType::Stats,
                                                text_nodes.text,
                                                0,
                                            ));
                                        });
                                },
                                20f32,
                            );
                        },
                    );
                },
                |parent| {
                    menu_core::make_button(StoreButton::Back, parent, font.clone());
                },
                70.0,
            )
        });

    for display_text in [display_text_shells, display_text_stats, display_text_cost] {
        commands
            .spawn()
            .insert(display_text.unwrap())
            .insert(StoreMenuOnly);
    }
}

fn menu_cleanup(q: Query<Entity, With<StoreMenuOnly>>, mut commands: Commands) {
    for entity in q.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
