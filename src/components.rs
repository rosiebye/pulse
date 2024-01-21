use glam::Mat4;
use glam::Quat;
use glam::Vec3;

use crate::Component;

/// # Visibility
///
/// Visibility of the node.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum Visibility {
    /// Inherit the visibility from the node's parent.
    #[default]
    Inherit,
    /// Node is visible.
    Visible,
    /// Node is not visible.
    Invisible,
}

impl Component for Visibility {}

/// # Computed Visibility
///
/// Computed visibility of the node.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ComputedVisibility {
    /// Node is visible.
    Visible,
    /// Node is not visible.
    Invisible,
}

impl Component for ComputedVisibility {}

/// # Local Transform
///
/// Position, rotation, and scale of the node relative to its parent.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct LocalTransform {
    /// Position of the transform.
    pub position: Vec3,
    /// Rotation of the transform.
    pub rotation: Quat,
    /// Scale of the transform.
    pub scale: Vec3,
}

impl LocalTransform {
    /// Identity transform.
    pub const IDENTITY: Self = Self {
        position: Vec3::ZERO,
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    };

    /// Returns a transform with the given position, rotation, and scale.
    pub const fn new(position: Vec3, rotation: Quat, scale: Vec3) -> Self {
        Self {
            position,
            rotation,
            scale,
        }
    }

    /// Returns a transform with the given position.
    pub const fn from_position(position: Vec3) -> Self {
        Self {
            position,
            ..Self::IDENTITY
        }
    }
}

impl Component for LocalTransform {}

impl Default for LocalTransform {
    fn default() -> Self {
        Self::IDENTITY
    }
}

/// # World Transform
///
/// Transform of the node in world coordinates.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct WorldTransform {
    /// Transform matrix.
    pub matrix: Mat4,
}

impl WorldTransform {
    /// Identity transform.
    pub const IDENTITY: Self = Self {
        matrix: Mat4::IDENTITY,
    };

    /// Returns a transform with the given transform matrix.
    pub const fn new(matrix: Mat4) -> Self {
        Self { matrix }
    }
}

impl Component for WorldTransform {}

impl Default for WorldTransform {
    fn default() -> Self {
        Self::IDENTITY
    }
}
