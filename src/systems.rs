//! # Systems

use glam::Mat4;

use crate::components::WorldTransform;
use crate::ComputedVisibility;
use crate::LocalTransform;
use crate::Node;
use crate::Scene;
use crate::Visibility;

/// Computes the visibility for all of the nodes in the scene.
pub fn compute_visibility(scene: &Scene) {
    for node in scene.get_root_nodes() {
        compute_visibility_internal(scene, node, ComputedVisibility::Visible);
    }
}

fn compute_visibility_internal(scene: &Scene, node: Node, parent_visibility: ComputedVisibility) {
    let visibility = match scene.get::<Visibility>(node) {
        Some(Visibility::Inherit) => parent_visibility,
        Some(Visibility::Visible) => ComputedVisibility::Visible,
        Some(Visibility::Invisible) => ComputedVisibility::Invisible,
        None => parent_visibility,
    };

    scene.set_or_add(node, visibility);

    for node in scene.get_children(node).into_iter().flatten().copied() {
        compute_visibility_internal(scene, node, visibility);
    }
}

/// Computes the world transform for all of the nodes in the scene with a [LocalTransform]
/// component.
pub fn compute_world_transform(scene: &Scene) {
    for node in scene.get_root_nodes() {
        compute_world_transform_internal(scene, node, WorldTransform::IDENTITY);
    }
}

fn compute_world_transform_internal(scene: &Scene, node: Node, parent_transform: WorldTransform) {
    let transform = match scene.get::<LocalTransform>(node) {
        Some(transform) => {
            let transform = WorldTransform::new(
                parent_transform.matrix
                    * Mat4::from_scale_rotation_translation(
                        transform.scale,
                        transform.rotation,
                        transform.position,
                    ),
            );

            scene.set_or_add(node, transform);

            transform
        }
        None => WorldTransform::IDENTITY,
    };

    for node in scene.get_children(node).into_iter().flatten().copied() {
        compute_world_transform_internal(scene, node, transform);
    }
}
