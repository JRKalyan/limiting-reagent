use amethyst::{
    core::timing::Time,
    ecs::{Join, Read, System, WriteStorage, ReadStorage, Entities},
    renderer::{SpriteRender, Flipped},
};

use crate::states::SpriteAnimation;
use crate::states::Mover;
use crate::states::JumpState;

pub struct SpriteAnimationSystem {
}

impl<'s> System<'s> for SpriteAnimationSystem {
    type SystemData = (
        Entities<'s>,
        WriteStorage<'s, SpriteAnimation>,
        WriteStorage<'s, SpriteRender>,
        WriteStorage<'s, Flipped>,
        ReadStorage<'s, Mover>,
        Read<'s, Time>,
    );

    fn run(&mut self, 
           (entities, mut sprite_animations, mut sprite_renders, mut flipped_components, movers, time): Self::SystemData) {
        for (e, animation, mover, render) in 
            (&*entities, &mut sprite_animations, &movers, &mut sprite_renders).join() {
            
            if mover.velocity_x < 0.0 {
                flipped_components.insert(e, Flipped::Horizontal).unwrap();
            }
            else if mover.velocity_x > 0.0 {
                flipped_components.remove(e);
            }
        }

        for (animation, render, mover) in 
            (&mut sprite_animations, &mut sprite_renders, &movers).join() {

            animation.elapsed_time += time.delta_seconds();

            if let JumpState::Airborne = mover.jump_state {
                render.sprite_number = animation.airborne_offset;
            } else if mover.velocity_x.abs() > 0.0 {
                let frame = (animation.elapsed_time / animation.time_per_frame) as usize
                    % animation.move_count;
                render.sprite_number = animation.move_offset + frame;
            } else {
                let frame = (animation.elapsed_time / animation.time_per_frame) as usize
                    % animation.idle_count;
                render.sprite_number = animation.idle_offset + frame;
            }
        }
    }
}
