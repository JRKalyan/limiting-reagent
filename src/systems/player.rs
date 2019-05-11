use amethyst::core::Transform;
use amethyst::ecs::{Join, Read, ReadStorage, System, WriteStorage};
use amethyst::input::InputHandler;

use crate::states::Player;

pub struct PlayerSystem {
}

impl<'s> System<'s> for PlayerSystem {
    type SystemData = (
        WriteStorage<'s, Transform>,
        ReadStorage<'s, Player>,
        Read<'s, InputHandler<String, String>>,
    );

    fn run (&mut self, (mut transforms, players, input): Self::SystemData) {
        for (_player, transform) in (&players, &mut transforms).join() {
            let axis_value = input.axis_value("player");
            if let Some(movement) = axis_value {
                transform.translate_x(1.2 * movement as f32);
            }
        }
    }
}