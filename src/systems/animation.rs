use amethyst::{
    core::timing::Time,
    ecs::{Join, Read, System, WriteStorage, ReadStorage},
    renderer::{SpriteRender},
};

use crate::states::SpriteAnimation;
use crate::states::Mover;

pub struct SpriteAnimationSystem {
}

impl<'s> System<'s> for SpriteAnimationSystem {
    type SystemData = (
        WriteStorage<'s, SpriteAnimation>,
        WriteStorage<'s, SpriteRender>,
        ReadStorage<'s, Mover>,
        Read<'s, Time>
    );

    fn run(&mut self, 
           (mut sprite_animations, mut sprite_renders, movers, time): Self::SystemData) {
        for (animation, mover, render) in 
            (&mut sprite_animations, &movers, &mut sprite_renders).join() {
            
            let current_sprite = render.sprite_number - animation.sprite_offset;
            if mover.velocity_x < 0.0 {
                animation.sprite_offset = animation.left_sprite_offset;
            } else if mover.velocity_x > 0.0 {
                animation.sprite_offset = animation.right_sprite_offset;
            }
            render.sprite_number = animation.sprite_offset + current_sprite;
        }

        for (animation, render) in 
            (&mut sprite_animations, &mut sprite_renders).join() {
            animation.elapsed_time += time.delta_seconds();
            let frame = 
                (animation.elapsed_time / animation.time_per_frame) as usize
                % animation.frame_count;
            
            if frame + animation.sprite_offset != render.sprite_number {
                render.sprite_number = animation.sprite_offset + frame;
            }
        }

    }
}
