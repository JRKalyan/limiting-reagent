use amethyst::prelude::*;
use amethyst::renderer::{DisplayConfig, DrawFlat2D, ColorMask, ALPHA, Pipeline,
                         RenderBundle, Stage,};
use amethyst::utils::application_root_dir;
use amethyst::core::TransformBundle;
use amethyst::input::InputBundle;
use amethyst::ui::{DrawUi, UiBundle};
use amethyst::audio::AudioBundle;

mod states;
mod systems;
mod collision;

use states::{LevelState};

pub struct NoMusic;

fn main() -> amethyst::Result<()> {

    //amethyst::start_logger(Default::default());

    let display_config_path = 
        format!("{}/resources/display_config.ron", application_root_dir());
    let bindings_path = 
        format!("{}/resources/bindings.ron", application_root_dir());

    let config = DisplayConfig::load(&display_config_path);

    let pipe = Pipeline::build()
        .with_stage(
            Stage::with_backbuffer()
                .clear_target([1.0, 0.5, 1.0, 1.0], 1.0)
                .with_pass(
                    DrawFlat2D::new()
                        .with_transparency(ColorMask::all(), ALPHA, None))
                .with_pass(
                    DrawUi::new()
                )
        );

    let input_bundle = InputBundle::<String, String>::new()
        .with_bindings_from_file(bindings_path)?;

    let game_data = GameDataBuilder::default()
        .with_bundle(
            RenderBundle::new(pipe, Some(config))
                .with_sprite_sheet_processor()
        )?
        .with_bundle(TransformBundle::new())?
        .with_bundle(input_bundle)?
        .with_bundle(UiBundle::<String, String>::new())?
        .with_bundle(AudioBundle::new(|_: &mut NoMusic|{None}))?
        .with(systems::PlayerSystem{health_tick_rate: 5.0, last_tick: 0.0}, 
            "player_system",  &["input_system"])
        .with(systems::EnemySystem{}, "enemy_system", &["player_system"])
        .with(systems::MoverSystem{}, "mover_system", &["enemy_system"])
        .with(systems::SpriteAnimationSystem{}, "sprite_animation_system", &["mover_system"])
        .with(systems::CameraSystem{}, "camera_system", &["player_system"])
        .with(systems::PotionSystem{}, "potion_system", &[]);

    let mut game = Application::new("./", LevelState{sprite_sheet: None}, game_data)?;

    game.run();

    Ok(())
}