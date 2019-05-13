use amethyst::core::Transform;
use amethyst::ecs::{Join, Read, ReadStorage, System, WriteStorage};
use amethyst::input::InputHandler;
use amethyst::renderer::{VirtualKeyCode, Camera};

use crate::states::Player;
use crate::states::Mover;
use crate::states::JumpState;
use crate::states::CAMERA_HEIGHT;
use crate::states::CAMERA_WIDTH;

pub struct CameraSystem {
}

struct TransformData {
    x: f32,
    y: f32,
}

impl<'s> System<'s> for CameraSystem {
    type SystemData = (
        ReadStorage<'s, Camera>,
        WriteStorage<'s, Transform>,
        ReadStorage<'s, Player>,
    );

    fn run (&mut self, (cameras, mut transforms, players): Self::SystemData) {
        // TODO could be better implemented by caching player
        let mut player_transform = None;
        for (_player, transform) in (&players, &transforms).join() {
            player_transform = Some(
                TransformData{x: transform.translation().x, y: transform.translation().y});
        }
        let mut camera_x = 0.0;
        let mut camera_y = 0.0;
        if let Some(transform_data) = player_transform {
            camera_x = transform_data.x;
            camera_y = transform_data.y;
        }
        for (_camera, transform) in (&cameras, &mut transforms).join() {
            transform.set_xyz(camera_x - CAMERA_WIDTH / 2.0, camera_y - CAMERA_HEIGHT / 2.0, 1.0);
        }
    }

}

