use crate::asset_handling::asset::ImageAsset;
use crate::asset_handling::ImageAssetStore;
use crate::game::ui::GameOverlayUiRootNode;
use crate::helpers::builders::WithSelf;
use crate::menu_core::menu_core::text::standard_centred_text;
use crate::menu_core::menu_core::ButtonComponent;
use bevy::prelude::*;

pub struct GameOverlayPlugin;

#[derive(Debug, Component)]
struct GameOverlayOnly;

impl Plugin for GameOverlayPlugin {
    fn build(&self, app: &mut App) {
        let state = crate::CoreState::GameOverlay;
        app.add_system_set(SystemSet::on_enter(state).with_system(menu_setup))
            .add_system_set(
                SystemSet::on_update(state)
                    .with_system(crate::menu_core::menu_core::button_system)
                    .with_system(input_watch_system)
                    .with_system(button_click_system),
            )
            .add_system_set(SystemSet::on_exit(state).with_system(menu_cleanup));
    }
}

struct ViewParentNode(Entity);

#[derive(Component, Debug, Clone)]
enum UiOverlayButton {
    Abandon,
    Resume,
    Help,
    Back,
}
impl ButtonComponent for UiOverlayButton {
    fn to_text(&self) -> &'static str {
        match self {
            Self::Abandon => "Abandon",
            Self::Resume => "Resume",
            Self::Help => "Help",
            Self::Back => "Back",
        }
    }
}

fn button_click_system(
    interaction_query: Query<
        (&Interaction, &UiOverlayButton),
        (With<Button>, Changed<Interaction>),
    >,
    mut app_state: ResMut<State<crate::CoreState>>,
    mut commands: Commands,
    view_parent_node: Res<ViewParentNode>,
    current_view_query: Query<Entity, With<OverlayView>>,
    image_store: Res<ImageAssetStore>,
    asset_server: Res<AssetServer>,
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
                    info!("Help pressed");
                    change_view(
                        &view_parent_node,
                        View::Help,
                        &mut commands,
                        &current_view_query,
                        &image_store,
                        &asset_server,
                    );
                }
                UiOverlayButton::Back => {
                    info!("Help pressed");
                    change_view(
                        &view_parent_node,
                        View::Base,
                        &mut commands,
                        &current_view_query,
                        &image_store,
                        &asset_server,
                    );
                }
            }
        }
    }
}

enum View {
    Base,
    Help,
}

fn change_view(
    parent: &ViewParentNode,
    view: View,
    commands: &mut Commands,
    current_view_query: &Query<Entity, With<OverlayView>>,
    image_store: &ImageAssetStore,
    asset_server: &AssetServer,
) {
    // Clear
    for entity in current_view_query.iter() {
        commands.entity(entity).despawn_recursive();
    }

    let font = asset_server.load("fonts/bigfish/Bigfish.ttf");

    // Add
    commands.entity(parent.0).with_children(|parent| {
        match view {
            View::Help => {
                help_view(parent, font, image_store);
            }
            View::Base => {
                base_view(parent, font);
            }
        };
    });
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
    let mut parent_view = None;
    commands.entity(ui_root.0).with_children(|parent| {
        parent
            .spawn_bundle(NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                    margin: UiRect::all(Val::Auto),
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    ..Default::default()
                },
                color: UiColor(Color::rgba(0f32, 0f32, 1.0f32, 0.8f32)),
                ..Default::default()
            })
            .insert(GameOverlayOnly)
            .with_self(|node| parent_view = Some(node.id()))
            .with_children(|parent| {
                base_view(parent, font.clone());
            });
    });
    commands.insert_resource(ViewParentNode(parent_view.unwrap()));
    println!("UI Overlay setup complete");
}

/// This component is on the toplevel parent of the views so can be used to delete all recursively
/// when switching views
#[derive(Component)]
struct OverlayView;

fn base_view(parent: &mut ChildBuilder, font: Handle<Font>) {
    parent
        .spawn_bundle(crate::menu_core::nodes::vertical::full())
        .with_children(|parent| {
            parent
                .spawn_bundle(crate::menu_core::nodes::horizontal::full())
                .with_children(|parent| {
                    crate::menu_core::menu_core::make_button_custom_size(
                        UiOverlayButton::Abandon,
                        Size::new(Val::Px(200.0), Val::Px(65.0)),
                        parent,
                        font.clone(),
                    );
                    crate::menu_core::menu_core::make_button_custom_size(
                        UiOverlayButton::Resume,
                        Size::new(Val::Px(200.0), Val::Px(65.0)),
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
        })
        .insert(OverlayView);
}

fn help_view(parent: &mut ChildBuilder, font: Handle<Font>, image_store: &ImageAssetStore) {
    parent
        .spawn_bundle(crate::menu_core::nodes::vertical::full())
        .with_children(|parent| {
            use crate::menu_core::nodes::general;
            parent
                .spawn_bundle(general::new(general::defaults::full(
                    FlexDirection::Row,
                    Some(vec![
                        general::Property::MarginAll(Val::Auto),
                        general::Property::FlexGrow(0f32),
                        general::Property::FlexBasis(Val::Px(1.0)),
                    ]),
                )))
                .with_children(|parent| {
                    crate::menu_core::menu_core::make_button(
                        UiOverlayButton::Back,
                        parent,
                        font.clone(),
                    );
                });

            parent
                .spawn_bundle(crate::menu_core::nodes::vertical::full())
                .with_children(|parent| {
                    parent.spawn_bundle(general::new(general::defaults::full(
                        FlexDirection::Column,
                        Some(vec![
                            general::Property::Image(image_store.get(&ImageAsset::HelpCard)),
                            general::Property::Colour(Color::WHITE),
                            general::Property::MarginAll(Val::Px(0.0)),
                            general::Property::AspectRatio(1.0),
                            general::Property::Height(Val::Percent(100.0)),
                            general::Property::Width(Val::Auto),
                        ]),
                    )));
                });
        })
        .insert(OverlayView);
}

fn menu_cleanup(q: Query<Entity, With<GameOverlayOnly>>, mut commands: Commands) {
    commands.remove_resource::<ViewParentNode>();
    for entity in q.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
