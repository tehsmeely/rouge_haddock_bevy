use bevy::prelude::*;

use log::info;

use crate::asset_handling::asset::ImageAsset;
use crate::asset_handling::ImageAssetStore;
use crate::game_menus::components::{NewGameButton, NewGameMenuOnly};
use crate::menu_core::menu_core;

use crate::menu_core::menu_core::text::{standard_centred_text, TextNodes};
use crate::menu_core::menu_core::{make_button, make_button_custom_size};
use crate::menu_core::nodes;
use crate::profiles::profiles::{
    HaddockVariant, LoadedUserProfile, LoadingProfileSlotNum, UserProfile,
};
use std::time::Duration;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        let state = crate::CoreState::NewGameMenu;
        app.add_system_set(SystemSet::on_enter(state).with_system(menu_setup))
            .add_system_set(
                SystemSet::on_update(state)
                    .with_system(menu_core::button_system)
                    .with_system(TextInput::system)
                    .with_system(button_click_system),
            )
            .add_system_set(SystemSet::on_exit(state).with_system(menu_cleanup));
    }
}

fn button_click_system(
    interaction_query: Query<(&Interaction, &NewGameButton), (With<Button>, Changed<Interaction>)>,
    input_query: Query<&TextInput>,
    mut app_state: ResMut<State<crate::CoreState>>,
    loaded_slot_num: Res<LoadingProfileSlotNum>,
    mut commands: Commands,
) {
    for (interaction, button) in interaction_query.iter() {
        if *interaction == Interaction::Clicked {
            match button {
                NewGameButton::Back => {
                    app_state.set(crate::CoreState::LoadMenu).unwrap();
                }
                NewGameButton::NewGame => {
                    let text_input = input_query.single();
                    match text_input.current_string_if_valid() {
                        Some(input) => {
                            let user_profile = LoadedUserProfile::new(
                                UserProfile {
                                    snail_shells: 0,
                                    level: 0,
                                    name: input.to_string(),
                                    haddock_variant: HaddockVariant::Normal,
                                },
                                loaded_slot_num.0,
                            );
                            user_profile.save();
                            commands.insert_resource(user_profile);
                            app_state.set(crate::CoreState::GameHub).unwrap();
                        }
                        None => (println!("Invalid input, ignoring button press")),
                    }
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
    println!("NewGameMenu Setup Start");
    let font = asset_server.load("fonts/bigfish/Bigfish.ttf");

    let mut text_input = None;
    commands
        .spawn_bundle(nodes::general::new(nodes::general::defaults::full(
            FlexDirection::Column,
            Some(vec![nodes::general::Property::Image(
                image_assets.get(&ImageAsset::Background),
            )]),
        )))
        .insert(NewGameMenuOnly {})
        .with_children(|parent| {
            parent
                .spawn_bundle(crate::menu_core::nodes::vertical::half())
                .with_children(|parent| {
                    make_button(NewGameButton::Back, parent, font.clone());
                    make_button_custom_size(
                        NewGameButton::NewGame,
                        Size::new(Val::Px(250.0), Val::Px(65.0)),
                        parent,
                        font.clone(),
                    );
                });
            parent
                .spawn_bundle(crate::menu_core::nodes::vertical::half())
                .with_children(|parent| {
                    text_input = Some(TextInput::create(parent, font.clone()));
                });
        });

    commands
        .spawn()
        .insert(text_input.unwrap())
        .insert(NewGameMenuOnly);
    println!("NewGameMenu Setup Done");
}
fn menu_cleanup(q: Query<Entity, With<NewGameMenuOnly>>, mut commands: Commands) {
    for entity in q.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

//TODO Move this out of this file
#[derive(Debug, Component)]
struct TextInput {
    text_nodes: TextNodes,
    current_text: String,
    cursor_on: bool,
    timer: Timer,
}

impl TextInput {
    const MAX_LEN: usize = 32;
    fn create(parent: &mut ChildBuilder, font: Handle<Font>) -> Self {
        let mut text_nodes = None;
        parent
            .spawn_bundle(crate::menu_core::nodes::horizontal::empty())
            .with_children(|parent| {
                text_nodes = Some(standard_centred_text(parent, "|".into(), font));
            });

        let timer = Timer::new(Duration::from_millis(500), true);
        Self {
            text_nodes: text_nodes.unwrap(),
            current_text: String::new(),
            cursor_on: true,
            timer,
        }
    }

    fn update(&self, text_query: &mut Query<&mut Text>) {
        if let Ok(mut text) = text_query.get_mut(self.text_nodes.text) {
            let cursor = if self.cursor_on { "|" } else { "" };
            text.sections[0].value = format!("{}{}", self.current_text.clone(), cursor);
        }
    }

    fn add_char(&mut self, char: char, text_query: &mut Query<&mut Text>) {
        if self.current_text.len() < Self::MAX_LEN {
            self.current_text.push(char);
            info!("Appending: {:?}. Result: {}", char, self.current_text);
            self.update(text_query);
        }
    }

    fn backspace(&mut self, text_query: &mut Query<&mut Text>) {
        if !self.current_text.is_empty() {
            self.current_text.pop();
            info!("Backspace. Result: {}", self.current_text);
            self.update(text_query);
        }
    }

    fn system(
        mut received_character_events: EventReader<ReceivedCharacter>,
        input_keys: Res<Input<KeyCode>>,
        mut text_query: Query<&mut Text>,
        mut self_query: Query<&mut Self>,
        time: Res<Time>,
    ) {
        let mut text_input = self_query.single_mut();
        for event in received_character_events.iter() {
            //TODO: Sanitise chars here, but also validate before submitting just in case too
            if Self::is_valid_input_char(&event.char) {
                text_input.add_char(event.char, &mut text_query);
            }
        }

        if input_keys.just_pressed(KeyCode::Back) {
            text_input.backspace(&mut text_query);
        }

        text_input.timer.tick(time.delta());

        if text_input.timer.just_finished() {
            text_input.cursor_on = !text_input.cursor_on;
            text_input.update(&mut text_query);
        }
    }

    fn current_string_if_valid(&self) -> Option<&str> {
        let len_ok = 2 <= self.current_text.len() && self.current_text.len() <= 32;
        let chars_ok = {
            self.current_text
                .chars()
                .map(|c| Self::is_valid_input_char(&c))
                .any(|x| x)
        };
        if len_ok && chars_ok {
            Some(&self.current_text)
        } else {
            None
        }
    }

    fn is_valid_input_char(c: &char) -> bool {
        c.is_ascii_alphabetic() || c == &' '
    }
}
