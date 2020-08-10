use serde::Deserialize;

use std::collections::HashMap;

#[derive(Eq, PartialEq, Hash, Deserialize)]
pub enum Sequence {
    Idle,
    Walk,
    Jump,
}

#[derive(Deserialize)]
pub enum Directions {
    Column,
    Row,
    None,
}

#[derive(Deserialize)]
pub struct SequenceDef {
    pub texture: String,
    pub offset: (u32, u32),
    pub size: (u32, u32),
    pub length: u32,
    pub speed: f32,
    pub directions: Directions,
}

pub type Animations = HashMap<Sequence, SequenceDef>;
