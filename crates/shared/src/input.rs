use bevy::{
    input::{keyboard::KeyboardInput, ButtonState},
    prelude::*,
};
use serde::{Deserialize, Serialize};

use crate::config::Config;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Input<InputAction>>()
            .add_systems(PreUpdate, keyboard_input_system);
    }
}

#[derive(PartialEq, Eq, Hash, Copy, Clone, Serialize, Deserialize)]
pub enum InputAction {
    Forward,
    Back,
    Left,
    Right,
    Jump,
}

fn keyboard_input_system(
    mut input: ResMut<Input<InputAction>>,
    mut keyboard_input_events: EventReader<KeyboardInput>,
    config: Res<Config>,
) {
    input.clear();
    for event in keyboard_input_events.read() {
        if let KeyboardInput {
            key_code: Some(key_code),
            state,
            ..
        } = event
        {
            let actions = config.key_bindings.get(key_code);
            match (state, actions) {
                (ButtonState::Pressed, Some(actions)) => {
                    actions.iter().for_each(|action| input.press(*action))
                }
                (ButtonState::Released, Some(actions)) => {
                    actions.iter().for_each(|action| input.release(*action))
                }
                _ => (),
            }
        }
    }
}
