use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use log::info;
use num::Integer;

use crate::asset_handling::asset::ImageAsset;
use crate::asset_handling::ImageAssetStore;
use crate::game::components::TileType;
use crate::map_gen::cell_map::CellMap;

pub type TileStorageQuery<'w, 's> = Query<'w, 's, &'static TileStorage, ()>;

#[derive(Component)]
pub struct TileMapOnly;

pub trait TilePosExt {
    fn add(&self, add: (i32, i32)) -> Self;

    ///non-euclidean distance between two tileposes
    fn distance_to(&self, other: &Self) -> usize;

    // TODO: Call this "to_global_position" to ensure non-confusion with local transforms?
    fn to_world_pos(&self, z: f32) -> Vec3;
    fn from_world_pos(x: f32, y: f32) -> Self;

    fn as_vec2(&self) -> Vec2;
    fn as_i32s(&self) -> (i32, i32);
}
impl TilePosExt for TilePos {
    fn add(&self, add: (i32, i32)) -> Self {
        Self {
            x: helpers::add(self.x, add.0).unwrap(),
            y: helpers::add(self.y, add.1).unwrap(),
        }
    }

    fn distance_to(&self, other: &Self) -> usize {
        let dist = self.x.abs_diff(other.x) + self.y.abs_diff(other.y);
        dist as usize
    }
    fn to_world_pos(&self, z: f32) -> Vec3 {
        // TODO: Support some "world_config" param to do cell size and 0,0 offset
        let x_offset = 0.0;
        let y_offset = 0.0;
        let centre_x_offset = 32.0;
        let centre_y_offset = 32.0;
        let x = self.x as f32 * 64.0;
        let y = self.y as f32 * 64.0;
        Vec3::new(
            x + x_offset + centre_x_offset,
            y + y_offset + centre_y_offset,
            z,
        )
    }

    fn from_world_pos(x: f32, y: f32) -> Self {
        //Anything inside the tile width/height  counts as the tile
        // TODO: As with [to_world_pos], support some world_config param
        let x_offset = 0.0;
        let y_offset = 0.0;
        let x_size = 64.0;
        let y_size = 64.0;

        let x = (x - x_offset).div_euclid(x_size);
        let y = (y - y_offset).div_euclid(y_size);
        if x >= 0.0 && y >= 0.0 {
            Self {
                x: x as u32,
                y: y as u32,
            }
        } else {
            Self { x: 0, y: 0 }
        }
    }

    fn as_vec2(&self) -> Vec2 {
        Vec2::new(self.x as f32, self.y as f32)
    }

    fn as_i32s(&self) -> (i32, i32) {
        (self.x as i32, self.y as i32)
    }
}

#[derive(Debug, Component)]
pub struct HasTileType(pub TileType);

pub fn cleanup(mut commands: Commands, tilemap_only_query: Query<Entity, With<TileMapOnly>>) {
    for entity in tilemap_only_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn init_tilemap(
    commands: &mut Commands,
    image_assets: &Res<ImageAssetStore>,
    cell_map: &CellMap<i32>,
    border_size: usize,
    images: &Assets<Image>,
) {
    let texture_handle = image_assets.get(&ImageAsset::TileMapSpritesheet);

    info!("Tilemap Init!");
    info!("Texture: {:?}", images.get(&texture_handle));

    // Create map entity and component:
    let map_entity = commands.spawn().id();

    let square_chunk_size = 8u32;
    let map_tile_dims = {
        let rect_size = cell_map.rect_size();
        let w = rect_size.0 + 2 * border_size;
        let h = rect_size.1 + 2 * border_size;
        println!("Rect Size: {:?}, World Size: {:?}", rect_size, (w, h));
        (w as u32, h as u32)
    };

    // Map chunk dims should be the map tile rounded up to the nearest multiple of square_chunk size
    let map_chunk_dims = (
        map_tile_dims.0.div_ceil(&square_chunk_size),
        map_tile_dims.1.div_ceil(&square_chunk_size),
    );

    let _tilemap_size = TilemapSize {
        x: map_chunk_dims.0,
        y: map_chunk_dims.1,
    };
    let tilemap_size = TilemapSize {
        x: map_tile_dims.0,
        y: map_tile_dims.1,
    };

    let grid_size = TilemapGridSize {
        x: square_chunk_size as f32,
        y: square_chunk_size as f32,
    };
    let mut tile_storage = TileStorage::empty(tilemap_size);
    let tilemap_entity = commands.spawn().id();

    println!(
        "Map_tile_dims: {:?}\nMap_chunk_dims: {:?}",
        map_tile_dims, map_chunk_dims
    );

    for j in 0..map_tile_dims.1 {
        for i in 0..map_tile_dims.0 {
            let tile_type = match cell_map.contains(&(i as i32, j as i32)) {
                true => TileType::WATER,
                false => TileType::WALL,
            };
            let tile_pos = TilePos {
                x: i as u32,
                y: j as u32,
            };
            let tile_entity = commands
                .spawn_bundle(TileBundle {
                    position: tile_pos,
                    texture: tile_type.to_raw_tile(),
                    tilemap_id: TilemapId(tilemap_entity.clone()),
                    ..Default::default()
                })
                .insert(HasTileType(tile_type))
                .insert(TileMapOnly)
                .id();
            tile_storage.set(&tile_pos, Some(tile_entity));

            /*
            let tile_type = match cell_map.contains(&(i as i32, j as i32)) {
                true => TileType::WATER,
                false => TileType::WALL,
            };
            let pos = TilePos(i as u32, j as u32);
            layer_builder
                .set_tile(pos, tile_type.to_raw_tile().into())
                .unwrap();
            let tile_entity = layer_builder.get_tile_entity(commands, pos).unwrap();
            commands.entity(tile_entity).insert(HasTileType(tile_type));
             */
        }
    }

    commands
        .entity(tilemap_entity)
        .insert_bundle(TilemapBundle {
            grid_size,
            size: tilemap_size,
            storage: tile_storage,
            texture: TilemapTexture(texture_handle),
            tile_size: TilemapTileSize { x: 64.0, y: 64.0 },
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..Default::default()
        })
        .insert(TileMapOnly);

    // Builds the layer.
    // Note: Once this is called you can no longer edit the layer until a hard sync in bevy.
    //let layer_entity = map_query.build_layer(commands, layer_builder, texture_handle);

    // Required to keep track of layers for a map internally.
    //map.add_layer(commands, 0u16, layer_entity);

    // Spawn Map
    // Required in order to use map_query to retrieve layers/tiles.
    /*
    commands
        .entity(map_entity)
        .insert(map)
        .insert(Transform::from_xyz(0.0, 0.0, 0.0))
        .insert(GlobalTransform::default());
     */
}

mod helpers {
    pub fn add(u: u32, i: i32) -> Option<u32> {
        if i.is_negative() {
            u.checked_sub(i.wrapping_abs() as u32)
        } else {
            u.checked_add(i as u32)
        }
    }
}
