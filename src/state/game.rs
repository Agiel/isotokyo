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
    cursor_grab: bool,
    toggle_cursor: bool,
}

impl GameState {
    pub fn new(assets: &mut Assets, ctx: &Context, gfx: &mut Graphics) -> Self {
        ctx.set_cursor_grab(true);

        let view =
            Matrix4::from_angle_z(cgmath::Deg(45.0)) * Matrix4::from_angle_x(cgmath::Deg(-30.0));

        let screen_size = Vector2::new(gfx.extent.width as f32, gfx.extent.height as f32);
        let mut camera = Camera {
            // projection: Projection::Perspective,
            eye: view.transform_point(Point3::origin() + Vector3::unit_y() * -CAMERA_DISTANCE),
            target: Point3::origin(),
            zfar: CAMERA_DISTANCE * 2.0,
            screen_size,
            ..Default::default()
        };
        camera.build_view_projection_matrix();

        #[rustfmt::skip]
        assets.load_texture("grass", "grass1.png", gfx).unwrap();
        assets.load_texture("sakura", "sakura1.png", gfx).unwrap();
        assets.load_texture("nsf_idle", "nsf_idle.png", gfx).unwrap();
        assets.load_texture("nsf_walk", "nsf_walk.png", gfx).unwrap();
        assets.load_texture("jinrai_idle", "jinrai_idle.png", gfx).unwrap();
        assets.load_texture("jinrai_walk", "jinrai_walk.png", gfx).unwrap();
        assets.load_texture("blob_shadow", "blob_shadow.png", gfx).unwrap();
        assets.load_font("x-scale", "X-SCALE_.TTF", gfx).unwrap();

        let sakura = assets.load_animation("sakura", "sakura.ron").unwrap();
        let jinrai = assets.load_animation("jinrai", "jinrai.ron").unwrap();
        let nsf = assets.load_animation("nsf", "nsf.ron").unwrap();

        let mut actors = Vec::<Actor>::new();

        for _ in 0..256 {
            let (x, y): (f32, f32) = rand::thread_rng().gen();
            actors.push(Actor::new(
                Point3::new(x * 128., y * 128., 0.0),
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
            cursor_grab: true,
            toggle_cursor: false,
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
                    camera.build_view_projection_matrix();
                }
            }
            a.update(ctx)
        });

        // Camera has probably moved so update aim_point again to
        // eliminate "crosshair" lag
        let ray = self.camera.screen_to_ray(ctx.input.mouse_pos());
        self.aim_point = self.get_aim_point(&ray);

        if self.toggle_cursor {
            self.toggle_cursor = false;
            self.cursor_grab = !self.cursor_grab;
            ctx.set_cursor_grab(self.cursor_grab);
        }
    }

    fn draw(&self, assets: &Assets, ctx: &Context, gfx: &mut Graphics) {
        let ground_texture = assets.get_texture("grass").unwrap();
        let font = assets.get_font("x-scale").unwrap();

        let mat = self.camera.matrix;
        let screen_size = self.camera.screen_size;

        let mut num_tiles = 0;
        for x in 0..128 {
            for y in 0..128 {
                use std::f32::consts::SQRT_2;
                let center = Point3::new(x as f32, y as f32, 0.0);
                let margin = Vector2::new(
                    64. * SQRT_2 / screen_size.x,
                    64. * SQRT_2 / screen_size.y);
                if !is_world_point_inside_screen(mat, center, margin) {
                    continue;
                }
                num_tiles = num_tiles + 1;
                gfx.draw_plane(
                    &ground_texture,
                    center,
                    1.0,
                    WHITE.into(),
                );
            }
        }
        gfx.draw_text(&format!("Tiles: {}", num_tiles), font, 24., (8., 16.).into(), WHITE);

        let mut num_actors = 0;
        self.actors.iter().for_each(|a| {
            if a.is_local_player {
                gfx.draw_text(
                    &format!("Speed: {:.2}", a.velocity.magnitude()),
                    font,
                    24.,
                    (8., 48.).into(),
                    WHITE
                );
            }
            if a.draw(&self.camera, assets, gfx) {
                num_actors = num_actors + 1;
            }
        });
        gfx.draw_text(&format!("Actors: {}", num_actors), font, 24., (8., 32.).into(), WHITE);

        let (point, color) = match self.aim_point {
            Some(point) => (point, (1., 1., 1., 0.5).into()),
            None => (Point3::origin(), RED.into()),
        };
        gfx.draw_debug_cube(point, (0.25, 0.25, 0.25).into(), color);

        // gfx.draw_debug_grid(Point3::new(0.0, 0.0, 0.25), 20);

        gfx.draw_text(
            &format!("fps: {:.2}", 1. / ctx.delta_time),
            font,
            24.,
            (8., 0.).into(),
            WHITE
        );

        gfx.flush(&self.camera);
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.camera.screen_size = Vector2::new(new_size.width as f32, new_size.height as f32);
    }

    fn handle_key_down(&mut self, virtual_keycode: winit::event::VirtualKeyCode) -> bool {
        if virtual_keycode == winit::event::VirtualKeyCode::P {
            self.camera.projection = match self.camera.projection {
                Projection::Orthographic => {
                    self.camera.zfar = 1000.0;
                    Projection::Perspective
                }
                Projection::Perspective => {
                    self.camera.zfar = CAMERA_DISTANCE * 2.;
                    Projection::Orthographic
                }
            }
        }

        if virtual_keycode == winit::event::VirtualKeyCode::O {
            self.toggle_cursor = true;
        }

        false
    }
}
