use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use num::Integer;

use crate::game::components::TileType;
use crate::map_gen::cell_map::CellMap;

pub trait TilePosExt {
    fn add(&self, add: (i32, i32)) -> Self;

    // TODO: Call this "to_global_position" to ensure non-confusion with local transforms?
    fn to_world_pos(&self, z: f32) -> Vec3;
    fn from_world_pos(x: f32, y: f32) -> Self;

    fn as_vec2(&self) -> Vec2;
}
impl TilePosExt for TilePos {
    fn add(&self, add: (i32, i32)) -> Self {
        Self(
            helpers::add(self.0, add.0).unwrap(),
            helpers::add(self.1, add.1).unwrap(),
        )
    }

    fn as_vec2(&self) -> Vec2 {
        Vec2::new(self.0 as f32, self.1 as f32)
    }

    fn to_world_pos(&self, z: f32) -> Vec3 {
        // TODO: Support some "world_config" param to do cell size and 0,0 offset
        let x_offset = 0.0;
        let y_offset = 0.0;
        let centre_x_offset = 32.0;
        let centre_y_offset = 32.0;
        let x = self.0 as f32 * 64.0;
        let y = self.1 as f32 * 64.0;
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
            Self(x as u32, y as u32)
        } else {
            Self(0, 0)
        }
    }
}

#[derive(Debug, Component)]
pub struct HasTileType(pub TileType);

pub fn cleanup(mut commands: Commands, mut map_query: MapQuery) {
    map_query.despawn(&mut commands, 0);
}

pub fn init_tilemap(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    mut map_query: MapQuery,
    cell_map: &CellMap<i32>,
    border_size: usize,
) {
    let texture_handle = asset_server.load("sprites/tilemap_spritesheet.png");

    // Create map entity and component:
    let map_entity = commands.spawn().id();
    let mut map = Map::new(0u16, map_entity);

    let square_chunk_size = 8u32;
    //let map_tile_dims = (DEBUG_MAP[0].len() as u32, DEBUG_MAP.len() as u32);
    let map_tile_dims = {
        let rect_size = cell_map.rect_size();
        let w = rect_size.0 + 2 * border_size;
        let h = rect_size.1 + 2 * border_size;
        println!("Rect Size: {:?}, World Size: {:?}", rect_size, (w, h));
        (w as u32, h as u32)
    };
    let map_chunk_dims = (
        map_tile_dims.0.div_ceil(&square_chunk_size),
        map_tile_dims.1.div_ceil(&square_chunk_size),
    );

    // Creates a new layer builder with a layer entity.
    let (mut layer_builder, _): (LayerBuilder<TileBundle>, Entity) = LayerBuilder::new(
        commands,
        LayerSettings::new(
            MapSize(map_chunk_dims.0, map_chunk_dims.1),
            ChunkSize(square_chunk_size, square_chunk_size),
            TileSize(64.0, 64.0),
            TextureSize(128.0, 64.0),
        ),
        0u16,
        0u16,
    );

    for j in 0..map_tile_dims.1 {
        for i in 0..map_tile_dims.0 {
            let tile_type = match cell_map.contains(&(i as i32, j as i32)) {
                true => TileType::WATER,
                false => TileType::WALL,
            };
            //print!("{}", tile_type.to_str());
            print!("{:?}", (i, j));
            let pos = TilePos(i as u32, j as u32);
            layer_builder
                .set_tile(pos.clone(), tile_type.to_raw_tile().into())
                .unwrap();
            let tile_entity = layer_builder.get_tile_entity(commands, pos).unwrap();
            commands.entity(tile_entity).insert(HasTileType(tile_type));
        }
        println!();
    }

    // Builds the layer.
    // Note: Once this is called you can no longer edit the layer until a hard sync in bevy.
    let layer_entity = map_query.build_layer(commands, layer_builder, texture_handle);

    // Required to keep track of layers for a map internally.
    map.add_layer(commands, 0u16, layer_entity);

    // Spawn Map
    // Required in order to use map_query to retrieve layers/tiles.
    commands
        .entity(map_entity)
        .insert(map)
        .insert(Transform::from_xyz(0.0, 0.0, 0.0))
        .insert(GlobalTransform::default());
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
