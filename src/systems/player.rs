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
use crate::states::{PotionSpawner, PotionInfo};

pub const ATTACK_H_COST: usize = 3;
pub const ATTACK_M_COST: usize = 1;
pub const HEAL_H_COST: usize = 1;
pub const HEAL_M_COST: usize = 3;
pub const HEAL_AMOUNT: i32 = 10;

pub struct PlayerSystem {
    pub health_tick_rate: f32,
    pub last_tick: f32,
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
        Write<'s, PotionSpawner>,
    );

    fn run (&mut self, (mut movers, mut players, ingredients, 
            transforms, colliders, input, time, entities, ui_entities, mut ui_values,
            mut ui_texts, mut potion_spawner): Self::SystemData) {

        
        let mut tick = false;
        let mut h_update = false;
        let mut m_update = false;
        let dt = time.delta_seconds();
        self.last_tick += dt;
        if self.last_tick >= self.health_tick_rate {
            self.last_tick = 0.0;
            tick = true;
        }
        let mut lose = true;
        for (ep, player, mover, player_collider, player_transform) in 
            (&*entities, &mut players, &mut movers, &colliders, &transforms).join() {
            
            lose = false; // if a player exists we don't lose

            if tick {
                player.health -= 1;
            }

            if player_transform.translation().y < -50.0 {
                player.health = 0;
            }

            if player.health <= 0 {
                player.health = 0;
                lose = true;
            }

            // update timings:
            player.last_hit += dt;
            player.last_throw += dt;
            player.last_heal += dt;

            if player.health != ui_values.health {
                ui_values.health = player.health;
                if let Some(text) = ui_texts.get_mut(ui_entities.health_entity) {
                    if ui_values.health <= 50 {
                        text.color = [0.8, 0.0, 0.0, 1.0];
                    }
                    else {
                        text.color = [0.0, 0.8, 0.0, 1.0];
                    }
                    text.text = format!("HEALTH: {}", ui_values.health);
                }
            }

            // move
            let axis_value = input.axis_value("player");
            if player.in_hit {
                if player.last_hit > player.hit_last {
                    player.in_hit = false;
                }
            }
            else if let Some(movement) = axis_value {
                mover.velocity_x = movement as f32 * 140.0;
            }

            // jump
            if input.key_is_down(VirtualKeyCode::Space) {
                // tell mover to jump
                if let JumpState::Landed = mover.jump_state {
                    mover.jump_state = JumpState::Jump;
                }
            }

            // throw potions
            let mut throw = false;
            if input.action_is_down("throw").unwrap() {
                if player.last_throw > player.throw_cooldown {
                    if player.hornwort >= ATTACK_H_COST && 
                        player.mushroom >= ATTACK_M_COST {

                        player.hornwort -= ATTACK_H_COST;
                        player.mushroom -= ATTACK_M_COST;

                        let ptrans = player_transform.translation();

                        // grab mouse target
                        let mouse_position = match input.mouse_position() {
                            Some(pos) => pos,
                            _ => (0.0, 0.0)
                        };

                        potion_spawner.potion = Some(
                            PotionInfo {
                                px: ptrans.x,
                                py: ptrans.y,
                                mx: mouse_position.0 as f32,
                                my: mouse_position.1 as f32,
                            }
                        );

                        player.last_throw = 0.0;
                        throw = true;
                    } 
                }
            }

            if input.action_is_down("heal").unwrap() && 
                player.last_heal > player.heal_cooldown {
                if player.hornwort >= HEAL_H_COST && 
                    player.mushroom >= HEAL_M_COST {

                    player.hornwort -= HEAL_H_COST;
                    player.mushroom -= HEAL_M_COST;

                    // TODO play a sound effect
                    player.health += HEAL_AMOUNT;
                    player.last_heal = 0.0;

                    throw = true;
                }
            }

            if throw {
                h_update = true;
                m_update = true;
                player.last_throw = 0.0;
            }


            // check for ingredient pickups
            for (e, ingredient, i_collider, i_transform) in 
                (&*entities, &ingredients, &colliders, &transforms).join() {
                // check for collision, if so then check for e pressed and match on resource
                if crate::collision::check_collision(&player_collider, &player_transform, &i_collider, &i_transform) {
                    if input.key_is_down(VirtualKeyCode::S) {
                        match ingredient {
                            Ingredient::Hornwort{count} => {
                                player.hornwort += count;
                                h_update = true;
                           },
                            Ingredient::Mushroom{count} => {
                                player.mushroom += count;
                                m_update = true;
                            }
                        }
                        entities.delete(e);
                    }
                }
            }

            if h_update {
                if let Some(text) = ui_texts.get_mut(ui_entities.hornwort_entity) {
                    text.text = player.hornwort.to_string();
                }
            }

            if m_update {
                if let Some(text) = ui_texts.get_mut(ui_entities.mushroom_entity) {
                    text.text = player.mushroom.to_string();
                }
            }

            if lose {
                entities.delete(ep);
            }
        }
        

        if lose {
            // TODO - in an actual game would try and use state system
            // and implement restart mechanic
            if let Some(text) = ui_texts.get_mut(ui_entities.game_over_entity) {
                text.text = "GAME OVER".to_string();
            }
        }
    }
}