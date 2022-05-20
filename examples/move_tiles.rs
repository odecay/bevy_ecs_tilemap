use bevy::prelude::*;
use bevy_ecs_tilemap::{
    map::{
        Tilemap2dGridSize, Tilemap2dSize, Tilemap2dTextureSize, Tilemap2dTileSize, TilemapId,
        TilemapTexture,
    },
    tiles::{Tile2dStorage, TileBundle, TilePos2d},
    Tilemap2dPlugin, TilemapBundle,
};
use rand::{thread_rng, Rng};

mod helpers;

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    let texture_handle: Handle<Image> = asset_server.load("tiles.png");

    let tilemap_size = Tilemap2dSize { x: 4, y: 8 };
    let mut tile_storage = Tile2dStorage::empty(tilemap_size);
    let tilemap_entity = commands.spawn().id();

    for x in 0..4u32 {
        for y in 7..8u32 {
            let tile_pos = TilePos2d { x, y };
            let tile_entity = commands
                .spawn()
                .insert_bundle(TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    ..Default::default()
                })
                .id();
            tile_storage.set(&tile_pos, Some(tile_entity));
        }
    }

    let tile_size = Tilemap2dTileSize { x: 16.0, y: 16.0 };

    commands
        .entity(tilemap_entity)
        .insert_bundle(TilemapBundle {
            grid_size: Tilemap2dGridSize { x: 16.0, y: 16.0 },
            size: tilemap_size,
            storage: tile_storage,
            texture_size: Tilemap2dTextureSize { x: 96.0, y: 16.0 },
            texture: TilemapTexture(texture_handle),
            tile_size,
            transform: bevy_ecs_tilemap::helpers::get_centered_transform_2d(
                &tilemap_size,
                &tile_size,
                0.0,
            ),
            ..Default::default()
        })
        .insert(LastUpdate::default());
}

#[derive(Default, Component)]
struct LastUpdate {
    value: f64,
}

fn remove_tiles(
    mut commands: Commands,
    time: Res<Time>,
    mut last_update_query: Query<(&mut LastUpdate, &mut Tile2dStorage)>,
) {
    let current_time = time.seconds_since_startup();
    for (mut last_update, mut tile_storage) in last_update_query.iter_mut() {
        // Remove a tile every half second.
        if (current_time - last_update.value) > 0.1 {
            let mut random = thread_rng();
            let position = TilePos2d {
                x: random.gen_range(0..4),
                y: random.gen_range(0..8),
            };

            if let Some(tile_entity) = tile_storage.get(&position) {
                commands.entity(tile_entity).despawn_recursive();
                // Don't forget to remove tiles from the tile storage!
                tile_storage.set(&position, None);
            }

            last_update.value = current_time;
        }
    }
}

fn gravity(
    mut query: Query<(&mut TilePos2d)>,
    size: Query<&Tilemap2dSize>,
    time: Res<Time>,
    mut last_update_query: Query<(&mut LastUpdate, &mut Tile2dStorage)>,
) {
    let bound = size.single();
    let current_time = time.seconds_since_startup();

    for (mut last_update, mut tile_storage) in last_update_query.iter_mut() {
        if (current_time - last_update.value) > 0.1 {
            for x in 0..bound.x {
                for y in 0..bound.y {
                    let tile_pos = TilePos2d { x: x, y: y };
                    if tile_pos.y != 0 {
                        let below_pos = TilePos2d { x: x, y: y - 1 };
                        if let None = tile_storage.get(&below_pos) {
                            if let Some(tile_entity) = tile_storage.get(&tile_pos) {
                                if let Ok(mut entity_tile_pos) =
                                    query.get_component_mut::<TilePos2d>(tile_entity)
                                {
                                    tile_storage.set(&below_pos, Some(tile_entity));
                                    tile_storage.set(&tile_pos, None);
                                    entity_tile_pos.y -= 1u32;
                                }
                            }
                        }
                    }
                }
            }
            last_update.value = current_time;
        }
    }
}

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            width: 1270.0,
            height: 720.0,
            title: String::from("Remove Tiles Example"),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(Tilemap2dPlugin)
        .add_startup_system(startup)
        .add_system(helpers::camera::movement)
        .add_system(helpers::texture::set_texture_filters_to_nearest)
        // .add_system(remove_tiles)
        .add_system(gravity)
        .run();
}
