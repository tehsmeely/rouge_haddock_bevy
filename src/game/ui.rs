use crate::game::components::{Health, Player};
use bevy::prelude::*;
use log::info;

trait RectExt {
    fn new_2(v_topbottom: Val, v_leftright: Val) -> Self;
}

impl RectExt for Rect<Val> {
    fn new_2(v_topbottom: Val, v_leftright: Val) -> Self {
        Rect {
            left: v_leftright.clone(),
            right: v_leftright,
            top: v_topbottom.clone(),
            bottom: v_topbottom,
        }
    }
}

pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set((SystemSet::on_enter(crate::State::Game).with_system(ui_setup)))
            .add_system_set(
                (SystemSet::on_update(crate::State::Game).with_system(ui_player_health_system)),
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
                size: Size::new(Val::Percent(100.0), banner_height.clone()),
                margin: Rect::all(Val::Px(0.0)),
                justify_content: JustifyContent::FlexStart,
                flex_direction: FlexDirection::RowReverse,
                ..Default::default()
            },
            color: UiColor(Color::rgb(1.0, 0.0, 0.2)),
            ..Default::default()
        })
        .with_children(|parent| {
            ui_components::health_counter(parent, font.clone(), &banner_height);
        });
}

fn ui_player_health_system(
    player_query: Query<&Health, (With<Player>, Changed<Health>)>,
    mut ui_query: Query<&mut Text, With<ui_components::HealthCounter>>,
) {
    for health in player_query.iter() {
        info!("Setting health ui to: {}", health.hp);
        for mut text in ui_query.iter_mut() {
            text.sections[0].value = "|".repeat(health.hp);
        }
    }
}

mod ui_components {
    use super::RectExt;
    use bevy::prelude::*;

    #[derive(Debug, Component)]
    pub struct HealthCounter;

    pub fn health_counter(parent: &mut ChildBuilder, font: Handle<Font>, banner_height: &Val) {
        parent
            .spawn_bundle(NodeBundle {
                style: Style {
                    size: Size::new(Val::Px(100.0), banner_height.clone()),
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
                            "|||",
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
                            flex_grow: 0.0,
                            justify_content: JustifyContent::Center,
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .insert(HealthCounter);
            });
    }
}
