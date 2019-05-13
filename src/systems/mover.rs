use amethyst::{
    core::timing::Time,
    core::Transform,
    ecs::{Join, Read, System, WriteStorage, ReadStorage, Entities},
};

use crate::states::Mover;
use crate::states::Platform;
use crate::states::Collider;
use crate::states::JumpState;

const MAX_DROP_VELOCITY: f32 = 600.0;
const GRAVITY: f32 = 400.0;
const JUMP_VELOCITY: f32 = 250.0;

pub struct MoverSystem {
}

impl<'s> System<'s> for MoverSystem {

    type SystemData = (
        WriteStorage<'s, Transform>,
        WriteStorage<'s, Mover>,
        ReadStorage<'s, Platform>,
        ReadStorage<'s, Collider>,
        Read<'s, Time>,
        Entities<'s>,
    );

    fn run(&mut self, (mut transforms, mut movers, 
                       platforms, colliders, time,
                       entities): Self::SystemData) {
        let dt = time.delta_seconds();
        let dv = dt * -GRAVITY;
        for (mover, transform) in (&mut movers, &mut transforms).join() {
            if let JumpState::Jump = mover.jump_state {
                mover.velocity_y = mover.velocity_y + JUMP_VELOCITY;
                mover.jump_state = JumpState::Airborne;
            }
            // translate distance
            transform.translate_y(1.2 * mover.velocity_y * dt + 0.5 * dv * dt);
            transform.translate_x(mover.velocity_x * dt);

            if transform.translation().x > mover.max_x {
                transform.set_x(mover.max_x);
            } else if transform.translation().x < mover.min_x {
                transform.set_x(mover.min_x);
            }

            let mut new_velocity_y = mover.velocity_y + dv;

            if new_velocity_y < -MAX_DROP_VELOCITY {
                new_velocity_y = -MAX_DROP_VELOCITY;
            }

            mover.velocity_y = new_velocity_y;
        }

        for (em, mover, mover_collider) in 
            (&*entities, &mut movers, &colliders).join() {
            mover.jump_state = JumpState::Airborne;
            // iterate over platforms that have colliders
            for (ep, platform, platform_collider) in 
                (&*entities, &platforms, &colliders).join() {

                let platform_translation = transforms.get(ep).unwrap().translation();
                let platform_x = platform_translation.x;
                let platform_y = platform_translation.y;
                let mover_transform = transforms.get_mut(em).unwrap();
                let mover_translation = mover_transform.translation();

                let pl = platform_x - platform_collider.width / 2.0;
                let pt = platform_y + platform_collider.height / 2.0;
                let pr = platform_x + platform_collider.width / 2.0;
                let pb = platform_y - platform_collider.height / 2.0;

                let wl = mover_translation.x - mover_collider.width / 2.0;
                let wt = mover_translation.y + mover_collider.height / 2.0;
                let wr = mover_translation.x + mover_collider.width / 2.0;
                let wb = mover_translation.y - mover_collider.height / 2.0;

                if wl < pr &&
                    wr > pl &&
                    wt > pb &&
                    wb < pt {

                    let displacement_r = pr - wl; 
                    let displacement_l = pl - wr;
                    let displacement_t = pt - wb;
                    let displacement_b = pb - wt;

                    let mut min = displacement_r.abs();
                    min = if displacement_l.abs() < min{displacement_l.abs()} else {min};
                    min = if displacement_t.abs() < min{displacement_t.abs()} else {min};
                    min = if displacement_b.abs() < min{displacement_b.abs()} else {min};

                    if displacement_r.abs() == min {
                        mover_transform.translate_x(displacement_r);
                    }
                    else if displacement_l.abs() == min {
                        mover_transform.translate_x(displacement_l);
                    }
                    else if displacement_t.abs() == min {
                        if mover.velocity_y < 0.0 {
                            mover.velocity_y = 0.0;
                            mover.jump_state = JumpState::Landed;
                        }
                        mover_transform.translate_y(displacement_t);
                    }
                    else if displacement_b.abs() == min {
                        if mover.velocity_y > 0.0 {
                            mover.velocity_y = 0.0;
                        }
                        mover_transform.translate_y(displacement_b);
                    }
                }
            }
        }
    }
}