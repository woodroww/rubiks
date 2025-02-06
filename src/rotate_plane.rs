use bevy_tweening::{Lens, Targetable};
use bevy::prelude::*;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct RotatePlane {
    /// The normalized rotation axis.
    pub axis: Vec3,
    /// Start value of the rotation angle, in radians.
    pub start: f32,
    /// End value of the rotation angle, in radians.
    pub end: f32,
    /// Original transform
    pub org: Transform,
}

impl Lens<Transform> for RotatePlane {
    fn lerp(&mut self, target: &mut dyn Targetable<Transform>, ratio: f32) {
        let angle = (self.end - self.start).mul_add(ratio, self.start);
        let rotation = Quat::from_axis_angle(self.axis, angle);
        let point = Vec3::default();
        target.translation = point + rotation * (self.org.translation - point);
        target.rotation = rotation * self.org.rotation;
    }
}
