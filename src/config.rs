use crate::input::{Action, Mouse};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::iter::FromIterator;
use winit::event::VirtualKeyCode;

const CONFIG_PATH: &str = "config/config.ron";

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub key_bindings: HashMap<VirtualKeyCode, Vec<Action>>,
    pub mouse_bindings: HashMap<Mouse, Vec<Action>>,
    pub physics: PhysicsConfig,
}

#[derive(Serialize, Deserialize)]
pub struct PhysicsConfig {
    walk_speed: f32,
    crouch_speed: f32,
    ground_accel: f32,
    air_accel: f32,
    ground_friction: f32,
    air_friction: f32,
    gravity: f32,
    jump_height: f32,
}

impl Config {
    pub fn new() -> Self {
        match std::fs::read_to_string(CONFIG_PATH) {
            Ok(config) => ron::from_str(&config).unwrap_or_else(|err| {
                println!("Failed to parse config! Backing up and writing a new one.\n{}", err);
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
        let pretty = ron::ser::PrettyConfig::new().with_depth_limit(2);
        let config_str = ron::ser::to_string_pretty(self, pretty)?;
        std::fs::create_dir_all("config/")?;
        std::fs::write(CONFIG_PATH, config_str)?;
        Ok(())
    }

    fn write_default() -> Self {
        let config: Self = Default::default();
        config.write().unwrap_or_else(|err| {
            println!("Failed to write config to '{}'!\n{}", CONFIG_PATH, err)
        });
        config
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            key_bindings: HashMap::from_iter(vec![
                (VirtualKeyCode::W, vec![Action::Forward]),
                (VirtualKeyCode::A, vec![Action::Left]),
                (VirtualKeyCode::S, vec![Action::Back]),
                (VirtualKeyCode::D, vec![Action::Right]),
                (VirtualKeyCode::Space, vec![Action::Jump]),
                (VirtualKeyCode::LControl, vec![Action::Crouch]),
                (VirtualKeyCode::R, vec![Action::Reload]),
                (VirtualKeyCode::E, vec![Action::Use]),
                (VirtualKeyCode::G, vec![Action::Drop]),
                (VirtualKeyCode::C, vec![Action::Cloak]),
                (VirtualKeyCode::B, vec![Action::Buy]),
                (VirtualKeyCode::Tab, vec![Action::Score, Action::SayMode]),
                (VirtualKeyCode::Return, vec![Action::Ok, Action::Say]),
                (VirtualKeyCode::Escape, vec![Action::Cancel]),
                (VirtualKeyCode::T, vec![Action::Say]),
                (VirtualKeyCode::Y, vec![Action::SayTeam]),
                (VirtualKeyCode::Key1, vec![Action::Primary]),
                (VirtualKeyCode::Key2, vec![Action::Secondary]),
                (VirtualKeyCode::Key3, vec![Action::Melee]),
                (VirtualKeyCode::Key4, vec![Action::Grenade]),
                (VirtualKeyCode::Q, vec![Action::LastEquip]),
            ]),
            mouse_bindings: HashMap::from_iter(vec![
                (Mouse::Left, vec![Action::Attack]),
                (Mouse::Right, vec![Action::Jump]),
                (Mouse::Middle, vec![Action::Beacon]),
                (Mouse::ScrollUp, vec![Action::Previous]),
                (Mouse::ScrollDown, vec![Action::Next]),
            ]),
            physics: PhysicsConfig {
                walk_speed: 3.0,
                crouch_speed: 1.0,
                ground_accel: 10.0,
                air_accel: 1.0,
                ground_friction: 8.0,
                air_friction: 1.0,
                gravity: 12.0,
                jump_height: 0.5,
            },
        }
    }
}
