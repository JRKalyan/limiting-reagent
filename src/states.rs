use amethyst::prelude::*;
use amethyst::core::transform::Transform;
use amethyst::renderer::{
    Camera, Projection, PngFormat, SpriteSheetFormat, TextureMetadata, Texture,
    SpriteSheet, SpriteSheetHandle, SpriteRender, Transparent,
};
use amethyst::assets::{AssetStorage, Loader};
use amethyst::ecs::prelude::{Component, VecStorage};

pub const LEVEL_WIDTH: f32 = 800.0;
pub const LEVEL_HEIGHT: f32 = 600.0;

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
        let player_animation = SpriteAnimation::new(2, 0, 2, 0.2);

        world
            .create_entity()
            .with(player_transform)
            .with(player_sprite_render)
            .with(player_animation)
            .with(Player{})
            .with(Mover::new())
            .with(Collider{width: 30.0, height: 30.0})
            .build();

        // create platform entity
        let mut platform_transform = Transform::default();
        platform_transform.set_xyz(50.0, 10.0, 0.0);
        let platform_sprite_render = SpriteRender{
            sprite_sheet: sprite_sheet.clone(),
            sprite_number: 1,
        };
        world
            .create_entity()
            .with(platform_sprite_render)
            .with(platform_transform)
            .with(Platform{x: 50.0, y: 10.0})
            .with(Collider{width: 700.0, height: 4.0})
            .with(Transparent)
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

pub enum JumpState {
    Landed,
    Airborne,
    Jump,
}

pub struct Mover {
    pub velocity_x: f32,
    pub velocity_y: f32,
    pub jump_state: JumpState,
}

impl Mover {
    fn new() -> Mover {
        Mover {
            velocity_x: 0.0,
            velocity_y: 0.0,
            jump_state: JumpState::Airborne,
        }
    }
}

impl Component for Mover {
    type Storage = VecStorage<Self>;
}

// TODO - move these x,y out and only
// alter transforms on second pass
pub struct Platform {
    pub x: f32,
    pub y: f32,
}

impl Component for Platform {
    type Storage = VecStorage<Self>;
}

// Rectangular collider to share
pub struct Collider {
    pub width: f32,
    pub height: f32,
}

impl Component for Collider {
    type Storage = VecStorage<Self>;
}

// Animation Component
pub struct SpriteAnimation {
    pub frame_count: usize,
    pub elapsed_time: f32,
    pub time_per_frame: f32,
    pub sprite_offset: usize,
    pub right_sprite_offset: usize, 
    pub left_sprite_offset: usize,
}

impl SpriteAnimation {
    fn new(left_sprite_offset: usize, right_sprite_offset: usize,
           frame_count: usize, time_per_frame: f32,) -> SpriteAnimation {
        SpriteAnimation {
            left_sprite_offset,
            right_sprite_offset,
            frame_count,
            time_per_frame,
            sprite_offset: right_sprite_offset,
            elapsed_time: 0.0,
        }
    }
}

impl Component for SpriteAnimation {
    type Storage = VecStorage<Self>;
}