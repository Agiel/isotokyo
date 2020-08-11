use crate::camera::*;
use crate::context::Context;
use crate::assets::Assets;
use crate::graphics::Graphics;
use crate::state::State;
use crate::input::Action;
use crate::utils::*;
use actor::Actor;

use cgmath::prelude::*;
use rand::Rng;

use std::collections::HashSet;

mod actor;
mod player;

pub struct Commands {
    pub actions: HashSet<Action>,
    pub aim_ray: Ray,
    pub wish_dir: Vector3,
}

pub struct GameState {
    camera: Camera,
    actors: Vec<Actor>,
    aim_point: Option<Point3>,
}

impl GameState {
    pub fn new(assets: &mut Assets, ctx: &Context, gfx: &Graphics) -> Self {
        ctx.set_cursor_grab(true);

        let view =
            Matrix4::from_angle_z(cgmath::Deg(45.0)) * Matrix4::from_angle_x(cgmath::Deg(-30.0));

        let screen_size = Vector2::new(gfx.extent.width as f32, gfx.extent.height as f32);
        let camera = Camera {
            // projection: Projection::Perspective,
            eye: view.transform_point(Point3::origin() + Vector3::unit_y() * -CAMERA_DISTANCE),
            target: Point3::origin(),
            zfar: CAMERA_DISTANCE * 2.0,
            screen_size,
            ..Default::default()
        };

        assets.load_texture("grass", "grass1.png", gfx).unwrap();
        assets.load_texture("sakura", "sakura1.png", gfx).unwrap();
        assets.load_texture("nsf_idle", "nsf_idle.png", gfx).unwrap();
        assets.load_texture("nsf_walk", "nsf_walk.png", gfx).unwrap();
        assets.load_texture("jinrai_idle", "jinrai_idle.png", gfx).unwrap();
        assets.load_texture("jinrai_walk", "jinrai_walk.png", gfx).unwrap();
        assets.load_texture("blob_shadow", "blob_shadow.png", gfx).unwrap();

        let sakura = assets.load_animation("sakura", "sakura.ron").unwrap();
        let jinrai = assets.load_animation("jinrai", "jinrai.ron").unwrap();
        let nsf = assets.load_animation("nsf", "nsf.ron").unwrap();

        let mut actors = Vec::<Actor>::new();

        for _ in 0..32 {
            let (x, y): (f32, f32) = rand::thread_rng().gen();
            actors.push(Actor::new(
                Point3::new(x * 32., y * 32., 0.0),
                sakura.clone(),
            ));
        }

        let mut player = Actor::new(Point3::new(16., 16., 0.), nsf);
        player.is_local_player = true;
        actors.push(player);

        Self {
            camera,
            actors,
            aim_point: None,
        }
    }

    fn get_aim_point(&self, ray: &Ray) -> Option<Point3> {
        let plane = Plane::new(Point3::origin(), -Vector3::unit_z());
        ray_plane_intersection(&ray, &plane, CAMERA_DISTANCE * 2.)
            .map(|distance| ray.start + ray.direction * distance)
    }

    fn get_player_commands(&self, ctx: &Context) -> Commands {
        let input = &ctx.input;
        let mut actions = HashSet::new();
        let aim_ray = self.camera.screen_to_ray(input.mouse_pos());

        let mut wish_dir = Vector3::zero();
        if input.is_key_down(Action::Forward) {
            wish_dir += Vector3::unit_y();
        }
        if input.is_key_down(Action::Back) {
            wish_dir -= Vector3::unit_y();
        }
        if input.is_key_down(Action::Right) {
            wish_dir += Vector3::unit_x();
        }
        if input.is_key_down(Action::Left) {
            wish_dir -= Vector3::unit_x();
        }
        if wish_dir.magnitude() > 1. {
            wish_dir = wish_dir.normalize();
        }

        if input.is_key_down(Action::Jump) || input.is_key_pressed(Action::Jump) {
            actions.insert(Action::Jump);
        }

        Commands {
            actions,
            aim_ray,
            wish_dir,
        }
    }
}

impl State for GameState {
    fn update(&mut self, assets: &Assets, ctx: &mut Context) {
        let commands = self.get_player_commands(ctx);
        self.aim_point = self.get_aim_point(&commands.aim_ray);
        let camera = &mut self.camera;
        let aim_point = &self.aim_point;
        self.actors.iter_mut().for_each(|a| {
            if a.is_local_player {
                a.player_move(&commands, ctx);

                if let Some(point) = aim_point {
                    let camera_offset = camera.eye - camera.target;
                    camera.target = a.position + (point - a.position) / 6.0;
                    camera.target.z = 0.;
                    camera.eye = camera.target + camera_offset;
                }
            }
            a.update(ctx)
        });

        // Camera has probably moved so update aim_point again to
        // eliminate "crosshair" lag
        let ray = self.camera.screen_to_ray(ctx.input.mouse_pos());
        self.aim_point = self.get_aim_point(&ray);
    }

    fn draw(&self, assets: &Assets, gfx: &mut Graphics) {
        let ground_texture = assets.get_texture("grass").unwrap();

        for x in 0..32 {
            for y in 0..32 {
                gfx.draw_plane(
                    &ground_texture,
                    Point3::new(x as f32, y as f32, 0.0),
                    1.0,
                    WHITE.into(),
                );
            }
        }

        self.actors.iter().for_each(|f| f.draw(&self.camera, assets, gfx));

        let (point, color) = match self.aim_point {
            Some(point) => (point, (1., 1., 1., 0.5).into()),
            None => (Point3::origin(), RED.into()),
        };
        gfx.draw_debug_cube(point, (0.25, 0.25, 0.25).into(), color);

        // gfx.draw_debug_grid(Point3::new(0.0, 0.0, 0.25), 20);

        gfx.flush(&self.camera);
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.camera.screen_size = Vector2::new(new_size.width as f32, new_size.height as f32);
    }
}
