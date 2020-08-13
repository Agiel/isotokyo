use crate::context::Context;
use crate::input::Action;
use crate::state::game::{Actor, Commands};
use crate::utils::*;

use cgmath::prelude::*;

impl Actor {
    pub fn player_move(&mut self, player_cmd: &Commands, ctx: &mut Context) {
        self.check_ground(ctx);

        if self.is_grounded && player_cmd.actions.contains(&Action::Jump) {
            if self.is_local_player {
                // Only one jump per press
                ctx.input.force_release_key(Action::Jump);
            }
            self.jump(ctx);
        }

        self.friction(ctx);

        let ground_plane = Plane::new(Point3::origin(), Vector3::unit_z());
        if let Some(distance) =
            ray_plane_intersection(&player_cmd.aim_ray, &ground_plane, CAMERA_DISTANCE * 2.)
        {
            let mut aim_point = player_cmd.aim_ray.start + player_cmd.aim_ray.direction * distance;
            aim_point.z = self.position.z;
            self.orientation = (aim_point - self.position).normalize();
        }
        let right = self.orientation.cross(Vector3::unit_z()) * player_cmd.wish_dir.x;
        let forward = self.orientation * player_cmd.wish_dir.y;

        let wish_dir = right + forward;
        let wish_speed = ctx.config.physics.walk_speed;

        self.accelerate(wish_dir, wish_speed, ctx);

        if !self.is_grounded {
            self.velocity -= Vector3::unit_z() * ctx.config.physics.gravity * ctx.delta_time;
        }

        self.position += self.velocity * ctx.delta_time;
    }

    fn check_ground(&mut self, ctx: &Context) {
        if self.velocity.z > 0. {
            return;
        }

        let player_ray = Ray::new(self.position + Vector3::unit_z() * 0.01, -Vector3::unit_z());
        let ground_plane = Plane::new(Point3::origin(), Vector3::unit_z());
        self.is_grounded = if let Some(_distance) = ray_plane_intersection(
            &player_ray,
            &ground_plane,
            -self.velocity.z * ctx.delta_time + 0.01,
        ) {
            self.velocity.z = 0.;
            self.position.z = ground_plane.point.z;
            true
        } else {
            false
        };
    }

    fn friction(&mut self, ctx: &Context) {
        let current_speed = self.velocity.magnitude();

        if current_speed == 0. {
            return;
        }

        let friction = if self.is_grounded {
            ctx.config.physics.ground_friction
        } else {
            ctx.config.physics.air_friction
        };

        // TODO: Use stop_speed instead of walk_speed?
        let drop = current_speed.max(ctx.config.physics.walk_speed) * friction * ctx.delta_time;
        let new_speed = (current_speed - drop).max(0.);
        self.velocity *= new_speed / current_speed;
    }

    fn jump(&mut self, ctx: &Context) {
        if self.is_grounded {
            self.velocity += Vector3::unit_z()
                * (2. * ctx.config.physics.gravity * ctx.config.physics.jump_height).sqrt();
            self.is_grounded = false;
        }
    }

    fn accelerate(&mut self, wish_dir: Vector3, wish_speed: f32, ctx: &Context) {
        let wsh_speed = if !self.is_grounded {
            ctx.config.physics.air_speed
        } else {
            wish_speed
        };
        let current_speed = self.velocity.dot(wish_dir);
        let add_speed = wsh_speed - current_speed;
        if add_speed <= 0. {
            return;
        }

        let accel = if self.is_grounded {
            ctx.config.physics.ground_accel
        } else {
            ctx.config.physics.air_accel
        };

        let accel_speed = add_speed.min(accel * wish_speed * ctx.delta_time);

        self.velocity += wish_dir * accel_speed;
    }
}
