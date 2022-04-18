use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, BufWriter};

const MAX_SAVES: usize = 4;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserProfile {
    pub eggs: usize,
    pub level: usize,
    pub name: String,
}

impl Default for UserProfile {
    fn default() -> Self {
        Self {
            eggs: 0,
            level: 0,
            name: "Default".to_string(),
        }
    }
}

pub struct ActiveProfile(UserProfile);

#[derive(Debug)]
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

fn filename_of_index(index: usize) -> String {
    format!("saves/save_{:02}.ron", index)
}

pub fn load_profiles_blocking() -> Vec<LoadedUserProfile> {
    let mut loaded_saves = Vec::new();
    for file_index in 0..MAX_SAVES {
        let filename = filename_of_index(file_index);
        if let Ok(file) = File::open(filename) {
            let reader = BufReader::new(file);
            if let Ok(user_profile) = ron::de::from_reader(reader) {
                loaded_saves.push(LoadedUserProfile {
                    user_profile,
                    file_index,
                })
            }
        }
    }
    loaded_saves
}
