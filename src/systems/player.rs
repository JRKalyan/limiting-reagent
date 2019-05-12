use amethyst::core::Transform;
use amethyst::ecs::{Join, Read, ReadStorage, System, WriteStorage};
use amethyst::input::InputHandler;
use amethyst::renderer::{VirtualKeyCode};

use crate::states::Player;
use crate::states::Mover;
use crate::states::JumpState;

pub struct PlayerSystem {
}

impl<'s> System<'s> for PlayerSystem {
    type SystemData = (
        WriteStorage<'s, Mover>,
        ReadStorage<'s, Player>,
        Read<'s, InputHandler<String, String>>,
    );

    fn run (&mut self, (mut movers, players, input): Self::SystemData) {
        for (_player, mover) in (&players, &mut movers).join() {
            let axis_value = input.axis_value("player");
            if let Some(movement) = axis_value {
                mover.velocity_x = movement as f32 * 100.0;
            }
            if input.key_is_down(VirtualKeyCode::Space) {
                // tell mover to jump
                if let JumpState::Landed = mover.jump_state {
                    mover.jump_state = JumpState::Jump;
                }
            }
        }
    }
}