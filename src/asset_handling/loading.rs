use super::asset::{AssetClass, ImageAsset};
use crate::CoreState;
use bevy::asset::LoadState;
use bevy::prelude::*;
use std::collections::HashMap;
use strum::IntoEnumIterator;

pub struct LoadingPlugin;

pub type ImageAssetStore = HashMap<ImageAsset, Handle<Image>>;

struct LoadingPrintTimer(Timer);

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        let state = CoreState::Loading;
        app.insert_resource(LoadingPrintTimer(Timer::from_seconds(1.0, true)))
            .add_system_set(SystemSet::on_enter(state).with_system(load_all))
            .add_system_set(SystemSet::on_update(state).with_system(loading_watcher));
    }
}

fn load_all(asset_server: Res<AssetServer>, mut commands: Commands) {
    let mut image_handles: ImageAssetStore = HashMap::new();
    for asset in ImageAsset::iter() {
        let handle = asset_server.load(asset.to_filename());
        image_handles.insert(asset, handle);
    }
    commands.insert_resource(image_handles);
}

fn loading_watcher(
    asset_server: Res<AssetServer>,
    image_handles: Res<ImageAssetStore>,
    mut state: ResMut<State<CoreState>>,
    mut print_timer: ResMut<LoadingPrintTimer>,
    time: Res<Time>,
) {
    let mut count = LoadStateCount::default();
    for handle in image_handles.values() {
        let load_state = asset_server.get_load_state(handle);
        count.incr(&load_state);
    }
    print_timer.0.tick(time.delta());
    if print_timer.0.just_finished() {
        info!("Loading. Progress: {:?}", count);
    }

    if count.all_finished() {
        info!("Finished Loading: {:?}", count);
        state.set(CoreState::MainMenu).unwrap();
    }
}

#[derive(Default, Debug)]
struct LoadStateCount {
    not_loaded: usize,
    loading: usize,
    loaded: usize,
    failed: usize,
    unloaded: usize,
}

impl LoadStateCount {
    fn incr(&mut self, load_state: &LoadState) {
        match load_state {
            LoadState::NotLoaded => self.not_loaded += 1,
            LoadState::Loading => self.loading += 1,
            LoadState::Loaded => self.loaded += 1,
            LoadState::Failed => self.loaded += 1,
            LoadState::Unloaded => self.unloaded += 1,
        }
    }

    fn all_finished(&self) -> bool {
        self.not_loaded == 0 && self.loading == 0
    }
}
