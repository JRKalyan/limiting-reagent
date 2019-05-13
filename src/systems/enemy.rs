use amethyst::{
    core::timing::Time,
    core::Transform,
    ecs::{Join, Read, System, WriteStorage, ReadStorage, Entities},
};

use crate::states::Enemy;
use crate::states::Player;
use crate::states::Mover;
use crate::states::Collider;

pub struct EnemySystem {
}

pub const SWAP_RANGE: f32 = 1.0;

pub const ENEMY_VELOCITY: f32 = 50.0;

impl<'s> System<'s> for EnemySystem {
    type SystemData = (
        ReadStorage<'s, Enemy>,
        ReadStorage<'s, Transform>,
        ReadStorage<'s, Collider>,
        WriteStorage<'s, Player>,
        WriteStorage<'s, Mover>,
    );

    fn run(&mut self, (enemies, transforms, colliders, mut players, mut movers): Self::SystemData) {
        for (enemy, enemy_transform, mover) in (&enemies, &transforms, &mut movers).join() {
            if ((enemy_transform.translation().x - mover.max_x).abs() < SWAP_RANGE && mover.velocity_x > 0.0) ||
               ((enemy_transform.translation().x - mover.min_x).abs() < SWAP_RANGE && mover.velocity_x < 0.0) {
               //
               mover.velocity_x = mover.velocity_x * -1.0;
            }
        }
        for (enemy, e_collider, e_transform) in (&enemies, &colliders, &transforms).join() {
            // check for collision, if so then check for e pressed and match on resource
            for (player, player_collider, player_transform, mover) in (&mut players, &colliders, &transforms, &mut movers).join() {
                if crate::collision::check_collision(&player_collider, &player_transform, &e_collider, &e_transform) &&
                   player.last_hit > player.hit_cooldown {
                    player.in_hit = true;
                    player.last_hit = 0.0;
                    player.health -= 20;
                    // instantaneous velocity
                    let mut velocity = player_transform.translation() - e_transform.translation();
                    velocity = velocity.normalize() * 100.0;
                    mover.velocity_x = velocity.x;
                    mover.velocity_y = velocity.y;
                }
            }
        }
    }
}