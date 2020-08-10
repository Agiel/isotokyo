use crate::utils::*;
use crate::config;

use cgmath::prelude::*;
use serde::{Deserialize, Serialize};
use winit::event::*;

use std::collections::{HashMap, HashSet};

#[derive(Eq, PartialEq, Hash, Copy, Clone, Serialize, Deserialize)]
pub enum Action {
    Forward,
    Back,
    Left,
    Right,
    Jump,
    Crouch,
    Attack,
    Reload,
    Use,
    Drop,
    Cloak,
    Buy,
    Score,
    Ok,
    Cancel,
    Say,
    SayTeam,
    SayMode,
    Beacon,
    Primary,
    Secondary,
    Melee,
    Grenade,
    Next,
    Previous,
    LastEquip,
}

#[derive(Eq, PartialEq, Hash, Copy, Clone, Serialize, Deserialize)]
pub enum Mouse {
    Left,
    Middle,
    Right,
    Button(u8),
    ScrollUp,
    ScrollDown,
    ScrollLeft,
    ScrollRight,
}

pub struct Input {
    key_bindings: HashMap<VirtualKeyCode, Vec<Action>>,
    mouse_bindings: HashMap<Mouse, Vec<Action>>,
    keys_pressed: HashSet<Action>,
    keys_released: HashSet<Action>,
    keys_down: HashSet<Action>,
    release_next: HashSet<Action>,
    mouse_pos: Point2,
}

impl Input {
    pub fn new(config: &config::Config) -> Self {
        Self {
            key_bindings: config.key_bindings.clone(),
            mouse_bindings: config.mouse_bindings.clone(),
            keys_pressed: HashSet::new(),
            keys_released: HashSet::new(),
            keys_down: HashSet::new(),
            release_next: HashSet::new(),
            mouse_pos: Point2::origin(),
        }
    }

    pub fn is_key_down(&self, action: Action) -> bool {
        self.keys_down.contains(&action)
    }

    pub fn is_key_pressed(&self, action: Action) -> bool {
        self.keys_pressed.contains(&action)
    }

    pub fn is_key_released(&self, action: Action) -> bool {
        self.keys_released.contains(&action)
    }

    pub fn mouse_pos(&self) -> Point2 {
        self.mouse_pos
    }

    pub fn clear(&mut self) {
        self.keys_pressed.clear();
        self.keys_released.clear();

        self.keys_released.extend(self.release_next.iter());
        self.release_next.clear();
    }

    pub fn handle_key_down(&mut self, virtual_keycode: winit::event::VirtualKeyCode) {
        if let Some(action) = self.key_bindings.get(&virtual_keycode) {
            self.keys_pressed.extend(action.iter());
            self.keys_down.extend(action.iter());
        }
    }

    pub fn handle_key_up(&mut self, virtual_keycode: winit::event::VirtualKeyCode) {
        if let Some(action) = self.key_bindings.get(&virtual_keycode) {
            self.keys_released.extend(action.iter());
            let keys_down = &mut self.keys_down;
            action.iter().for_each(|a| {
                keys_down.remove(a);
            });
        }
    }

    pub fn handle_mouse_move(&mut self, position: winit::dpi::PhysicalPosition<f64>) {
        self.mouse_pos = Point2::new(position.x as f32, position.y as f32);
    }

    pub fn handle_mouse_down(&mut self, button: winit::event::MouseButton) {
        let button = match button {
            MouseButton::Left => Mouse::Left,
            MouseButton::Middle => Mouse::Middle,
            MouseButton::Right => Mouse::Right,
            MouseButton::Other(n) => Mouse::Button(n),
        };
        if let Some(action) = self.mouse_bindings.get(&button) {
            self.keys_pressed.extend(action);
            self.keys_down.extend(action);
        }
    }

    pub fn handle_mouse_up(&mut self, button: winit::event::MouseButton) {
        let button = match button {
            MouseButton::Left => Mouse::Left,
            MouseButton::Middle => Mouse::Middle,
            MouseButton::Right => Mouse::Right,
            MouseButton::Other(n) => Mouse::Button(n),
        };
        if let Some(action) = self.mouse_bindings.get(&button) {
            self.keys_released.extend(action);
            let keys_down = &mut self.keys_down;
            action.iter().for_each(|a| {
                keys_down.remove(a);
            });
        }
    }

    pub fn handle_mouse_scroll(&mut self, delta: winit::event::MouseScrollDelta) {
        if let MouseScrollDelta::LineDelta(x, y) = delta {
            if let Some(button) = if y > 0. {
                Some(Mouse::ScrollDown)
            } else if y < 0. {
                Some(Mouse::ScrollUp)
            } else if x > 0. {
                Some(Mouse::ScrollRight)
            } else if x < 0. {
                Some(Mouse::ScrollLeft)
            } else {
                None
            } {
                if let Some(action) = self.mouse_bindings.get(&button) {
                    self.keys_pressed.extend(action);
                    self.keys_down.extend(action);
                    // Mouse wheel actions are held for one frame
                    self.release_next.extend(action);
                }
            }
        }
    }

    pub fn force_release_key(&mut self, action: Action) {
        self.release_next.insert(action);
    }
}
