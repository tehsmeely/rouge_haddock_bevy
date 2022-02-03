use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

pub struct GameTilemapPlugin;

impl Plugin for GameTilemapPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(crate::helpers::texture::set_texture_filters_to_nearest)
            .add_startup_system(init_tilemap);
    }
}
fn init_tilemap(mut commands: Commands, asset_server: Res<AssetServer>, mut map_query: MapQuery) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    let texture_handle = asset_server.load("sprites/tilemap_spritesheet.png");

    // Create map entity and component:
    let map_entity = commands.spawn().id();
    let mut map = Map::new(0u16, map_entity);

    // Creates a new layer builder with a layer entity.
    let (mut layer_builder, _) = LayerBuilder::new(
        &mut commands,
        LayerSettings::new(
            MapSize(2, 2),
            ChunkSize(8, 8),
            TileSize(64.0, 64.0),
            TextureSize(128.0, 64.0),
        ),
        0u16,
        0u16,
    );

    let water_tile = TileBundle {
        tile: Tile {
            texture_index: 0,
            ..Default::default()
        },
        ..Default::default()
    };
    let stone_tile = TileBundle {
        tile: Tile {
            texture_index: 1,
            ..Default::default()
        },
        ..Default::default()
    };
    layer_builder.set_all(stone_tile);
    for i in 3..8 {
        for j in 4..10 {
            layer_builder.set_tile(TilePos(i, j), water_tile.clone());
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
        .insert(Transform::from_xyz(-128.0, -128.0, 0.0))
        .insert(GlobalTransform::default());
}
