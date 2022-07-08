use crate::menu_core::menu_core::text::standard_centred_text;
use bevy::prelude::*;

pub struct GameOverlayPlugin;

#[derive(Debug, Component)]
struct GameOverlayOnly;

impl Plugin for GameOverlayPlugin {
    fn build(&self, app: &mut App) {
        let state = crate::CoreState::GameOverlay;
        app.add_system_set(SystemSet::on_enter(state).with_system(menu_setup))
            .add_system_set(SystemSet::on_update(state).with_system(input_watch_system))
            .add_system_set(SystemSet::on_exit(state).with_system(menu_cleanup));
    }
}

fn input_watch_system(
    mut input: Res<Input<KeyCode>>,
    mut app_state: ResMut<State<crate::CoreState>>,
) {
    if input.just_pressed(KeyCode::B) {
        println!("UI Overlay popping state");
        app_state.pop().unwrap();
    }
}

fn menu_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    println!("UI Overlay");
    let font = asset_server.load("fonts/bigfish/Bigfish.ttf");
    // ui camera
    commands
        .spawn_bundle(UiCameraBundle::default())
        .insert(GameOverlayOnly);

    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                margin: Rect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..Default::default()
            },
            color: UiColor(Color::rgba(0f32, 0f32, 0f32, 0f32)),
            ..Default::default()
        })
        .insert(GameOverlayOnly)
        .with_children(|parent| {
            standard_centred_text(parent, "Hello".to_string(), font);
        });
    println!("UI Overlay setup complete");
}

fn menu_cleanup(q: Query<Entity, With<GameOverlayOnly>>, mut commands: Commands) {
    for entity in q.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
