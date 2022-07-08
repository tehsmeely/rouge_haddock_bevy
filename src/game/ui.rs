use bevy::prelude::*;
use log::info;

use crate::game::components::{Health, Player, PowerCharges};
use crate::game::turn::{GlobalLevelCounter, GlobalTurnCounter};
use crate::helpers::cleanup::recursive_cleanup;

#[derive(Debug, Component)]
pub struct GameUiOnly;

pub struct GameUiPlugin;

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

fn ui_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/bigfish/Bigfish.ttf");
    let banner_height = Val::Px(40.0);
    commands.spawn_bundle(UiCameraBundle::default());
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), banner_height),
                margin: Rect::all(Val::Px(0.0)),
                justify_content: JustifyContent::FlexStart,
                flex_direction: FlexDirection::RowReverse,
                ..Default::default()
            },
            color: UiColor(Color::rgba(0.0, 0.0, 0.0, 0.0)),
            ..Default::default()
        })
        .insert(GameUiOnly {})
        .with_children(|parent| {
            ui_components::health_counter(parent, font.clone(), &banner_height);
            ui_components::power_charge_counter(parent, font.clone(), &banner_height);
            ui_components::turn_counter(parent, font.clone(), &banner_height);
        });
}

fn ui_player_health_system(
    player_query: Query<&Health, (With<Player>, Changed<Health>)>,
    mut ui_query: Query<&mut Text, With<ui_components::HealthCounter>>,
) {
    for health in player_query.iter() {
        info!("Setting health ui to: {}", health.hp);
        for mut text in ui_query.iter_mut() {
            text.sections[0].value = format!("HP: {}", "|".repeat(health.hp));
        }
    }
}

fn ui_player_power_system(
    player_query: Query<&PowerCharges, (With<Player>, Changed<PowerCharges>)>,
    mut ui_query: Query<&mut Text, With<ui_components::PowerChargeCounter>>,
) {
    for charges in player_query.iter() {
        info!("Setting power charge ui to: {}", charges.charges);
        for mut text in ui_query.iter_mut() {
            text.sections[0].value = format!("Charges: {}", "|".repeat(charges.charges));
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

    use crate::menu_core::helpers::RectExt;

    #[derive(Debug, Component)]
    pub struct HealthCounter;

    #[derive(Debug, Component)]
    pub struct PowerChargeCounter;

    #[derive(Debug, Component)]
    pub struct TurnCounter;

    pub fn health_counter(parent: &mut ChildBuilder, font: Handle<Font>, banner_height: &Val) {
        parent
            .spawn_bundle(NodeBundle {
                style: Style {
                    size: Size::new(Val::Px(100.0), *banner_height),
                    margin: Rect::new_2(Val::Px(0.0), Val::Px(10.0)),
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::ColumnReverse,
                    flex_grow: 0.0,
                    ..Default::default()
                },
                color: UiColor(Color::rgb(0.0, 1.0, 0.2)),
                ..Default::default()
            })
            .with_children(|parent| {
                parent
                    .spawn_bundle(TextBundle {
                        text: Text::with_section(
                            "",
                            TextStyle {
                                font,
                                font_size: 35.0,
                                color: Color::rgb(0.0, 0.0, 0.0),
                            },
                            TextAlignment {
                                vertical: VerticalAlign::Center,
                                horizontal: HorizontalAlign::Center,
                            },
                        ),
                        style: Style {
                            flex_grow: 1.0,
                            justify_content: JustifyContent::Center,
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .insert(HealthCounter);
            });
    }

    pub fn power_charge_counter(
        parent: &mut ChildBuilder,
        font: Handle<Font>,
        banner_height: &Val,
    ) {
        parent
            .spawn_bundle(NodeBundle {
                style: Style {
                    size: Size::new(Val::Px(200.0), *banner_height),
                    margin: Rect::new_2(Val::Px(0.0), Val::Px(10.0)),
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::ColumnReverse,
                    flex_grow: 0.0,
                    ..Default::default()
                },
                color: UiColor(Color::rgb(0.0, 1.0, 0.2)),
                ..Default::default()
            })
            .with_children(|parent| {
                parent
                    .spawn_bundle(TextBundle {
                        text: Text::with_section(
                            "",
                            TextStyle {
                                font,
                                font_size: 35.0,
                                color: Color::rgb(0.0, 0.0, 0.0),
                            },
                            TextAlignment {
                                vertical: VerticalAlign::Center,
                                horizontal: HorizontalAlign::Center,
                            },
                        ),
                        style: Style {
                            flex_grow: 1.0,
                            justify_content: JustifyContent::Center,
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .insert(PowerChargeCounter);
            });
    }

    pub fn turn_counter(parent: &mut ChildBuilder, font: Handle<Font>, banner_height: &Val) {
        parent
            .spawn_bundle(NodeBundle {
                style: Style {
                    size: Size::new(Val::Px(200.0), *banner_height),
                    margin: Rect::new_2(Val::Px(0.0), Val::Px(10.0)),
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
                            margin: Rect::new_2(Val::Px(0.0), Val::Px(10.0)),
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
                                text: Text::with_section(
                                    "",
                                    TextStyle {
                                        font,
                                        font_size: 35.0,
                                        color: Color::rgb(0.0, 0.0, 0.0),
                                    },
                                    TextAlignment {
                                        vertical: VerticalAlign::Center,
                                        horizontal: HorizontalAlign::Center,
                                    },
                                ),
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
