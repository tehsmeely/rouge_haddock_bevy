use crate::asset_handling::asset::{ImageAsset, TextureAtlasAsset};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, BufWriter};

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
    fn to_filename(&self) -> String {
        filename_of_index(self.file_index)
    }

    pub fn save(&self) {
        let filename = self.to_filename();
        let file = File::create(filename).unwrap();
        let writer = BufWriter::new(file);
        ron::ser::to_writer(writer, &self.user_profile).unwrap();
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

fn filename_of_index(index: usize) -> String {
    format!("saves/save_{:02}.ron", index)
}

pub fn load_profiles_blocking() -> Vec<ProfileSlot> {
    let mut loaded_saves = Vec::new();
    for file_index in 0..MAX_SAVES {
        let filename = filename_of_index(file_index);
        let loaded_profile = if let Ok(file) = File::open(filename) {
            let reader = BufReader::new(file);
            if let Ok(user_profile) = ron::de::from_reader(reader) {
                Some(LoadedUserProfile {
                    user_profile,
                    file_index,
                })
            } else {
                None
            }
        } else {
            None
        };

        if let Some(profile) = loaded_profile {
            loaded_saves.push(ProfileSlot::Loaded(profile));
        } else {
            loaded_saves.push(ProfileSlot::Free(file_index));
        }
    }
    loaded_saves
}
