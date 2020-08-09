use crate::context::Context;
use crate::camera::*;
use crate::graphics::{Graphics, Texture};
use crate::utils::*;
use crate::state::State;

use std::sync::Arc;
use std::fs;

use cgmath::prelude::*;

pub struct GameState {
    ground_texture: Arc<Texture>,
    prop_texture: Arc<Texture>,
    player_texture: Arc<Texture>,
    shadow_texture: Arc<Texture>,
    camera: Camera,
}

impl GameState {
    pub fn new(gfx: &Graphics) -> Self {
        let view =
            Matrix4::from_angle_z(cgmath::Deg(45.0)) * Matrix4::from_angle_x(cgmath::Deg(-30.0));
        let camera = Camera {
            projection: Projection::Orthographic,
            eye: view.transform_point(Point3::origin() + Vector3::unit_y() * -CAMERA_DISTANCE),
            target: Point3::origin(),
            up: Vector3::unit_z(),
            aspect: gfx.extent.width as f32 / gfx.extent.height as f32,
            fovy: gfx.extent.height as f32 / PIXELS_PER_UNIT,
            znear: 0.1,
            zfar: CAMERA_DISTANCE * 2.0,
        };
        //let camera = graphics::Camera {
        //    eye: (0.0, 5.0, 10.0).into(),
        //    target: (0.0, 0.0, 0.0).into(),
        //    up: Vector3::unit_z(),
        //    aspect: size.width as f32 / size.height as f32,
        //    fovy: 45.0,
        //    znear: 0.1,
        //    zfar: 100.0,
        //    projection: graphics::Projection::Perspective,
        //};

        let ground_bytes = fs::read("resources/grass1.png").unwrap();
        let ground_texture = gfx
            .load_texture_bytes(ground_bytes.as_slice(), "grass1.png")
            .unwrap();

        let prop_bytes = fs::read("resources/sakura1.png").unwrap();
        let prop_texture = gfx
            .load_texture_bytes(prop_bytes.as_slice(), "sakura1.png")
            .unwrap();

        let player_bytes = fs::read("resources/nsf_idle.png").unwrap();
        let player_texture = gfx
            .load_texture_bytes(player_bytes.as_slice(), "nsf_idle.png")
            .unwrap();

        let shadow_bytes = fs::read("resources/blob_shadow.png").unwrap();
        let shadow_texture = gfx
            .load_texture_bytes(shadow_bytes.as_slice(), "blob_shadow.png")
            .unwrap();

        Self {
            ground_texture,
            prop_texture,
            player_texture,
            shadow_texture,
            camera,
        }
    }
}

impl State for GameState {
    fn update(&mut self, ctx: &Context) {
    }

    fn draw(&self, gfx: &mut Graphics) {
        for x in -10..10 {
            for y in -10..10 {
                gfx.draw_plane(
                    &self.ground_texture,
                    Point3::new(x as f32, y as f32, 0.0),
                    1.0,
                    WHITE.into(),
                );
            }
        }
        gfx.draw_billboard(
            &self.camera,
            &self.prop_texture,
            Rect::new(0., 0., 1., 1.),
            Point3::new(0.0, 0.0, 0.5),
            Vector2::new(1.5, 2.0),
            Vector3::new(0.0, 0.5, 0.0),
        );
        gfx.draw_plane(
            &self.shadow_texture,
            Point3::new(0.0, 0.0, 0.0),
            1.0,
            (0.0, 0.0, 0.0, 0.8).into(),
        );

        gfx.draw_billboard(
            &self.camera,
            &self.player_texture,
            Rect::new(0., 0.5, 1., 0.125),
            Point3::new(-4.0, -4.0, 0.5),
            Vector2::new(1.0, 1.0),
            Vector3::new(0.0, 0.0, 0.0),
        );
        gfx.draw_plane(
            &self.shadow_texture,
            Point3::new(-4.0, -4.0, 0.0),
            1.0,
            (0.0, 0.0, 0.0, 0.8).into(),
        );

        gfx.draw_debug_cube(Point3::origin(), WHITE.into());
        gfx.draw_debug_cube(Point3::new(1.0, 0.0, 0.0), WHITE.into());

        gfx.draw_debug_grid(Point3::new(0.0, 0.0, 0.25), 20);

        gfx.draw(&self.camera);
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        match self.camera.projection {
            Projection::Orthographic => {
                self.camera.fovy = new_size.height as f32 / PIXELS_PER_UNIT
            }
            _ => (),
        }
        self.camera.aspect = new_size.width as f32 / new_size.height as f32;
    }
}
