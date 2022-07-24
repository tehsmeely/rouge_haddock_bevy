use crate::asset_handling::asset::{ImageAsset, TextureAtlasAsset};
use serde::{Deserialize, Serialize};



const MAX_SAVES: usize = 4;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum HaddockVariant {
    Normal,
    Whale,
}

impl HaddockVariant {
    pub fn to_image_asset(&self) -> ImageAsset {
        match self {
            Self::Normal => ImageAsset::HaddockSprite,
            Self::Whale => ImageAsset::WhaleSprite,
        }
    }
    pub fn to_texture_atlas_asset(&self) -> TextureAtlasAsset {
        match self {
            Self::Normal | Self::Whale => TextureAtlasAsset::HaddockSpritesheet,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserProfile {
    pub snail_shells: usize,
    pub level: usize,
    pub name: String,
    pub haddock_variant: HaddockVariant,
}

impl Default for UserProfile {
    fn default() -> Self {
        Self {
            snail_shells: 0,
            level: 0,
            name: "Default".to_string(),
            haddock_variant: HaddockVariant::Normal,
        }
    }
}

impl UserProfile {
    pub fn max_power_charges(&self) -> usize {
        match self.level {
            0..=4 => 3,
            5..=9 => 4,
            10..=14 => 5,
            15..=20 => 6,
            _ => 7,
        }
    }
    pub fn max_health(&self) -> usize {
        match self.level {
            0..=2 => 3,
            3..=7 => 4,
            8..=14 => 5,
            15..=19 => 6,
            _ => 7,
        }
    }
}

// Resource for creating new slots which defines the slot to be loaded into
pub struct LoadingProfileSlotNum(pub usize);

#[derive(Debug, Clone)]
pub struct LoadedUserProfile {
    pub user_profile: UserProfile,
    file_index: usize,
}

impl LoadedUserProfile {
    pub fn save(&self) {
        platform_fs::save(self.file_index, &self.user_profile);
    }

    pub fn new(user_profile: UserProfile, file_index: usize) -> Self {
        Self {
            user_profile,
            file_index,
        }
    }
}

#[derive(Debug)]
pub enum ProfileSlot {
    Loaded(LoadedUserProfile),
    Free(usize),
}

fn save_id_of_index(index: usize) -> String {
    format!("save_{:02}.ron", index)
}

fn filename_of_index(index: usize) -> String {
    format!("saves/{}", save_id_of_index(index))
}

pub fn load_profiles_blocking() -> Vec<ProfileSlot> {
    let mut loaded_saves = Vec::new();
    for file_index in 0..MAX_SAVES {
        if let Some(profile) = platform_fs::maybe_load(file_index) {
            loaded_saves.push(ProfileSlot::Loaded(profile));
        } else {
            loaded_saves.push(ProfileSlot::Free(file_index));
        }
    }
    loaded_saves
}

#[cfg(not(target_arch = "wasm32"))]
mod platform_fs {
    use crate::profiles::profiles::{filename_of_index, LoadedUserProfile, UserProfile};
    use std::fs::File;
    use std::io::{BufReader, BufWriter};

    pub fn maybe_load(index: usize) -> Option<LoadedUserProfile> {
        let filename = filename_of_index(index);
        let file = File::open(filename).ok()?;
        let reader = BufReader::new(file);
        let user_profile = ron::de::from_reader(reader).ok()?;
        Some(LoadedUserProfile {
            user_profile,
            file_index: index,
        })
    }

    pub fn save(index: usize, user_profile: &UserProfile) {
        let filename = filename_of_index(index);
        let file = File::create(filename).unwrap();
        let writer = BufWriter::new(file);
        ron::ser::to_writer(writer, user_profile).unwrap();
    }
}

#[cfg(target_arch = "wasm32")]
mod platform_fs {
    use crate::profiles::profiles::{save_id_of_index, LoadedUserProfile, UserProfile};

    pub fn maybe_load(index: usize) -> Option<LoadedUserProfile> {
        let window: web_sys::Window = web_sys::window()?;
        let local_storage: web_sys::Storage = window.local_storage().ok()??;
        let save_id = save_id_of_index(index);
        let profile_entry = local_storage.get_item(&save_id).ok()??;
        let user_profile = ron::de::from_str(&profile_entry).ok()?;
        Some(LoadedUserProfile {
            user_profile,
            file_index: index,
        })
    }

    pub fn save(index: usize, user_profile: &UserProfile) {
        let window: web_sys::Window = web_sys::window().unwrap();
        let local_storage: web_sys::Storage = window.local_storage().unwrap().unwrap();
        let save_id = save_id_of_index(index);
        let user_profile_ron = ron::ser::to_string(user_profile).unwrap();
        local_storage.set_item(&save_id, &user_profile_ron).unwrap();
    }
}
