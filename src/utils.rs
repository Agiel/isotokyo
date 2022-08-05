use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// A 3D ray, with an origin and direction. The direction is guaranteed to be normalized.
#[derive(Debug, PartialEq, Copy, Clone, Default, Serialize, Deserialize)]
pub struct Ray3d {
    pub(crate) origin: Vec3,
    pub(crate) direction: Vec3,
}

impl Ray3d {
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Ray3d { origin, direction }
    }

    pub fn from_screenspace(windows: &Res<Windows>, camera: &Camera, camera_transform: &GlobalTransform) -> Option<Self> {
        let window = match windows.get_primary() {
            Some(w) => w,
            None => return None,
        };
        let cursor_position = match window.cursor_position() {
            Some(c) => c,
            None => return None,
        };

        let view = camera_transform.compute_matrix();
        let screen_size = camera.logical_target_size()?;
        let projection = camera.projection_matrix();
        let far_ndc = projection.project_point3(Vec3::NEG_Z).z;
        let near_ndc = projection.project_point3(Vec3::Z).z;
        let cursor_ndc = (cursor_position / screen_size) * 2.0 - Vec2::ONE;
        let ndc_to_world: Mat4 = view * projection.inverse();
        let near = ndc_to_world.project_point3(cursor_ndc.extend(near_ndc));
        let far = ndc_to_world.project_point3(cursor_ndc.extend(far_ndc));
        let ray_direction = far - near;

        Some(Ray3d::new(near, ray_direction))
    }

    pub fn intersect_y_plane(&self, y_offset: f32) -> Option<Vec3> {
        let plane_normal = Vec3::Y;
        let plane_origin = Vec3::new(0.0, y_offset, 0.0);
        let denominator = self.direction.dot(plane_normal);
        if denominator.abs() > f32::EPSILON {
            let point_to_point = plane_origin - self.origin;
            let intersect_dist = plane_normal.dot(point_to_point) / denominator;
            let intersect_position = self.direction * intersect_dist + self.origin;
            Some(intersect_position)
        } else {
            None
        }
    }
}
