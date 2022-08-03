use bevy::{prelude::*, utils::HashMap};
use bevy_rapier3d::plugin::{RapierConfiguration, TimestepMode};
use serde::{Deserialize, Serialize};

use crate::input::InputAction;

const CONFIG_PATH: &str = "config/config.ron";

pub struct ConfigPlugin;

impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(read_config);
    }
}

fn read_config(mut commands: Commands, mut physics_config: ResMut<RapierConfiguration>) {
    let config = Config::new();
    physics_config.gravity = -Vec3::Y * config.physics.gravity;
    physics_config.timestep_mode = TimestepMode::Interpolated {
        dt: 1.0 / 60.0,
        substeps: 1,
        time_scale: 1.0,
    };
    commands.insert_resource(config);
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub key_bindings: HashMap<KeyCode, Vec<InputAction>>,
    pub physics: PhysicsConfig,
}

#[derive(Serialize, Deserialize)]
pub struct PhysicsConfig {
    pub ground_speed: f32,
    pub air_speed: f32,
    pub ground_accel: f32,
    pub air_accel: f32,
    pub ground_friction: f32,
    pub air_friction: f32,
    pub gravity: f32,
    pub jump_height: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            key_bindings: HashMap::from_iter(vec![
                (KeyCode::W, vec![InputAction::Forward]),
                (KeyCode::S, vec![InputAction::Back]),
                (KeyCode::A, vec![InputAction::Left]),
                (KeyCode::D, vec![InputAction::Right]),
                (KeyCode::Space, vec![InputAction::Jump]),
            ]),
            physics: PhysicsConfig {
                ground_speed: 3.0,
                air_speed: 0.5,
                ground_accel: 10.0,
                air_accel: 1.0,
                ground_friction: 5.0,
                air_friction: 0.0,
                gravity: 12.0,
                jump_height: 0.5,
            },
        }
    }
}

impl Config {
    pub fn new() -> Self {
        match std::fs::read_to_string(CONFIG_PATH) {
            Ok(config) => ron::from_str(&config).unwrap_or_else(|err| {
                println!(
                    "Failed to parse config! Backing up and writing a new one.\n{}",
                    err
                );
                std::fs::copy(CONFIG_PATH, "config/config.old.ron").unwrap_or_else(|err| {
                    println!("Unable to backup old config!\n{}", err);
                    0
                });
                Self::write_default()
            }),
            _ => Self::write_default(),
        }
    }

    pub fn write(&self) -> Result<(), Box<dyn std::error::Error>> {
        let pretty = ron::ser::PrettyConfig::new().depth_limit(2);
        let config_str = ron::ser::to_string_pretty(self, pretty)?;
        std::fs::create_dir_all("config/")?;
        std::fs::write(CONFIG_PATH, config_str)?;
        Ok(())
    }

    fn write_default() -> Self {
        let config = Self::default();
        config.write().unwrap_or_else(|err| {
            println!("Failed to write config to '{}'!\n{}", CONFIG_PATH, err)
        });
        config
    }
}
