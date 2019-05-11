use amethyst::prelude::*;
use amethyst::core::transform::Transform;
use amethyst::renderer::{
    Camera, Projection, PngFormat, SpriteSheetFormat, TextureMetadata, Texture,
    SpriteSheet, SpriteSheetHandle, SpriteRender,
};
use amethyst::assets::{AssetStorage, Loader};
use amethyst::ecs::prelude::{Component, VecStorage};

pub const LEVEL_WIDTH: f32 = 100.0;
pub const LEVEL_HEIGHT: f32 = 100.0;

pub struct LevelState {
}

impl LevelState {
    fn create_entities(world: &mut World, sprite_sheet: SpriteSheetHandle){

        // Create the player entity:
        let mut player_transform = Transform::default();
        player_transform.set_xyz(LEVEL_WIDTH / 2.0, LEVEL_HEIGHT / 2.0, 0.0);
        let player_sprite_render = SpriteRender{
            sprite_sheet: sprite_sheet.clone(),
            sprite_number: 0,
        };


        world
            .create_entity()
            .with(player_transform)
            .with(player_sprite_render)
            .with(Player{})
            .with(Weight{})
            .build();
    }

    fn load_sprite_sheet(world: &mut World) -> SpriteSheetHandle {
        let texture_handle = {
            let loader = world.read_resource::<Loader>();
            let texture_storage = 
                world.read_resource::<AssetStorage<Texture>>();
            loader.load(
                "texture/spritesheet.png",
                PngFormat,
                TextureMetadata::srgb_scale(),
                (),
                &texture_storage,
            )
        };

        let loader = world.read_resource::<Loader>();
        let sprite_sheet_store = world.read_resource::<AssetStorage<SpriteSheet>>();

        loader.load(
            "texture/spritesheet.ron",
            SpriteSheetFormat,
            texture_handle,
            (),
            &sprite_sheet_store,
        )
    }

    fn initialize_camera(world: &mut World) {
        let mut transform = Transform::default();
        transform.set_z(1.0);
        world
            .create_entity()
            .with(Camera::from(Projection::orthographic(
                0.0,
                LEVEL_WIDTH,
                0.0,
                LEVEL_HEIGHT,
            )))
            .with(transform)
            .build();
    }
}

impl SimpleState for LevelState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;

        let sprite_sheet_handle = LevelState::load_sprite_sheet(world);

        LevelState::create_entities(world, sprite_sheet_handle);
        LevelState::initialize_camera(world);
    }
}

pub struct Player {
}

impl Component for Player {
    type Storage = VecStorage<Self>;
}

pub struct Weight {
}

impl Component for Weight {
    type Storage = VecStorage<Self>;
}