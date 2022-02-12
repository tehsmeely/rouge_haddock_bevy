use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use num::Integer;

pub struct GameTilemapPlugin;

pub trait TilePosExt {
    fn add(&self, add: (i32, i32)) -> Self;

    // TODO: Call this "to_global_position" to ensure non-confusion with local transforms?
    fn to_world_pos(&self, z: f32) -> Vec3;
    fn from_world_pos(x: f32, y: f32) -> Self;
}
impl TilePosExt for TilePos {
    fn add(&self, add: (i32, i32)) -> Self {
        Self(
            helpers::add(self.0, add.0).unwrap(),
            helpers::add(self.1, add.1).unwrap(),
        )
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

#[derive(Debug)]
pub enum TileType {
    WALL,
    WATER,
}

impl TileType {
    pub fn can_enter(&self) -> bool {
        match self {
            Self::WALL => false,
            Self::WATER => true,
        }
    }

    fn to_raw_tile(&self) -> Tile {
        match self {
            Self::WATER => Tile {
                texture_index: 0,
                ..Default::default()
            },
            Self::WALL => Tile {
                texture_index: 1,
                ..Default::default()
            },
        }
    }
}

#[derive(Debug, Component)]
pub struct HasTileType(pub TileType);

impl Plugin for GameTilemapPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(crate::helpers::texture::set_texture_filters_to_nearest)
            .add_startup_system(init_tilemap);
    }
}
fn init_tilemap(mut commands: Commands, asset_server: Res<AssetServer>, mut map_query: MapQuery) {
    let texture_handle = asset_server.load("sprites/tilemap_spritesheet.png");

    // Create map entity and component:
    let map_entity = commands.spawn().id();
    let mut map = Map::new(0u16, map_entity);

    let square_chunk_size = 8u32;
    let map_tile_dims = (DEBUG_MAP[0].len() as u32, DEBUG_MAP.len() as u32);
    let map_chunk_dims = (
        map_tile_dims.0.div_ceil(&square_chunk_size),
        map_tile_dims.1.div_ceil(&square_chunk_size),
    );

    // Creates a new layer builder with a layer entity.
    let (mut layer_builder, _): (LayerBuilder<TileBundle>, Entity) = LayerBuilder::new(
        &mut commands,
        LayerSettings::new(
            MapSize(map_chunk_dims.0, map_chunk_dims.1),
            ChunkSize(square_chunk_size, square_chunk_size),
            TileSize(64.0, 64.0),
            TextureSize(128.0, 64.0),
        ),
        0u16,
        0u16,
    );
    for (j, row) in DEBUG_MAP.iter().enumerate() {
        for (i, c) in row.chars().enumerate() {
            let pos = TilePos(i as u32, j as u32);
            let tile_type = match c {
                'W' => TileType::WALL,
                ' ' => TileType::WATER,
                _ => TileType::WATER,
            };
            layer_builder
                .set_tile(pos.clone(), tile_type.to_raw_tile().into())
                .unwrap();
            let tile_entity = layer_builder.get_tile_entity(&mut commands, pos).unwrap();
            commands.entity(tile_entity).insert(HasTileType(tile_type));
        }
    }

    // Builds the layer.
    // Note: Once this is called you can no longer edit the layer until a hard sync in bevy.
    let layer_entity = map_query.build_layer(&mut commands, layer_builder, texture_handle);

    // Required to keep track of layers for a map internally.
    map.add_layer(&mut commands, 0u16, layer_entity);

    // Spawn Map
    // Required in order to use map_query to retrieve layers/tiles.
    commands
        .entity(map_entity)
        .insert(map)
        .insert(Transform::from_xyz(0.0, 0.0, 0.0))
        .insert(GlobalTransform::default());
}

const DEBUG_MAP: [&str; 18] = [
    "WWWWWWWWWWWWWWWWW",
    "WWWF          EFW",
    "WWW  E        WWW",
    "WE        S     W",
    "W    F         FW",
    "WWWWWW       WW W",
    "W F  W        W W",
    "W    W        WFW",
    "WWW          WW W",
    "WFW             W",
    "W W       E     W",
    "W W             W",
    "W W       E     W",
    "W W E           W",
    "W W       E     W",
    "W W             W",
    "W               W",
    "WWWWWWWWWWWWWWWWW",
];

mod helpers {
    pub fn add(u: u32, i: i32) -> Option<u32> {
        if i.is_negative() {
            u.checked_sub(i.wrapping_abs() as u32)
        } else {
            u.checked_add(i as u32)
        }
    }
}
