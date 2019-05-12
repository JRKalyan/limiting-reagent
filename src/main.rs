use amethyst::prelude::*;
use amethyst::renderer::{DisplayConfig, DrawFlat2D, Event, Pipeline,
                         RenderBundle, Stage, VirtualKeyCode};
use amethyst::utils::application_root_dir;
use amethyst::core::TransformBundle;
use amethyst::input::InputBundle;

mod states;
mod systems;

use states::{LevelState};

fn main() -> amethyst::Result<()> {

    amethyst::start_logger(Default::default());

    let display_config_path = 
        format!("{}/resources/display_config.ron", application_root_dir());
    let bindings_path = 
        format!("{}/resources/bindings.ron", application_root_dir());

    let config = DisplayConfig::load(&display_config_path);

    let pipe = Pipeline::build()
        .with_stage(
            Stage::with_backbuffer()
                .clear_target([1.0, 0.5, 1.0, 1.0], 1.0)
                .with_pass(DrawFlat2D::new()),
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
        .with(systems::PlayerSystem{}, "player_system",  &["input_system"])
        .with(systems::MoverSystem{}, "weight_system", &["player_system"])
        .with(systems::SpriteAnimationSystem{}, "sprite_animation_system", &[]);

    let mut game = Application::new("./", LevelState{}, game_data)?;

    game.run();

    Ok(())
}