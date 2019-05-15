use amethyst::{
    core::timing::Time,
    core::Transform,
};

use crate::states::Collider;


pub fn check_collision(collider_1: &Collider, transform_1: &Transform,
                       collider_2: &Collider, transform_2: &Transform) -> bool {
    // simple collision for AABB
    let translation_1 = transform_1.translation();
    let translation_2 = transform_2.translation();

    let pl = translation_1.x - collider_1.width / 2.0;
    let pt = translation_1.y + collider_1.height / 2.0;
    let pr = translation_1.x + collider_1.width / 2.0;
    let pb = translation_1.y - collider_1.height / 2.0;

    let wl = translation_2.x - collider_2.width / 2.0;
    let wt = translation_2.y + collider_2.height / 2.0;
    let wr = translation_2.x + collider_2.width / 2.0;
    let wb = translation_2.y - collider_2.height / 2.0;

    return wl < pr && wr > pl && wt > pb && wb < pt;
}