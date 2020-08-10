use crate::utils::*;
use cgmath::prelude::*;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Matrix4 = Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

pub enum Projection {
    Orthographic,
    Perspective,
}

pub struct Camera {
    pub eye: Point3,
    pub target: Point3,
    pub up: Vector3,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
    pub projection: Projection,
    pub screen_size: Vector2,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            eye: Point3::origin(),
            target: Point3::origin(),
            up: Vector3::unit_z(),
            fovy: 75.0,
            znear: 0.1,
            zfar: 1000.0,
            projection: Projection::Orthographic,
            screen_size: Vector2::new(1920., 1080.),
        }
    }
}

impl Camera {
    pub fn build_view_projection_matrix(&self) -> Matrix4 {
        let aspect = self.screen_size.x / self.screen_size.y;
        let view = Matrix4::look_at(self.eye, self.target, self.up);
        let proj = match self.projection {
            Projection::Perspective => cgmath::perspective(
                cgmath::Deg(45.0),
                aspect,
                self.znear,
                self.zfar
            ),
            Projection::Orthographic => {
                let width = self.screen_size.x / PIXELS_PER_UNIT;
                let height = self.screen_size.y / PIXELS_PER_UNIT;
                cgmath::ortho(
                    -width / 2.,
                    width / 2.,
                    -height / 2.,
                    height / 2.,
                    self.znear,
                    self.zfar
                )
            }
        };

        proj * view
    }

    pub fn screen_to_ray(&self, point: Point2) -> Ray {
        let x = 2. * point.x / self.screen_size.x - 1.;
        let y = 1. - 2. * point.y / self.screen_size.y;

        let mat = self.build_view_projection_matrix().invert().unwrap();

        let near = mat.transform_point(Point3::new(x, y, 0.));
        let far = mat.transform_point(Point3::new(x, y, 1.));

        let direction = (far - near).normalize();
        let start = match self.projection {
            Projection::Perspective => self.eye,
            Projection::Orthographic => mat.transform_point(Point3::new(x, y, -1.)),
        };

        Ray::new(start, direction)
    }
}
