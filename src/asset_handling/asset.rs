use strum_macros::EnumIter;

pub trait AssetClass {
    fn to_filename(&self) -> &str;
}

#[derive(EnumIter, PartialEq, Eq, Hash, Debug, Clone, Copy)]
pub enum ImageAsset {
    HaddockSpritesheet,
    SharkSpritesheet,
    CrabSpritesheet,
    ProjectileSpritesheet,
}

#[derive(EnumIter, PartialEq, Eq, Hash, Debug, Clone, Copy)]
pub enum AudioAsset {}

impl AssetClass for ImageAsset {
    fn to_filename(&self) -> &str {
        match self {
            Self::HaddockSpritesheet => "sprites/haddock_spritesheet.png",
            Self::SharkSpritesheet => "sprites/shark_spritesheet.png",
            Self::CrabSpritesheet => "sprites/crab_spritesheet.png",
            Self::ProjectileSpritesheet => "sprites/projectile_spritesheet.png",
        }
    }
}
