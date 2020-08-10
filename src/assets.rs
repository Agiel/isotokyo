pub mod animation;

use crate::graphics::{Graphics, texture::Texture};
use animation::Animations;

use std::sync::Arc;
use std::collections::HashMap;
use std::fs;
use std::error::Error;

pub struct Assets {
    textures: HashMap<String, Arc<Texture>>,
    animations: HashMap<String, Arc<Animations>>,
}

impl Assets {
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
            animations: HashMap::new(),
        }
    }

    pub fn load_texture(&mut self, name: &str, path: &str, gfx: &Graphics) -> Result<Arc<Texture>, Box<dyn Error>> {
        let texture_bytes = fs::read(&format!("resources/textures/{}", path))?;
        let texture = gfx.load_texture_bytes(texture_bytes.as_slice(), name)?;
        self.textures.insert(name.to_string(), texture.clone());
        Ok(texture)
    }

    pub fn get_texture(&self, name: &str) -> Option<Arc<Texture>> {
        self.textures.get(name).cloned()
    }

    pub fn load_animation(&mut self, name: &str, path: &str) -> Result<Arc<Animations>, Box<dyn Error>> {
        let animations_str = fs::read_to_string(&format!("resources/animations/{}", path))?;
        let animations: Animations = ron::from_str(&animations_str)?;
        let animations = Arc::new(animations);
        self.animations.insert(name.to_string(), animations.clone());
        Ok(animations)
    }

    pub fn get_animation(&self, name: &str) -> Option<Arc<Animations>> {
        self.animations.get(name).cloned()
    }
}
