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
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
    pub projection: Projection,
}

impl Camera {
    pub fn build_view_projection_matrix(&self) -> Matrix4 {
        let view = Matrix4::look_at(self.eye, self.target, self.up);
        let proj = match self.projection {
            Projection::Perspective => cgmath::perspective(
                cgmath::Deg(self.fovy),
                self.aspect, 
                self.znear, 
                self.zfar
            ),
            Projection::Orthographic => {
                let height = self.fovy;
                let width = height * self.aspect;
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
        let (screen_width, screen_height) = (self.fovy * self.aspect, self.fovy);
        let x = 2. * point.x / screen_width - 1.;
        let y = 1. - 2. * point.y / screen_height;

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
