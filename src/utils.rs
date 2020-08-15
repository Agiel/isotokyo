use cgmath::prelude::*;

pub type Vector2 = cgmath::Vector2<f32>;
pub type Vector3 = cgmath::Vector3<f32>;
pub type Vector4 = cgmath::Vector4<f32>;
pub type Point2 = cgmath::Point2<f32>;
pub type Point3 = cgmath::Point3<f32>;

pub type Quaternion = cgmath::Quaternion<f32>;
pub type Matrix4 = cgmath::Matrix4<f32>;

pub const WHITE: (f32, f32, f32, f32) = (1.0, 1.0, 1.0, 1.0);
pub const RED: (f32, f32, f32, f32) = (1.0, 0.0, 0.0, 1.0);
pub const GREEN: (f32, f32, f32, f32) = (0.0, 1.0, 0.0, 1.0);
pub const BLUE: (f32, f32, f32, f32) = (0.0, 0.0, 1.0, 1.0);
pub const MAGENTA: (f32, f32, f32, f32) = (1.0, 0.0, 1.0, 1.0);

pub const CAMERA_DISTANCE: f32 = 20.0;
pub const PIXELS_PER_UNIT: f32 = 64.0;

#[derive(Copy, Clone, Debug)]
pub struct Rect {
    pub position: Point2,
    pub size: Vector2,
}

impl Rect {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            position: Point2::new(x, y),
            size: Vector2::new(w, h),
        }
    }
}

pub fn is_point_inside_rect(point: Point2, rect: Rect, margin: Vector2) -> bool {
    point.x + margin.x >= rect.position.x
        && point.y + margin.y >= rect.position.y
        && point.x - margin.x <= rect.position.x + rect.size.x
        && point.y - margin.y <= rect.position.y + rect.size.y
}

pub fn is_world_point_inside_screen(matrix: Matrix4, point: Point3, margin: Vector2) -> bool {
    let screen_point = matrix.transform_point(point);
    let screen_point = Point2::new(screen_point.x, screen_point.y);
    is_point_inside_rect(screen_point, Rect::new(-1., -1., 2., 2.), margin)
}

#[derive(Debug)]
pub struct Plane {
    pub point: Point3,
    pub normal: Vector3,
}

impl Plane {
    pub fn new(point: Point3, normal: Vector3) -> Self {
        Self { point, normal }
    }
}

#[derive(Debug)]
pub struct Ray {
    pub start: Point3,
    pub direction: Vector3,
}

impl Ray {
    pub fn new(start: Point3, direction: Vector3) -> Self {
        Self { start, direction }
    }
}

pub fn ray_plane_intersection(ray: &Ray, plane: &Plane, max_length: f32) -> Option<f32> {
    let v = ray.direction * max_length;
    let w = plane.point - ray.start;

    let k = w.dot(plane.normal) / v.dot(plane.normal);

    if k >= 0. && k <= 1. {
        Some(k * max_length)
    } else {
        None
    }
}
