use crate::game::ui::GameOverlayUiRootNode;
use crate::menu_core::menu_core::text::standard_centred_text;
use crate::menu_core::menu_core::ButtonComponent;
use bevy::prelude::*;
use bevy::render::view::VisibleEntities;

pub struct GameOverlayPlugin;

#[derive(Debug, Component)]
struct GameOverlayOnly;

impl Plugin for GameOverlayPlugin {
    fn build(&self, app: &mut App) {
        let state = crate::CoreState::GameOverlay;
        app.add_system_set(SystemSet::on_enter(state).with_system(menu_setup))
            .add_system_set(
                SystemSet::on_update(state)
                    .with_system(input_watch_system)
                    .with_system(button_click_system),
            )
            .add_system_set(SystemSet::on_exit(state).with_system(menu_cleanup));
    }
}

#[derive(Component, Debug, Clone)]
enum UiOverlayButton {
    Abandon,
    Resume,
    Help,
}
impl ButtonComponent for UiOverlayButton {
    fn to_text(&self) -> &'static str {
        match self {
            Self::Abandon => "Abandon",
            Self::Resume => "Resume",
            Self::Help => "Help",
        }
    }
}

fn button_click_system(
    interaction_query: Query<
        (&Interaction, &UiOverlayButton),
        (With<Button>, Changed<Interaction>),
    >,
    mut app_state: ResMut<State<crate::CoreState>>,
) {
    for (interaction, button) in interaction_query.iter() {
        if *interaction == Interaction::Clicked {
            match button {
                UiOverlayButton::Abandon => {
                    // TODO: Implement me
                    info!("Abandon pressed");
                }
                UiOverlayButton::Resume => {
                    info!("Resume pressed");
                    app_state.pop().unwrap();
                }
                UiOverlayButton::Help => {
                    // TODO: Implement me
                    info!("Help pressed");
                }
            }
        }
    }
}

fn input_watch_system(
    mut input: ResMut<Input<KeyCode>>,
    mut app_state: ResMut<State<crate::CoreState>>,
) {
    if input.just_pressed(KeyCode::Escape) {
        println!("UI Overlay popping state");
        input.clear();
        app_state.pop().unwrap();
    }
}

fn menu_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    ui_root: Res<GameOverlayUiRootNode>,
    mut input: ResMut<Input<KeyCode>>,
) {
    println!("UI Overlay");

    // Need to clear input as we want to use ESC to toggle back and forth but without these it
    // just gets stuck in a loop as "just_pressed(esc)" is always true
    input.clear();

    let font = asset_server.load("fonts/bigfish/Bigfish.ttf");
    commands.entity(ui_root.0).with_children(|parent| {
        parent
            .spawn_bundle(NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                    margin: Rect::all(Val::Auto),
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    ..Default::default()
                },
                color: UiColor(Color::rgba(0f32, 0f32, 1.0f32, 0.8f32)),
                ..Default::default()
            })
            .insert(GameOverlayOnly)
            .with_children(|parent| {
                parent
                    .spawn_bundle(crate::menu_core::nodes::horizontal::full())
                    .with_children(|parent| {
                        crate::menu_core::menu_core::make_button(
                            UiOverlayButton::Abandon,
                            parent,
                            font.clone(),
                        );
                        crate::menu_core::menu_core::make_button(
                            UiOverlayButton::Resume,
                            parent,
                            font.clone(),
                        );
                        crate::menu_core::menu_core::make_button(
                            UiOverlayButton::Help,
                            parent,
                            font.clone(),
                        );
                    });
                standard_centred_text(parent, "Hello".to_string(), font);
            });
    });
    println!("UI Overlay setup complete");
}

fn menu_cleanup(q: Query<Entity, With<GameOverlayOnly>>, mut commands: Commands) {
    for entity in q.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
