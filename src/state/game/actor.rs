use crate::assets::animation::*;
use crate::assets::Assets;
use crate::camera::Camera;
use crate::context::Context;
use crate::graphics::Graphics;
use crate::input::Action;
use crate::utils::*;

use cgmath::prelude::*;

use std::sync::Arc;

struct Animator {
    animations: Arc<Animations>,
    sequence: Sequence,
    next_frame: f64,
    current_frame: u32,
}

impl Animator {
    fn new(animations: Arc<Animations>) -> Self {
        Self {
            animations,
            sequence: Sequence::Idle,
            next_frame: 0.,
            current_frame: 0,
        }
    }

    fn update(&mut self, game_time: f64) {
        if let Some(animation) = self.animations.get(&self.sequence) {
            if animation.length > 0 && game_time >= self.next_frame {
                if self.next_frame == 0. {
                    // Sequence just started
                    self.next_frame = game_time + animation.speed as f64;
                } else {
                    self.current_frame = (self.current_frame + 1) % animation.length;
                    self.next_frame += animation.speed as f64;
                }
            }
        } else {
            // Invalid sequence, default to Idle
            self.set_sequence(Sequence::Idle);
        }
    }

    fn set_sequence(&mut self, sequence: Sequence) {
        if sequence == self.sequence {
            return;
        }

        self.sequence = sequence;
        self.current_frame = 0;
        self.next_frame = 0.;
    }

    fn rad_to_dir(radians: cgmath::Rad<f32>) -> u32 {
        use std::f32::consts::PI;
        let frac = radians.0 / (2. * PI);
        ((1.0625 + frac) * 8.0) as u32 % 8
    }

    fn get_rect(&self, angle: cgmath::Rad<f32>) -> Rect {
        if let Some(animation) = self.animations.get(&self.sequence) {
            let offset = match animation.directions {
                Directions::Column => {
                    let direction = Self::rad_to_dir(angle);
                    (self.current_frame, direction)
                }
                Directions::Row => {
                    let direction = Self::rad_to_dir(angle);
                    (self.current_frame + direction, 0)
                }
                Directions::None => (0, 0),
            };
            Rect::new(
                (animation.offset.0 + offset.0 * animation.size.0) as f32,
                (animation.offset.1 + offset.1 * animation.size.1) as f32,
                animation.size.0 as f32,
                animation.size.1 as f32,
            )
        } else {
            Rect::new(0., 0., 64., 64.)
        }
    }

    fn get_texture(&self) -> &str {
        if let Some(animation) = self.animations.get(&self.sequence) {
            &animation.texture
        } else {
            "error"
        }
    }
}

// TODO: Should probably use a proper ECS architecture, but for now we can use a basic Actor setup.
pub struct Actor {
    pub position: Point3,
    pub orientation: Vector2,
    pub velocity: Vector3,
    animator: Animator,
    pub is_local_player: bool,
}

impl Actor {
    pub fn new(position: Point3, animations: Arc<Animations>) -> Self {
        use std::f32::consts::FRAC_1_SQRT_2;
        Self {
            position,
            orientation: Vector2::new(-FRAC_1_SQRT_2, -FRAC_1_SQRT_2),
            velocity: Vector3::zero(),
            animator: Animator::new(animations),
            is_local_player: false,
        }
    }

    pub fn update(&mut self, ctx: &Context) {
        if self.is_local_player {
            if ctx.input.is_key_down(Action::Jump) {
                self.animator.set_sequence(Sequence::Jump);
            } else if ctx.input.is_key_down(Action::Forward) {
                self.animator.set_sequence(Sequence::Walk);
            } else {
                self.animator.set_sequence(Sequence::Idle);
            }
        }

        self.animator.update(ctx.game_time);
    }

    pub fn draw(&self, camera: &Camera, assets: &Assets, gfx: &mut Graphics) {
        // Draw sprite
        if let Some(texture) = assets.get_texture(self.animator.get_texture()) {
            let forward = camera.target - camera.eye;
            let forward = Vector2::new(forward.x, forward.y).normalize();
            let angle = self.orientation.angle(forward);

            let source = self.animator.get_rect(angle);
            let size = source.size / PIXELS_PER_UNIT;
            gfx.draw_billboard(
                camera,
                &texture,
                source,
                self.position,
                size,
                Vector3::new(0.0, size.y / 2. - 0.5, 0.5),
            );
        } else {
            // Error
            gfx.draw_debug_cube(self.position, (1., 1., 1.).into(), RED.into());
        }

        // Draw shadow
        if let Some(shadow_texture) = assets.get_texture("blob_shadow") {
            // Offset position to avoid ray missing the ground
            let position = self.position + Vector3::unit_z() * 0.1;
            let ray = Ray::new(position, -Vector3::unit_z());
            let plane = Plane::new(Point3::origin(), Vector3::unit_z());

            if let Some(distance) = ray_plane_intersection(&ray, &plane, 1.0) {
                let shadow_strength = 0.6 + 0.4 * (1.0 - distance);
                gfx.draw_plane(
                    &shadow_texture,
                    Point3::new(position.x, position.y, position.z - distance + 0.01),
                    1.0,
                    (0.0, 0.0, 0.0, shadow_strength).into(),
                );
            }
        }
    }
}
