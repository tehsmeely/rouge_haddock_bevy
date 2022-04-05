pub mod asset;
mod loading;

pub use loading::LoadingPlugin as Plugin;
pub use loading::{ImageAssetStore, TextureAtlasStore};
