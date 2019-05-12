use amethyst::{
    core::timing::Time,
    core::Transform,
    ecs::{Join, Read, System, WriteStorage, ReadStorage},
};

use crate::states::Mover;
use crate::states::Platform;
use crate::states::Collider;
use crate::states::JumpState;

const MAX_DROP_VELOCITY: f32 = 600.0;
const GRAVITY: f32 = 300.0;
const JUMP_VELOCITY: f32 = 100.0;

pub struct MoverSystem {
}

impl<'s> System<'s> for MoverSystem {

    type SystemData = (
        WriteStorage<'s, Transform>,
        WriteStorage<'s, Mover>,
        ReadStorage<'s, Platform>,
        ReadStorage<'s, Collider>,
        Read<'s, Time>,
    );

    fn run(&mut self, (mut transforms, mut movers, 
                       platforms, colliders,  time): Self::SystemData) {
        let dt = time.delta_seconds();
        let dv = dt * -GRAVITY;
        for (mover, transform) in (&mut movers, &mut transforms).join() {
            if let JumpState::Jump = mover.jump_state {
                mover.velocity_y = mover.velocity_y + JUMP_VELOCITY;
                mover.jump_state = JumpState::Airborne;
            }
            // translate distance
            transform.translate_y(mover.velocity_y * dt + 0.5 * dv * dt);
            transform.translate_x(mover.velocity_x * dt);

            let mut new_velocity_y = mover.velocity_y + dv;

            if new_velocity_y < -MAX_DROP_VELOCITY {
                new_velocity_y = -MAX_DROP_VELOCITY;
            }

            mover.velocity_y = new_velocity_y;
        }

        for (mover, mover_transform, mover_collider) in 
            (&mut movers, &mut transforms, &colliders).join() {
                mover.jump_state = JumpState::Airborne;
                // iterate over platforms that have colliders
                for (platform, platform_collider) in 
                    (&platforms, &colliders).join() {
                        let mover_translation = mover_transform.translation();
                        let pl = platform.x - platform_collider.width / 2.0;
                        let pt = platform.y + platform_collider.height / 2.0;
                        let pr = platform.x + platform_collider.width / 2.0;
                        let pb = platform.y - platform_collider.height / 2.0;

                        let wl = mover_translation.x - mover_collider.width / 2.0;
                        let wt = mover_translation.y + mover_collider.height / 2.0;
                        let wr = mover_translation.x + mover_collider.width / 2.0;
                        let wb = mover_translation.y - mover_collider.height / 2.0;
                        // TODO - just mutably set the weights
                        // then do a second mutable pass over the (weight, transform)
                        // to perform the updates cached...
                        // then I can move the x/y out of the platform

                        if wl < pr &&
                           wr > pl &&
                           wt > pb &&
                           wb < pt {
                                if wt > pt {
                                    mover.velocity_y = 0.0;
                                    mover.jump_state = JumpState::Landed;
                                    mover_transform.set_y(
                                        pt + mover_collider.height / 2.0);
                                } else if wb < pb {
                                    mover.velocity_y = 0.0;
                                    mover_transform.set_y(
                                        pb - mover_collider.height / 2.0);
                                } else if wr > pr {
                                    mover.velocity_x = 0.0;
                                    mover_transform.set_x(
                                        pr + mover_collider.width / 2.0);
                                } else {
                                    mover.velocity_x = 0.0;
                                    mover_transform.set_x(
                                        pl - mover_collider.width / 2.0);
                                }
                        }
                 }
        }
    }
}