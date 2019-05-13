extern crate rand;
use rand::Rng;

use amethyst::prelude::*;
use amethyst::core::transform::Transform;
use amethyst::renderer::{
    Camera, Projection, PngFormat, SpriteSheetFormat, TextureMetadata, Texture,
    SpriteSheet, SpriteSheetHandle, SpriteRender, Transparent,
};
use amethyst::assets::{AssetStorage, Loader};
use amethyst::ecs::prelude::{Component, VecStorage, Entity};
use amethyst::ui::{
    Anchor, TtfFormat, UiText, UiTransform,
};

pub const LEVEL_WIDTH: f32 = 800.0;
pub const LEVEL_HEIGHT: f32 = 450.0;
pub const CAMERA_WIDTH: f32 = 400.0;
pub const CAMERA_HEIGHT: f32 = 225.0;
pub const PLATFORM_HEIGHT: f32 = 25.0;
pub const PLATFORM_WIDTH: f32 = 100.0;
pub const RESOURCE_WIDTH: f32 = 28.0;
pub const RESOURCE_HEIGHT: f32 = 25.0;
pub const PLAYER_HEIGHT: f32 = 25.0;
pub const PLAYER_WIDTH: f32 = 28.0;

// UI:
// health
// score
// hornwort, mushroom count
pub struct UiEntities {
    pub score_entity: Entity,
    //pub hornwort_entity: Entity,
    //pub mushroom_entity: Entity,
    pub health_entity: Entity,
}

#[derive(Default)]
pub struct UiValues {
    pub score: i32,
    // TODO write to these from player system
    pub hornwort: i32,
    pub mushroom: i32,
    pub health: i32,
}

#[derive(Default)]
pub struct LevelState {
}

impl LevelState {
    fn create_entities(world: &mut World, sprite_sheet: SpriteSheetHandle){
        // TODO create the player on the first platform

        // Create the player entity:
        let mut player_transform = Transform::default();
        player_transform.set_xyz(LEVEL_WIDTH / 2.0, 50.0, 0.0);
        let player_sprite_render = SpriteRender{
            sprite_sheet: sprite_sheet.clone(),
            sprite_number: 0,
        };
        let player_animation = SpriteAnimation::new(0, 2, 0, 1, 0.2, 0);

        world
            .create_entity()
            .with(player_transform)
            .with(player_sprite_render)
            .with(player_animation)
            .with(Player::new())
            .with(Mover::new(0.0 + PLAYER_WIDTH / 2.0, LEVEL_WIDTH - PLAYER_WIDTH / 2.0))
            .with(Collider{width: 24.0, height: 25.0})
            .build();
        // create platform entities
        LevelState::generate_platforms(world, sprite_sheet);
    }

    fn generate_platforms(world: &mut World, sprite_sheet: SpriteSheetHandle) {
        let mut y = 2.0 * PLATFORM_HEIGHT;
        let mut x = LEVEL_WIDTH / 2.0;
        let mut rng = rand::thread_rng();
        while y < LEVEL_HEIGHT {
            // TODO restrict x by max and make that affect whether we spawn or just choose a side
            let mut platform_transform = Transform::default();
            platform_transform.set_xyz(x, y, 0.0);
            let platform_sprite_render = SpriteRender{
                sprite_sheet: sprite_sheet.clone(),
                sprite_number: 4,
            };
            world
                .create_entity()
                .with(platform_sprite_render)
                .with(platform_transform)
                .with(Platform{})
                .with(Collider{width: PLATFORM_WIDTH, height: PLATFORM_HEIGHT})
                .build();

            // generate resources on this platform:
            let min_x = x - PLATFORM_WIDTH / 2.0 + RESOURCE_WIDTH / 2.0;
            let max_x = x + PLATFORM_WIDTH / 2.0 - RESOURCE_WIDTH / 2.0;

            LevelState::generate_resources(world, sprite_sheet.clone(), 
                                           y + PLATFORM_HEIGHT, min_x, max_x);
            
            // spawn enemy
            let mut enemy_transform = Transform::default();
            let mut enemy_mover = Mover::new(min_x, max_x);
            enemy_mover.velocity_x = super::systems::enemy::ENEMY_VELOCITY;
            if rng.gen() {
                enemy_mover.velocity_x = enemy_mover.velocity_x * -1.0;
            }
            enemy_transform.set_xyz(x, y + 25.0, 0.0);
            world
                .create_entity()
                .with(enemy_transform)
                .with(Enemy{})
                .with(SpriteRender {
                    sprite_sheet: sprite_sheet.clone(),
                    sprite_number: 0,
                })
                .with(SpriteAnimation::new(0, 2, 0, 1, 0.2, 0))
                .with(enemy_mover)
                .with(Collider{width: PLAYER_WIDTH, height: PLAYER_HEIGHT})
                .build();
            
            // Setup next platform spawn
            y += PLATFORM_HEIGHT + PLAYER_HEIGHT; // min allow gap for player
            let left: bool = rng.gen();
            let mut random: f32 = rng.gen();
            random += 1.0;
            if left {
                random = random * -1.0;
            }
            let new_x = x + random * 90.0;
            if new_x < 0.0 + PLATFORM_WIDTH / 2.0 || new_x > LEVEL_WIDTH - PLATFORM_WIDTH / 2.0 {
                // transform to other direction
                random = random * -1.0;
            }
            x += random * 90.0;

            random = rng.gen();
            y += random * 45.0;

        }
    }

    fn generate_resources(world: &mut World, sprite_sheet: SpriteSheetHandle,
                          y: f32, x_min: f32, x_max: f32) {
        let mut rng = rand::thread_rng();
        let hornwort_count = rng.gen_range(1, 2);
        for _ in 0..hornwort_count {
            let mut transform = Transform::default();
            let x: f32 = rng.gen_range(x_min, x_max);
            transform.set_xyz(x, y, -1.0); // TODO make sure Z value is read
            world
                .create_entity()
                .with(transform)
                .with(Collider{width: RESOURCE_WIDTH, height: RESOURCE_HEIGHT})
                .with(SpriteRender {
                    sprite_sheet: sprite_sheet.clone(),
                    sprite_number: 5,
                })
                .with(Ingredient::Hornwort{count: 1})
                .build();
        }

        let mushroom_count = rng.gen_range(0, 2);
        for _ in 0..mushroom_count {
            let mut transform = Transform::default();
            let x: f32 = rng.gen_range(x_min, x_max);
            transform.set_xyz(x, y, -1.0); 
            world
                .create_entity()
                .with(transform)
                .with(Collider{width: RESOURCE_WIDTH, height: RESOURCE_HEIGHT})
                .with(SpriteRender {
                    sprite_sheet: sprite_sheet.clone(),
                    sprite_number: 6,
                })
                .with(Ingredient::Hornwort{count: 1})
                .build();
        }

    }

    fn initialize_ui(world: &mut World) {
        let font = world.read_resource::<Loader>().load(
            "font/square.ttf",
            TtfFormat,
            Default::default(),
            (),
            &world.read_resource(),
        );

        let score_transform = UiTransform::new(
            "score".to_string(), Anchor::TopRight,
            -100.0, -50.0, 1.0, 200.0, 50.0, 0
        );

        let score_entity = world
            .create_entity()
            .with(score_transform)
            .with(UiText::new(
                font.clone(),
                "SCORE: 0".to_string(),
                [0.0, 0.0, 0.0, 1.0],
                50.0,
            ))
            .build();
        
        // TODO display health as a bar instead
        let health_transform = UiTransform::new(
            "health".to_string(), Anchor::BottomMiddle,
            0.0, 50.0, 1.0, 400.0, 50.0, 0
        );

        let health_entity = world
            .create_entity()
            .with(health_transform)
            .with(UiText::new(
                font.clone(),
                "HEALTH: 100".to_string(),
                [0.0, 0.0, 0.0, 1.0],
                50.0,
            ))
            .build();
        
        world.add_resource(UiEntities{score_entity, health_entity});
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
                CAMERA_WIDTH,
                0.0,
                CAMERA_HEIGHT,
            )))
            .with(transform)
            .build();
    }
}

impl SimpleState for LevelState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;

        let sprite_sheet_handle = LevelState::load_sprite_sheet(world);

        LevelState::initialize_ui(world);
        LevelState::create_entities(world, sprite_sheet_handle);
        LevelState::initialize_camera(world);
    }
}

// COMPONENTS
// ----------

pub struct Player {
    pub in_hit: bool, // in hit state until hit_last
    pub last_hit: f32,
    pub last_throw: f32,
    pub hit_cooldown: f32, // frequently you can get hit
    pub hit_last: f32,
    pub throw_cooldown: f32,
    pub health: i32,

    // Enum map would be nice for this purpose
    pub hornwort: usize,
    pub mushroom: usize,
}

impl Player {
    pub fn new() -> Player {
        Player {
            in_hit: false,
            last_hit: 0.0,
            last_throw: 0.0,
            health: 100,
            hit_cooldown: 1.0,
            hit_last: 0.2,
            throw_cooldown: 0.5,
            hornwort: 0,
            mushroom: 0,
        }
    }
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
    pub min_x: f32,
    pub max_x: f32,
}

impl Mover {
    fn new(min_x: f32, max_x: f32) -> Mover {
        // TODO change constructor to accep tarugments
        Mover {
            velocity_x: 0.0,
            velocity_y: 0.0,
            jump_state: JumpState::Airborne,
            min_x,
            max_x,
        }
    }
}

impl Component for Mover {
    type Storage = VecStorage<Self>;
}

pub struct Platform {
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
    pub elapsed_time: f32,
    pub time_per_frame: f32,
    pub move_offset: usize, 
    pub move_count: usize,
    pub idle_offset: usize,
    pub idle_count: usize,
    pub airborne_offset: usize,
}

impl SpriteAnimation {
    fn new(move_offset: usize, move_count: usize, idle_offset: usize,
           idle_count: usize, time_per_frame: f32, airborne_offset: usize) -> SpriteAnimation {
        SpriteAnimation {
            move_count,
            move_offset,
            idle_count,
            idle_offset,
            time_per_frame,
            airborne_offset,
            elapsed_time: 0.0,
        }
    }
}

impl Component for SpriteAnimation {
    type Storage = VecStorage<Self>;
}

pub enum Ingredient {
    Hornwort{count: usize},
    Mushroom{count: usize},
}

impl Component for Ingredient {
    type Storage = VecStorage<Self>;
}

// TODO - if within range of the player move towards the player but not past limits
pub struct Enemy {
}

impl Component for Enemy {
    type Storage = VecStorage<Self>;
}
