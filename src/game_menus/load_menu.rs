
use bevy::prelude::*;


use crate::asset_handling::asset::ImageAsset;
use crate::asset_handling::ImageAssetStore;
use crate::game_menus::components::{LoadButton, LoadMenuOnly};

use crate::menu_core::helpers::RectExt;
use crate::menu_core::menu_core;

use crate::menu_core::menu_core::text::{
    standard_centred_text, TextNodes,
};
use crate::menu_core::menu_core::{make_button, ButtonComponent};
use crate::profiles::profiles::{
    load_profiles_blocking, LoadingProfileSlotNum, ProfileSlot,
};
use bevy::prelude::{FlexDirection, JustifyContent};


pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        let state = crate::CoreState::LoadMenu;
        app.add_system_set(SystemSet::on_enter(state).with_system(menu_setup))
            .add_system_set(
                SystemSet::on_update(state)
                    .with_system(menu_core::button_system)
                    .with_system(button_click_system)
                    .with_system(profile_picker_click_system),
            )
            .add_system_set(SystemSet::on_exit(state).with_system(menu_cleanup));
    }
}

fn button_click_system(
    interaction_query: Query<(&Interaction, &LoadButton), (With<Button>, Changed<Interaction>)>,
    profile_picker_query: Query<&ProfilePicker>,
    mut app_state: ResMut<State<crate::CoreState>>,
    mut commands: Commands,
) {
    for (interaction, button) in interaction_query.iter() {
        if *interaction == Interaction::Clicked {
            match button {
                LoadButton::Back => {
                    app_state.set(crate::CoreState::MainMenu).unwrap();
                }
                LoadButton::LoadOrNew => {
                    let picker = profile_picker_query.single();
                    let slot: &ProfileSlot = picker.get_current_slot();
                    match slot {
                        ProfileSlot::Loaded(user_profile) => {
                            commands.insert_resource(user_profile.clone());
                            app_state.set(crate::CoreState::GameHub).unwrap();
                        }
                        ProfileSlot::Free(slot_num) => {
                            commands.insert_resource(LoadingProfileSlotNum(*slot_num));
                            app_state.set(crate::CoreState::NewGameMenu).unwrap();
                        }
                    }
                }
            }
        }
    }
}
fn profile_picker_click_system(
    interaction_query: Query<
        (&Interaction, &ProfilePickerButton),
        (With<Button>, Changed<Interaction>),
    >,
    mut profile_picker_query: Query<&mut ProfilePicker>,
    mut text_query: Query<&mut Text>,
    mut image_query: Query<&mut UiImage>,
    image_asset_store: Res<ImageAssetStore>,
) {
    let mut profile_picker = profile_picker_query.single_mut();
    for (interaction, button) in interaction_query.iter() {
        if *interaction == Interaction::Clicked {
            let change = match button {
                ProfilePickerButton::Left => -1,
                ProfilePickerButton::Right => 1,
            };
            profile_picker.change(
                change,
                &mut text_query,
                &mut image_query,
                &image_asset_store,
            );
        }
    }
    if !profile_picker.initialised {
        profile_picker.initialise(&mut text_query, &mut image_query, &image_asset_store);
    }
}

fn menu_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    image_assets: Res<ImageAssetStore>,
) {
    println!("LoadMenu Setup Start");
    let loaded_profiles = load_profiles_blocking();
    let font = asset_server.load("fonts/bigfish/Bigfish.ttf");
    // ui camera
    commands
        .spawn_bundle(UiCameraBundle::default())
        .insert(LoadMenuOnly);

    println!("LoadMenu Setup Middle");
    let mut profile_picker = None;
    let mut load_button_text_entity = None;
    commands
        .spawn_bundle(crate::menu_core::nodes::vertical::full_with_background(
            image_assets.get(&ImageAsset::Background),
        ))
        .insert(LoadMenuOnly {})
        .with_children(|parent| {
            parent
                .spawn_bundle({
                    use crate::menu_core::nodes::general::*;
                    new(vec![
                        Property::Direction(FlexDirection::Row),
                        Property::Height(Val::Percent(20f32)),
                        Property::Width(Val::Percent(100f32)),
                        Property::Margin(Val::Percent(0f32)),
                        Property::Justify(JustifyContent::Center),
                    ])
                })
                .with_children(|parent| {
                    make_button(LoadButton::Back, parent, font.clone());
                    let (_button, text) = make_button(LoadButton::LoadOrNew, parent, font.clone());
                    load_button_text_entity = Some(text);
                });
            parent
                .spawn_bundle(crate::menu_core::nodes::vertical::half())
                .with_children(|parent| {
                    profile_picker = Some(ProfilePicker::create(
                        parent,
                        font.clone(),
                        loaded_profiles,
                        load_button_text_entity.unwrap(),
                        &image_assets,
                    ));
                });
        });
    commands
        .spawn()
        .insert(profile_picker.unwrap())
        .insert(LoadMenuOnly {});
    println!("LoadMenu Setup Done");
}
fn menu_cleanup(q: Query<Entity, With<LoadMenuOnly>>, mut commands: Commands) {
    for entity in q.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

#[derive(Debug, Component)]
struct ProfilePicker {
    text_nodes: TextNodes,
    current_index: i32,
    loaded_profiles: Vec<ProfileSlot>,
    load_button_entity: Entity,
    image_entity: Entity,
    initialised: bool, // Texts may be out of date until initialised
}

#[derive(Debug, Component)]
enum ProfilePickerButton {
    Left,
    Right,
}

impl ButtonComponent for ProfilePickerButton {
    fn to_text(&self) -> &'static str {
        match self {
            Self::Left => "<",
            Self::Right => ">",
        }
    }
}

impl ProfilePicker {
    fn profile_specific_texts(slot: &ProfileSlot) -> (String, String) {
        match slot {
            ProfileSlot::Loaded(loaded_user_profile) => {
                let picker_text = {
                    let up = &loaded_user_profile.user_profile;
                    format!(
                        "{}\nLevel {}\n({} Shells)",
                        up.name, up.level, up.snail_shells
                    )
                };
                let button_text = String::from("Load");
                (picker_text, button_text)
            }
            ProfileSlot::Free(slot_num) => (format!("Empty: {}", slot_num), String::from("New")),
        }
    }

    fn image_from_slot(slot: &ProfileSlot, image_asset_store: &ImageAssetStore) -> Handle<Image> {
        match slot {
            ProfileSlot::Free(_) => Default::default(),
            ProfileSlot::Loaded(loaded_profile) => {
                image_asset_store.get(&loaded_profile.user_profile.haddock_variant.to_image_asset())
            }
        }
    }

    fn create(
        builder: &mut ChildBuilder,
        font: Handle<Font>,
        loaded_profiles: Vec<ProfileSlot>,
        load_button_entity: Entity,
        image_asset_store: &ImageAssetStore,
    ) -> Self {
        let mut text_nodes = None;
        let mut image_entity = None;
        builder
            .spawn_bundle(crate::menu_core::nodes::horizontal::full())
            .with_children(|parent| {
                parent
                    .spawn_bundle(crate::menu_core::nodes::horizontal::empty())
                    .with_children(|parent| {
                        make_button(ProfilePickerButton::Left, parent, font.clone());
                    });
                parent
                    .spawn_bundle(crate::menu_core::nodes::horizontal::half())
                    .with_children(|parent| {
                        parent
                            .spawn_bundle(crate::menu_core::nodes::general::new(
                                crate::menu_core::nodes::general::defaults::full(
                                    FlexDirection::Column,
                                    Some(vec![
                                        crate::menu_core::nodes::general::Property::Justify(
                                            JustifyContent::Center,
                                        ),
                                    ]),
                                ),
                            ))
                            //.spawn_bundle(crate::menu_core::nodes::vertical::full())
                            .with_children(|parent| {
                                let image_scale = 2.0;
                                let image_size = 64f32 * image_scale;
                                image_entity = Some(
                                    parent
                                        .spawn_bundle(ImageBundle {
                                            style: Style {
                                                size: Size::new(
                                                    Val::Px(image_size),
                                                    Val::Px(image_size),
                                                ),
                                                margin: Rect::new_2(Val::Px(0f32), Val::Auto),
                                                ..Default::default()
                                            },
                                            image: UiImage(Self::image_from_slot(
                                                &loaded_profiles[0],
                                                image_asset_store,
                                            )),
                                            ..default()
                                        })
                                        .id(),
                                );
                                let (text, _) = Self::profile_specific_texts(&loaded_profiles[0]);
                                text_nodes =
                                    Some(standard_centred_text(parent, text, font.clone()));
                            });
                    });
                parent
                    .spawn_bundle(crate::menu_core::nodes::horizontal::empty())
                    .with_children(|parent| {
                        make_button(ProfilePickerButton::Right, parent, font.clone());
                    });
            });
        ProfilePicker {
            text_nodes: text_nodes.unwrap(),
            current_index: 0,
            loaded_profiles,
            load_button_entity,
            image_entity: image_entity.unwrap(),
            initialised: false,
        }
    }

    fn get_current_slot(&self) -> &ProfileSlot {
        &self.loaded_profiles[self.current_index as usize]
    }

    fn change(
        &mut self,
        change: i32,
        text_query: &mut Query<&mut Text>,
        image_query: &mut Query<&mut UiImage>,
        image_asset_store: &ImageAssetStore,
    ) {
        self.current_index += change;
        if self.current_index >= self.loaded_profiles.len() as i32 {
            self.current_index = 0;
        } else if self.current_index < 0 {
            self.current_index = self.loaded_profiles.len() as i32 - 1
        }
        let (picker_text, button_text) =
            Self::profile_specific_texts(&(self.loaded_profiles[self.current_index as usize]));
        let mut text = text_query.get_mut(self.text_nodes.text).unwrap();
        text.sections[0].value = picker_text;

        let mut text = text_query.get_mut(self.load_button_entity).unwrap();
        text.sections[0].value = button_text;

        let mut ui_image = image_query.get_mut(self.image_entity).unwrap();
        ui_image.0 = Self::image_from_slot(
            &self.loaded_profiles[self.current_index as usize],
            image_asset_store,
        );
    }

    fn initialise(
        &mut self,
        text_query: &mut Query<&mut Text>,
        image_query: &mut Query<&mut UiImage>,
        image_asset_store: &ImageAssetStore,
    ) {
        self.change(0, text_query, image_query, image_asset_store);
        self.initialised = true;
    }
}
