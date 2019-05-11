use amethyst::core::Transform;
use amethyst::ecs::{Join, ReadStorage, System, WriteStorage};

use crate::states::Weight;

pub struct WeightSystem {
}

impl<'s> System<'s> for WeightSystem {

    type SystemData = (
        WriteStorage<'s, Transform>,
        ReadStorage<'s, Weight>,
    );

    fn run(&mut self, (mut transforms, weights): Self::SystemData) {
        for (_weight, transform) in (&weights, &mut transforms).join() {
            // TODO apply the gravity properly
            transform.translate_y(-0.1);
        }
    }
}
