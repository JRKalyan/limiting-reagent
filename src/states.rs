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
use amethyst::audio::{output::Output, AudioSink, OggFormat, Source, SourceHandle};

//pub const LEVEL_WIDTH: f32 = 3000.0;
//pub const LEVEL_HEIGHT: f32 = 600.0;

pub const LEVEL_WIDTH: f32 = 3000.0;
pub const LEVEL_HEIGHT: f32 = 600.0;


pub const CAMERA_WIDTH: f32 = 400.0;
pub const CAMERA_HEIGHT: f32 = 225.0;
pub const PLATFORM_HEIGHT: f32 = 25.0;
pub const PLATFORM_WIDTH: f32 = 100.0;
pub const RESOURCE_WIDTH: f32 = 28.0;
pub const RESOURCE_HEIGHT: f32 = 25.0;
pub const PLAYER_HEIGHT: f32 = 25.0;
pub const PLAYER_WIDTH: f32 = 28.0;
pub const GATE_HEIGHT: f32 = 22.0;
pub const GATE_WIDTH: f32 = 26.0;

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
    pub health: i32,
    pub win: bool,
}

pub struct SoundEffects {
    pub hurt: SourceHandle,
    pub potion_throw: SourceHandle,
    pub potion_hit: SourceHandle,
    pub heal: SourceHandle,
    pub jump: SourceHandle,
    pub pickup: SourceHandle,
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
        // create platform entities
        let (plat_x, plat_y) = LevelState::generate_platforms(world, sprite_sheet.clone());

        // Create the player entity:
        let mut player_transform = Transform::default();
        player_transform.set_xyz(plat_x, plat_y + PLATFORM_HEIGHT/2.0 + PLAYER_HEIGHT, 0.0);
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
            .with(Mover::new(-100.0, LEVEL_WIDTH + 100.0))
            .with(Collider{width: 24.0, height: 25.0})
            .build();
    }

    fn generate_platforms(world: &mut World, sprite_sheet: SpriteSheetHandle) -> (f32, f32) {
        let mut rng = rand::thread_rng();

        let jump_x =  PLATFORM_WIDTH + 50.0;
        let jump_y = 70.0;
        let wiggle_y = 30.0;
        let wiggle_x = 35.0;
        let mut offset = false;

        let mut gate_spawned = false;
        let mut gate_x = 0.0;
        let mut gate_y = 0.0;

        let mut first = true;

        let mut nat_y = PLATFORM_HEIGHT / 2.0;
        let mut ret_x = 0.0;
        let mut ret_y = 0.0;
        while nat_y < LEVEL_HEIGHT {
            let mut nat_x = PLATFORM_WIDTH / 2.0;
            if offset {
                nat_x += jump_x;
            } 
            offset = !offset;
            while nat_x < LEVEL_WIDTH - PLATFORM_WIDTH / 2.0 {
                let rand_x: f32 = rng.gen();
                let rand_y: f32 = rng.gen();
                let x = nat_x + wiggle_x * rand_x;
                let y = nat_y + wiggle_y * rand_y;
                gate_y = y;
                gate_x = x;
                println!("{},{}", x, y);

                if first {
                    first = false;
                    ret_y = y;
                    ret_x = x;
                }

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

                // spawn gate
                if (x >= LEVEL_WIDTH / 2.0 && y >= LEVEL_HEIGHT / 2.0) {
                    let roll: f32 = rng.gen();
                    if roll > 0.9 {
                        gate_spawned = true;
                        LevelState::spawn_gate(world, sprite_sheet.clone(),
                            gate_x, gate_y);
                    }
                }
                
                // spawn enemy
                if !first && rng.gen() {
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
                }


                nat_x += jump_x * 2.0;
            }
            nat_y += jump_y;
        }

        if !gate_spawned {
            LevelState::spawn_gate(world, sprite_sheet.clone(),
            gate_x, gate_y);
        }
                
        (ret_x, ret_y)
    }

    fn generate_resources(world: &mut World, sprite_sheet: SpriteSheetHandle,
                          y: f32, x_min: f32, x_max: f32) {
        let mut rng = rand::thread_rng();
        let hornwort_count = rng.gen_range(0, 3);
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

    fn spawn_gate(world: &mut World, sprite_sheet: SpriteSheetHandle,
        px: f32, py: f32) {
            let mut gate_transform = Transform::default();
            gate_transform.set_xyz(px, py + PLATFORM_HEIGHT / 2.0 + GATE_HEIGHT / 2.0, 0.0);

            world.
                create_entity()
                .with(gate_transform)
                .with(Gate{})
                .with(Collider{
                    width: GATE_WIDTH,
                    height: GATE_HEIGHT,
                })
                .with(SpriteRender {
                    sprite_sheet: sprite_sheet,
                    sprite_number: 3,
                })
                .build();

        }
    
    fn initialize_sound(world: &mut World) {
        let effects = {
            let loader = world.read_resource::<Loader>();
            SoundEffects {
                hurt: loader.load("audio/hurt.ogg", OggFormat, (), (), &world.read_resource()),
                potion_throw: loader.load("audio/potion_throw.ogg", OggFormat, (), (), &world.read_resource()),
                potion_hit: loader.load("audio/potion_hit.ogg", OggFormat, (), (), &world.read_resource()),
                jump: loader.load("audio/jump.ogg", OggFormat, (), (), &world.read_resource()),
                pickup: loader.load("audio/pickup.ogg", OggFormat, (), (), &world.read_resource()),
                heal: loader.load("audio/heal.ogg", OggFormat, (), (), &world.read_resource()),
            }
        };
        world.add_resource(effects);
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

        world.add_resource(crate::NoMusic);
        LevelState::initialize_ui(world);
        LevelState::create_entities(world, sprite_sheet_handle);
        LevelState::initialize_camera(world);
        LevelState::initialize_sound(world);

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
                
                let logical_mx = (potion_info.mx / screen_dim.0) * CAMERA_WIDTH;
                let logical_my = (potion_info.my / screen_dim.1) * CAMERA_HEIGHT;

                let world_mx = potion_info.px - CAMERA_WIDTH / 2.0 + logical_mx;
                let world_my = potion_info.py + CAMERA_HEIGHT / 2.0 - logical_my;

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
                        gravity: 0.5,
                        velocity_x: velocity.x * POTION_SPEED,
                        velocity_y: velocity.y * POTION_SPEED,
                        jump_state: JumpState::Airborne,
                        min_x: -100.0,
                        max_x: LEVEL_WIDTH + 100.0,
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
    pub gravity: f32,
}

impl Mover {
    fn new(min_x: f32, max_x: f32) -> Mover {
        Mover {
            velocity_x: 0.0,
            velocity_y: 0.0,
            jump_state: JumpState::Airborne,
            min_x,
            max_x,
            gravity: 1.0,
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

pub struct Gate {
}

impl Component for Gate {
    type Storage = VecStorage<Self>;
}
