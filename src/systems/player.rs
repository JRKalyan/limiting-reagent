use amethyst::core::Transform;
use amethyst::core::timing::Time;
use amethyst::ecs::{Join, Read, ReadStorage, System, WriteStorage, 
    Entities, ReadExpect, Write};
use amethyst::input::InputHandler;
use amethyst::renderer::{VirtualKeyCode};
use amethyst::ui::{UiText};

use crate::states::Player;
use crate::states::Mover;
use crate::states::JumpState;
use crate::states::Ingredient;
use crate::states::Collider;
use crate::states::UiEntities;
use crate::states::UiValues;

pub struct PlayerSystem {
}

impl<'s> System<'s> for PlayerSystem {
    type SystemData = (
        WriteStorage<'s, Mover>,
        WriteStorage<'s, Player>,
        ReadStorage<'s, Ingredient>,
        ReadStorage<'s, Transform>,
        ReadStorage<'s, Collider>,
        Read<'s, InputHandler<String, String>>,
        Read<'s, Time>,
        Entities<'s>,
        ReadExpect<'s, UiEntities>,
        Write<'s, UiValues>,
        WriteStorage<'s, UiText>,
    );

    fn run (&mut self, (mut movers, mut players, ingredients, 
            transforms, colliders, input, time, entities, ui_entities, mut ui_values,
            mut ui_texts): Self::SystemData) {
        for (player, mover, player_collider, player_transform) in (&mut players, &mut movers, &colliders, &transforms).join() {
            if player_transform.translation().y < 0.0 {
                player.health = 0;
            }
            else if player_transform.translation().y >= crate::states::LEVEL_HEIGHT {
                // TODO win the game
            }
            if player.health <= 0 {
                player.health = 0;
                // TODO lose the game;
            }
            // update timings:
            let dt = time.delta_seconds();
            player.last_hit += dt;
            player.last_throw += dt;

            if player.health != ui_values.health {
                ui_values.health = player.health;
                if let Some(text) = ui_texts.get_mut(ui_entities.health_entity) {
                    text.text = format!("HEALTH: {}", ui_values.health);
                }
            }

            let axis_value = input.axis_value("player");
            if player.in_hit {
                if player.last_hit > player.hit_last {
                    player.in_hit = false;
                }
            }
            else if let Some(movement) = axis_value {
                mover.velocity_x = movement as f32 * 140.0;
            }

            if input.key_is_down(VirtualKeyCode::Space) {
                // tell mover to jump
                if let JumpState::Landed = mover.jump_state {
                    mover.jump_state = JumpState::Jump;
                }
            }
            for (e, ingredient, i_collider, i_transform) in (&*entities, &ingredients, &colliders, &transforms).join() {
                // check for collision, if so then check for e pressed and match on resource
                if crate::collision::check_collision(&player_collider, &player_transform, &i_collider, &i_transform) {
                    if input.key_is_down(VirtualKeyCode::LShift) {
                        match ingredient {
                            Ingredient::Hornwort{count} => {
                                player.hornwort += count;
                            },
                            Ingredient::Mushroom{count} => {
                                player.mushroom += count;
                            }
                        }
                        // can also reduce count instead, and change sprite
                        entities.delete(e);
                    }
                }
            }
        }
    }
}