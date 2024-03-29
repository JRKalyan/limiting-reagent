use amethyst::{
    core::Transform,
    ecs::{Join, Read, System, WriteStorage, ReadStorage, 
        Entities, ReadExpect, Write},
    audio::{output::Output, Source},
    assets::AssetStorage,
};
use amethyst::ui::{UiText};

use crate::states::Enemy;
use crate::states::Mover;
use crate::states::Platform;
use crate::states::Collider;
use crate::states::Potion;
use crate::collision::check_collision;
use crate::states::UiEntities;
use crate::states::UiValues;
use crate::states::SoundEffects;


pub struct PotionSystem {
}

impl<'s> System<'s> for PotionSystem {
    type SystemData = (
        Entities<'s>,
        ReadStorage<'s, Transform>,
        ReadStorage<'s, Enemy>,
        ReadStorage<'s, Platform>,
        ReadStorage<'s, Potion>,
        ReadStorage<'s, Collider>,
        ReadExpect<'s, UiEntities>,
        Write<'s, UiValues>,
        WriteStorage<'s, UiText>,
        Read<'s, AssetStorage<Source>>,
        ReadExpect<'s, SoundEffects>,
        Option<Read<'s, Output>>,
    );

    fn run(&mut self, 
        (entities, transforms, enemies, platforms, potions, colliders,
         ui_entities, mut ui_values, mut ui_texts,
         audio_source, sound_effects, audio_output): Self::SystemData) {
        //
        for (ep, potion, p_transform) in 
            (&* entities, &potions, &transforms).join() {
            // Check for collisions with enemies to delete them
            let p_collider = Collider {
                width: potion.width,
                height: potion.height,
            };
            for (e, _enemy, e_transform, e_collider) in
                (&*entities, &enemies, &transforms, &colliders).join() {
                if (check_collision(&p_collider, &p_transform,
                    e_collider, e_transform)) {
                    // update the score
                    if let Some(text) = ui_texts.get_mut(ui_entities.score_entity) {
                        ui_values.score += 1;
                        text.text = format!("SCORE: {}", ui_values.score).to_string();
                    }

                    if let Some(ref out_device) = audio_output.as_ref() {
                        if let Some(sound) = audio_source.get(&sound_effects.potion_hit) {
                            out_device.play_once(sound, 0.2);
                        }
                    }


                    entities.delete(e).unwrap();
                    entities.delete(ep).unwrap();
                }
            }
            for (_platform, e_transform, e_collider) in
                (&platforms, &transforms, &colliders).join() {
                if (check_collision(&p_collider, &p_transform,
                    e_collider, e_transform)) {
                    entities.delete(ep).unwrap();
                }
            }
        }
    }

}
