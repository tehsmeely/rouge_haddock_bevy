use bevy::math::Vec2;
use strum_macros::EnumIter;

pub trait AssetClass {
    fn to_filename(&self) -> &str;
}

#[derive(EnumIter, PartialEq, Eq, Hash, Debug, Clone, Copy)]
pub enum ImageAsset {
    HaddockSpritesheet,
    HaddockSprite,
    WhaleSprite,
    SharkSpritesheet,
    CrabSpritesheet,
    ProjectileSpritesheet,
    TileMapSpritesheet,
    SnailSpritesheet,
    JellySpritesheet,
    JellyChargeIndicator,
    JellyLightning,
    VortexSprite,
    Background,
    HelpCard,
}

impl ImageAsset {
    pub fn is_for_tilemap(&self) -> bool {
        match self {
            Self::TileMapSpritesheet => true,
            _ => false,
        }
    }
}

#[derive(EnumIter, PartialEq, Eq, Hash, Debug, Clone, Copy)]
pub enum AudioAsset {
    JellyLightning,
    Hurt,
    Chomp,
    FishSlap,
    Death,
    Pickup,
}

impl AssetClass for AudioAsset {
    fn to_filename(&self) -> &str {
        match self {
            Self::JellyLightning => "audio/512471__michael-grinnell__electric-zap.quiet.wav",
            Self::Hurt => "audio/342229__christopherderp__hurt-1-male.wav",
            Self::Chomp => "audio/carrotnom.wav",
            Self::FishSlap => "audio/fish_slap.ogg",
            Self::Death => "audio/398068__happyparakeet__pixel-death.wav",
            Self::Pickup => "audio/608431__plasterbrain__shiny-coin-pickup.flac",
        }
    }
}

impl AssetClass for ImageAsset {
    fn to_filename(&self) -> &str {
        match self {
            Self::HaddockSpritesheet => "sprites/haddock_spritesheet.png",
            Self::HaddockSprite => "sprites/haddock.png",
            Self::WhaleSprite => "sprites/whale.png",
            Self::SharkSpritesheet => "sprites/shark_spritesheet.png",
            Self::CrabSpritesheet => "sprites/crab_spritesheet.png",
            Self::ProjectileSpritesheet => "sprites/projectile_spritesheet.png",
            Self::TileMapSpritesheet => "sprites/tilemap_spritesheet.png",
            Self::SnailSpritesheet => "sprites/prey_snail_spritesheet.png",
            Self::JellySpritesheet => "sprites/jelly_spritesheet.png",
            Self::VortexSprite => "sprites/vortex.png",
            Self::Background => "sprites/back.png",
            Self::JellyChargeIndicator => "sprites/jelly_charge_indicator.png",
            Self::JellyLightning => "sprites/jelly_lightning.png",
            Self::HelpCard => "ui/help_card.png",
        }
    }
}

#[derive(EnumIter, PartialEq, Eq, Hash, Debug, Clone, Copy)]
pub enum TextureAtlasAsset {
    HaddockSpritesheet,
    SharkSpritesheet,
    CrabSpritesheet,
    ProjectileSpritesheet,
    SnailSpritesheet,
    JellySpritesheet,
    JellyChargeIndicator,
    JellyLightning,
}

impl TextureAtlasAsset {
    pub fn to_image_asset(&self) -> ImageAsset {
        match self {
            Self::HaddockSpritesheet => ImageAsset::HaddockSpritesheet,
            Self::SharkSpritesheet => ImageAsset::SharkSpritesheet,
            Self::CrabSpritesheet => ImageAsset::CrabSpritesheet,
            Self::ProjectileSpritesheet => ImageAsset::ProjectileSpritesheet,
            Self::SnailSpritesheet => ImageAsset::SnailSpritesheet,
            Self::JellySpritesheet => ImageAsset::JellySpritesheet,
            Self::JellyChargeIndicator => ImageAsset::JellyChargeIndicator,
            Self::JellyLightning => ImageAsset::JellyLightning,
        }
    }

    pub fn frame_size(&self) -> Vec2 {
        let (x, y) = match self {
            Self::HaddockSpritesheet
            | Self::SharkSpritesheet
            | Self::CrabSpritesheet
            | Self::JellySpritesheet
            | Self::JellyChargeIndicator
            | Self::JellyLightning
            | Self::SnailSpritesheet => (64.0, 64.0),
            Self::ProjectileSpritesheet => (20.0, 20.0),
        };
        Vec2::new(x, y)
    }

    pub fn columns(&self) -> usize {
        match self {
            Self::HaddockSpritesheet => 5,
            Self::SharkSpritesheet
            | Self::CrabSpritesheet
            | Self::ProjectileSpritesheet
            | Self::JellySpritesheet
            | Self::JellyChargeIndicator
            | Self::SnailSpritesheet => 4,
            Self::JellyLightning => 3,
        }
    }
    pub fn rows(&self) -> usize {
        match self {
            Self::HaddockSpritesheet | Self::SharkSpritesheet | Self::ProjectileSpritesheet => 4,
            Self::CrabSpritesheet
            | Self::JellySpritesheet
            | Self::JellyChargeIndicator
            | Self::JellyLightning
            | Self::SnailSpritesheet => 1,
        }
    }
}
