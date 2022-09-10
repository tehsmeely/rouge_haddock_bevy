use bevy::prelude::*;
use code_location::code_location;
use log::info;
use num::Integer;

use crate::asset_handling::ImageAssetStore;
use crate::game::components::{Health, Player, PowerCharges};
use crate::game::turn::{GlobalLevelCounter, GlobalTurnCounter};
use crate::game::ui::ui_components::{HealthCounter, PowerChargeCounter};
use crate::helpers::cleanup::recursive_cleanup;
use crate::helpers::error_handling::ResultOkLog;
use crate::menu_core::helpers::RectExt;
use crate::menu_core::menu_core::text::standard_centred_text;
use crate::profiles::profiles::LoadedUserProfile;
use bevy::prelude::JustifyContent;
use bevy_ui_nodes::HeightOrWidth;

#[derive(Debug, Component)]
pub struct GameUiOnly;

pub struct GameUiPlugin;

pub struct GameOverlayUiRootNode(pub Entity);

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(crate::CoreState::GameLevel).with_system(ui_setup))
            .add_system_set(
                SystemSet::on_exit(crate::CoreState::GameLevel)
                    .with_system(recursive_cleanup::<GameUiOnly>),
            )
            .add_system_set(
                SystemSet::on_update(crate::CoreState::GameLevel)
                    .with_system(ui_player_health_system)
                    .with_system(ui_player_power_system)
                    .with_system(ui_turn_counter_system),
            );
    }
}

fn ui_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    image_assets: Res<ImageAssetStore>,
    loaded_profile: Res<LoadedUserProfile>,
) {
    let font: Handle<Font> = asset_server.load("fonts/bigfish/Bigfish.ttf");
    let banner_height = Val::Px((ui_components::ICON_HEIGHT * 2.0) + 4.0);
    let mut root_node = None;
    commands
        .spawn_bundle(bevy_ui_nodes::default_node::full_vertical())
        .insert(GameUiOnly {})
        .with_children(|parent| {
            // Bottom Bar
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(100.0), banner_height),
                        margin: UiRect::all(Val::Px(0.0)),
                        justify_content: JustifyContent::FlexStart,
                        flex_direction: FlexDirection::RowReverse,
                        ..Default::default()
                    },
                    color: UiColor(Color::rgba(1.0, 1.0, 0.0, 0.2)),
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent
                        .spawn_bundle({
                            use bevy_ui_nodes::*;
                            new(vec![
                                Property::Height(Val::Percent(100.0)),
                                Property::Width(Val::Auto),
                                Property::FlexGrow(0.0),
                                Property::Direction(FlexDirection::Column),
                                Property::Justify(JustifyContent::FlexStart),
                                Property::Colour(Color::AQUAMARINE),
                                Property::PaddingAll(Val::Px(10.0)),
                            ])
                        })
                        .with_children(|parent| {
                            ui_components::power_charge_counter(
                                parent,
                                loaded_profile.user_profile.max_power_charges(),
                            );
                            ui_components::health_counter(
                                parent,
                                loaded_profile.user_profile.max_health(),
                            );
                        });
                    ui_components::turn_counter(parent, font.clone(), &banner_height);
                });

            // Central Panel
            root_node = Some(
                parent
                    .spawn_bundle(bevy_ui_nodes::default_node::half(
                        HeightOrWidth::Height,
                        FlexDirection::Column,
                        None,
                    ))
                    .id(),
            );

            // Top Bar
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(100.0), banner_height),
                        margin: UiRect::all(Val::Px(0.0)),
                        justify_content: JustifyContent::FlexStart,
                        flex_direction: FlexDirection::RowReverse,
                        ..Default::default()
                    },
                    color: UiColor(Color::rgba(1.0, 0.0, 0.0, 0.2)),
                    ..Default::default()
                })
                .with_children(|parent| {
                    standard_centred_text(parent, "Top Bar!".to_string(), font);
                });
        });
    commands.insert_resource(GameOverlayUiRootNode(root_node.unwrap()));
}

fn ui_player_health_system(
    mut commands: Commands,
    player_query: Query<&Health, (With<Player>, Changed<Health>)>,
    mut ui_query: Query<(Entity, &HealthCounter)>,
    image_assets: Res<ImageAssetStore>,
) {
    if let Ok(health) = player_query.get_single() {
        info!("Setting health ui to: {}", health.hp);
        if let Some((entity, counter)) = ui_query.get_single().ok_log(code_location!()) {
            ui_components::health_counter_set(
                &mut commands,
                entity,
                &image_assets,
                health.hp,
                counter.0,
            );
        }
    }
}

fn ui_player_power_system(
    mut commands: Commands,
    player_query: Query<&PowerCharges, (With<Player>, Changed<PowerCharges>)>,
    mut ui_query: Query<(Entity, &PowerChargeCounter)>,
    image_assets: Res<ImageAssetStore>,
) {
    if let Ok(charges) = player_query.get_single() {
        info!("Setting power charge ui to: {}", charges.charges);
        if let Some((entity, counter)) = ui_query.get_single().ok_log(code_location!()) {
            ui_components::power_charge_counter_set(
                &mut commands,
                entity,
                &image_assets,
                charges.charges,
                counter.0,
            );
        }
    }
}

fn ui_turn_counter_system(
    global_turn_counter: Res<GlobalTurnCounter>,
    global_level_counter: Res<GlobalLevelCounter>,
    mut last_set_turn: Local<usize>,
    mut ui_query: Query<&mut Text, With<ui_components::TurnCounter>>,
    mut double_set: Local<usize>,
) {
    if (*last_set_turn != global_turn_counter.turn_count) || *double_set > 0 {
        info!(
            "Setting turn counter ui to: {}",
            global_turn_counter.turn_count
        );
        for mut text in ui_query.iter_mut() {
            text.sections[0].value = format!(
                "{} - Turn: {}",
                global_level_counter.level(),
                global_turn_counter.turn_count
            );
        }
        *last_set_turn = global_turn_counter.turn_count;
        *double_set = match *double_set {
            0_usize => 1,
            _ => 0,
        };
    }
}

mod ui_components {
    use bevy::prelude::*;

    use crate::asset_handling::asset::ImageAsset;
    use crate::asset_handling::ImageAssetStore;
    use crate::menu_core::helpers::RectExt;
    use bevy::prelude::{FlexDirection, JustifyContent};
    use bevy::ui::UiImage;

    pub const ICON_HEIGHT: f32 = 42.0;

    #[derive(Debug, Component)]
    pub struct HealthCounter(pub usize);

    #[derive(Debug, Component)]
    pub struct PowerChargeCounter(pub usize);

    #[derive(Debug, Component)]
    pub struct TurnCounter;

    pub fn health_counter(parent: &mut ChildBuilder, max: usize) {
        use bevy_ui_nodes::*;
        println!("HEALTH COUNTER");
        // Node, right aligned, with ui_images stacking right to left
        parent
            .spawn_bundle(new(vec![
                Property::Justify(JustifyContent::FlexStart),
                Property::Height(Val::Percent(100.0)),
                Property::Width(Val::Percent(100.0)),
                Property::Direction(FlexDirection::RowReverse),
            ]))
            .insert(HealthCounter(max));
    }

    pub fn health_counter_set(
        commands: &mut Commands,
        root: Entity,
        image_assets: &ImageAssetStore,
        value: usize,
        max: usize,
    ) {
        commands.entity(root).despawn_descendants();
        commands.entity(root).with_children(|parent| {
            let num_icons = {
                // TODO: Better div_ceil??
                let mut m = max / 2;
                if max % 2 > 0 {
                    m += 1;
                }
                m
            };
            for i in 0usize..num_icons {
                let representative = 2 + (i * 2);
                let asset = if representative > value {
                    if representative.saturating_sub(1) == value {
                        ImageAsset::UiHealthHalf
                    } else {
                        ImageAsset::UiHealthEmpty
                    }
                } else {
                    ImageAsset::UiHealthFull
                };
                parent.spawn_bundle(image_node(image_assets, &asset));
            }
        });
    }

    pub fn power_charge_counter(parent: &mut ChildBuilder, max: usize) {
        use bevy_ui_nodes::*;
        println!("POWER CHARGE COUNTER");
        // Node, right aligned, with ui_images stacking right to left
        parent
            .spawn_bundle(new(vec![
                Property::Justify(JustifyContent::FlexStart),
                Property::Height(Val::Percent(100.0)),
                Property::Width(Val::Percent(100.0)),
                Property::Direction(FlexDirection::RowReverse),
            ]))
            .insert(PowerChargeCounter(max));
    }

    pub fn power_charge_counter_set(
        commands: &mut Commands,
        root: Entity,
        image_assets: &ImageAssetStore,
        value: usize,
        max: usize,
    ) {
        commands.entity(root).despawn_descendants();
        commands.entity(root).with_children(|parent| {
            for i in 0..max {
                let asset = if i < value {
                    ImageAsset::UiPowerFull
                } else {
                    ImageAsset::UiPowerEmpty
                };
                parent.spawn_bundle(image_node(image_assets, &asset));
            }
        });
    }

    fn image_node(image_assets: &ImageAssetStore, asset: &ImageAsset) -> NodeBundle {
        use bevy_ui_nodes::*;
        let properties = vec![
            Property::Colour(Color::WHITE),
            Property::Height(Val::Px(ICON_HEIGHT)),
            Property::Width(Val::Px(ICON_HEIGHT)),
            Property::Image(image_assets.get(asset)),
            Property::FlexGrow(0.0),
        ];
        new(properties)
    }

    pub fn counter_text<I: Component>(
        parent: &mut ChildBuilder,
        font: Handle<Font>,
        banner_height: &Val,
        identifier: I,
    ) {
        println!("TURN COUNTER");
        parent
            .spawn_bundle(NodeBundle {
                style: Style {
                    size: Size::new(Val::Px(200.0), *banner_height),
                    margin: UiRect::new_2(Val::Px(0.0), Val::Px(10.0)),
                    justify_content: JustifyContent::FlexStart,
                    flex_direction: FlexDirection::Column,
                    flex_grow: 1.0,
                    ..Default::default()
                },
                color: UiColor(Color::rgba(0.0, 0.0, 0.0, 0.0)),
                ..Default::default()
            })
            .with_children(|parent| {
                parent
                    .spawn_bundle(NodeBundle {
                        style: Style {
                            size: Size::new(Val::Px(150.0), *banner_height),
                            margin: UiRect::new_2(Val::Px(0.0), Val::Px(10.0)),
                            justify_content: JustifyContent::Center,
                            flex_direction: FlexDirection::ColumnReverse,
                            flex_grow: 0.0,
                            flex_shrink: 1.0,
                            ..Default::default()
                        },
                        color: UiColor(Color::rgb(0.0, 1.0, 0.2)),
                        ..Default::default()
                    })
                    .with_children(|parent| {
                        parent
                            .spawn_bundle(TextBundle {
                                text: Text::from_section(
                                    "A",
                                    TextStyle {
                                        font,
                                        font_size: 35.0,
                                        color: Color::rgb(0.0, 0.0, 0.0),
                                    },
                                )
                                .with_alignment(TextAlignment {
                                    vertical: VerticalAlign::Center,
                                    horizontal: HorizontalAlign::Center,
                                }),
                                style: Style {
                                    flex_grow: 1.0,
                                    justify_content: JustifyContent::Center,
                                    ..Default::default()
                                },
                                ..Default::default()
                            })
                            .insert(identifier);
                    });
            });
    }

    pub fn turn_counter(parent: &mut ChildBuilder, font: Handle<Font>, banner_height: &Val) {
        println!("TURN COUNTER");
        parent
            .spawn_bundle(NodeBundle {
                style: Style {
                    size: Size::new(Val::Px(200.0), *banner_height),
                    margin: UiRect::new_2(Val::Px(0.0), Val::Px(10.0)),
                    justify_content: JustifyContent::FlexStart,
                    flex_direction: FlexDirection::Column,
                    flex_grow: 1.0,
                    ..Default::default()
                },
                color: UiColor(Color::rgba(1.0, 0.0, 0.0, 0.0)),
                ..Default::default()
            })
            .with_children(|parent| {
                parent
                    .spawn_bundle(NodeBundle {
                        style: Style {
                            size: Size::new(Val::Px(150.0), *banner_height),
                            margin: UiRect::new_2(Val::Px(0.0), Val::Px(10.0)),
                            justify_content: JustifyContent::Center,
                            flex_direction: FlexDirection::ColumnReverse,
                            flex_grow: 0.0,
                            flex_shrink: 1.0,
                            ..Default::default()
                        },
                        color: UiColor(Color::rgb(0.0, 1.0, 0.2)),
                        ..Default::default()
                    })
                    .with_children(|parent| {
                        parent
                            .spawn_bundle(TextBundle {
                                text: Text::from_section(
                                    "AAA",
                                    TextStyle {
                                        font,
                                        font_size: 35.0,
                                        color: Color::rgb(0.0, 0.0, 0.0),
                                    },
                                )
                                .with_alignment(TextAlignment {
                                    vertical: VerticalAlign::Center,
                                    horizontal: HorizontalAlign::Center,
                                }),
                                style: Style {
                                    flex_grow: 1.0,
                                    justify_content: JustifyContent::Center,
                                    ..Default::default()
                                },
                                ..Default::default()
                            })
                            .insert(TurnCounter);
                    });
            });
    }
}
