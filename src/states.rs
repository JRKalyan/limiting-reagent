extern crate rand;
use rand::Rng;

use amethyst::prelude::*;
use amethyst::core::transform::Transform;
use amethyst::core::nalgebra::Vector3;
use amethyst::renderer::{
    Camera, Projection, PngFormat, SpriteSheetFormat, TextureMetadata, Texture,
    SpriteSheet, SpriteSheetHandle, SpriteRender, Transparent, ScreenDimensions
};
use amethyst::assets::{AssetStorage, Loader};
use amethyst::ecs::prelude::{Component, VecStorage, Entity};
use amethyst::ui::{
    Anchor, TtfFormat, UiText, UiTransform, UiImage,
};
use amethyst::input::is_key_down;
use amethyst::Trans::*;

pub const LEVEL_WIDTH: f32 = 800.0;
pub const LEVEL_HEIGHT: f32 = 3000.0;
pub const CAMERA_WIDTH: f32 = 400.0;
pub const CAMERA_HEIGHT: f32 = 225.0;
pub const PLATFORM_HEIGHT: f32 = 25.0;
pub const PLATFORM_WIDTH: f32 = 100.0;
pub const RESOURCE_WIDTH: f32 = 28.0;
pub const RESOURCE_HEIGHT: f32 = 25.0;
pub const PLAYER_HEIGHT: f32 = 25.0;
pub const PLAYER_WIDTH: f32 = 28.0;

pub const POTION_SPEED: f32 = 200.0;

// UI:
pub struct UiEntities {
    pub score_entity: Entity,
    pub hornwort_entity: Entity,
    pub mushroom_entity: Entity,
    pub health_entity: Entity,
    pub game_over_entity: Entity,
}

#[derive(Default)]
pub struct UiValues {
    pub score: i32,
    // TODO write to these from player system
    pub health: i32,
}

// implement a lazy spawner for simplicity..
#[derive(Default, Debug)]
pub struct PotionInfo {
    pub px: f32,
    pub py: f32,
    pub mx: f32,
    pub my: f32,
}

pub struct PotionSpawner {
    pub potion: Option<PotionInfo>,
}

impl Default for PotionSpawner {
    fn default() -> PotionSpawner {
        PotionSpawner {
            potion: std::option::Option::None,
        }
    }
}

#[derive(Default)]
pub struct LevelState {
    pub sprite_sheet: Option<SpriteSheetHandle>,
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
                .with(Ingredient::Mushroom{count: 1})
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


        let m_texture = {
            let loader = world.read_resource::<Loader>();
            let texture_storage = 
                world.read_resource::<AssetStorage<Texture>>();

            loader.load(
                "texture/mushroom.png",
                PngFormat,
                TextureMetadata::srgb_scale(),
                (),
                &texture_storage
            )
        };

        let h_texture = {
            let loader = world.read_resource::<Loader>();
            let texture_storage = 
                world.read_resource::<AssetStorage<Texture>>();

            loader.load(
                "texture/plant.png",
                PngFormat,
                TextureMetadata::srgb_scale(),
                (),
                &texture_storage
            )
        };

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

        let mi_transform = UiTransform::new(
            "mushroomi".to_string(), Anchor::TopLeft,
            25.0, -25.0, 1.0, 50.0, 50.0, 0
        );
        let m_transform = UiTransform::new(
            "mushroom".to_string(), Anchor::TopLeft,
            100.0, -25.0, 1.0, 50.0, 100.0, 0
        );

        let mi_entity = world
            .create_entity()
            .with(mi_transform)
            .with(UiImage {
                texture: m_texture,
            })
            .build();

        let mushroom_entity = world
            .create_entity()
            .with(m_transform)
            .with(UiText::new(
                font.clone(),
                "0".to_string(),
                [0.0, 0.0, 0.0, 1.0],
                50.0,
            ))
            .build();

        let hi_transform = UiTransform::new(
            "hornworti".to_string(), Anchor::TopLeft,
            175.0, -25.0, 1.0, 50.0, 50.0, 0 // todo shift
        );

        let h_transform = UiTransform::new(
            "hornwort".to_string(), Anchor::TopLeft,
            250.0, -25.0, 1.0, 50.0, 50.0, 0 // todo shift
        );

        let hi_entity = world
            .create_entity()
            .with(hi_transform)
            .with(UiImage {
                texture: h_texture,
            })
            .build();

        let hornwort_entity = world
            .create_entity()
            .with(h_transform)
            .with(UiText::new(
                font.clone(),
                "0".to_string(),
                [0.0, 0.0, 0.0, 1.0],
                50.0,
            ))
            .build();

        let game_over_transform = UiTransform::new(
            "game_over".to_string(), Anchor::Middle,
            0.0, 0.0, 1.0, 900.0, 100.0, 0
        );

        let game_over_entity = world
            .create_entity()
            .with(game_over_transform)
            .with(UiText::new(
                font.clone(),
                "".to_string(),
                [1.0, 0.0, 0.0, 1.0],
                100.0,
            ))
            .build();
        
        world.add_resource(
            UiEntities {
                score_entity, 
                health_entity, 
                game_over_entity,
                mushroom_entity,
                hornwort_entity,
            }
        );
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
        self.sprite_sheet = Some(sprite_sheet_handle.clone());

        LevelState::initialize_ui(world);
        LevelState::create_entities(world, sprite_sheet_handle);
        LevelState::initialize_camera(world);

        world.add_resource(PotionSpawner {
            potion: std::option::Option::None,
        });
    }

    fn handle_event(
        &mut self,
        _data: StateData<GameData>,
        event: StateEvent
    ) -> SimpleTrans {
        if let StateEvent::Window(event) = &event {
            if is_key_down(&event, amethyst::renderer::VirtualKeyCode::Escape) {
                return Trans::Quit;
            }
        }
        return Trans::None;
    }

    fn update(
        &mut self,
        data: &mut StateData<'_, GameData<'_, '_>>
    ) -> SimpleTrans {
        let mut spawn = false;
        let screen_dim = {
            let dimensions = data.world.read_resource::<ScreenDimensions>();
            (dimensions.width(), dimensions.height())
        };
        let potion_info = {
            let mut spawner = data.world.write_resource::<PotionSpawner>();
            match spawner.potion.take() {
                Some(potion_info) => {
                    spawn = true;
                    potion_info
                },
                _ => PotionInfo::default(),
            }
        };

        if spawn {
            if let Some(sprite_sheet) = &self.sprite_sheet {
                // spawn a potion
                
                // TODO translate mouse x/y to world coord
                let logical_mx = (potion_info.mx / screen_dim.0) * CAMERA_WIDTH;
                let logical_my = (potion_info.mx / screen_dim.1) * CAMERA_HEIGHT;

                let world_mx = potion_info.px - CAMERA_WIDTH / 2.0 + logical_mx;
                let world_my = potion_info.py - CAMERA_HEIGHT / 2.0 + logical_my;

                let velocity = 
                    Vector3::new(
                        world_mx - potion_info.px, 
                        world_my - potion_info.py, 0.0
                    )
                    .normalize();

                let mut potion_transform = Transform::default();
                potion_transform.set_xyz(
                    potion_info.px,  // TODO tune spawn dist
                    potion_info.py,
                    0.0
                );
                data.world
                    .create_entity()
                    .with(potion_transform)
                    .with(SpriteRender{
                        sprite_sheet: sprite_sheet.clone(),
                        sprite_number: 2,
                    })
                    .with(Mover{
                        velocity_x: velocity.x * POTION_SPEED,
                        velocity_y: velocity.y * POTION_SPEED,
                        jump_state: JumpState::Airborne,
                        min_x: 0.0,
                        max_x: LEVEL_WIDTH,
                    })
                    .with(Potion{
                        width: 10.0, // hacky way to manage our own collisions
                        height: 10.0,
                    })
                    .build();
            }
        }
        Trans::None
    }
}

// COMPONENTS
// ----------

pub struct Player {
    pub in_hit: bool, // in hit state until hit_last
    pub last_hit: f32,
    pub last_throw: f32,
    pub last_heal: f32,
    pub hit_cooldown: f32, // frequently you can get hit
    pub hit_last: f32,
    pub throw_cooldown: f32,
    pub heal_cooldown: f32,
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
            last_heal: 0.0,
            health: 100,
            hit_cooldown: 1.0,
            heal_cooldown: 0.3,
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

pub struct Enemy {
}

impl Component for Enemy {
    type Storage = VecStorage<Self>;
}

pub struct Potion {
    pub width: f32,
    pub height: f32,
}

impl Component for Potion {
    type Storage = VecStorage<Self>;
}
