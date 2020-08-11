use winit::{
    event::*,
    window::Window,
};

use crate::assets;
use crate::graphics;
use crate::state;
use crate::input;
use crate::config;

use std::rc::Rc;

pub struct Context {
    pub game_time: f64,
    pub delta_time: f32,
    pub input: input::Input,
    pub config: config::Config,
    window: Rc<Window>,
}

impl Context {
    pub fn new(window: Rc<Window>, config: config::Config) -> Self {
        let input = input::Input::new(&config);
        Self {
            game_time: 0.,
            delta_time: 0.,
            input,
            config,
            window,
        }
    }

    pub fn set_cursor_grab(&self, grab: bool) {
        self.window.set_cursor_grab(grab).unwrap_or_else(|_| println!("Cursor grab not supported"));
        self.window.set_cursor_visible(!grab);
    }
}

pub struct MainContext {
    gfx: graphics::Graphics,
    state: Box<dyn state::State>,
    ctx: Context,
    assets: assets::Assets,
}

impl MainContext {
    pub async fn new(window: Rc<Window>, config: config::Config) -> Self {
        let gfx = graphics::Graphics::new(&window, &config).await;
        let ctx = Context::new(window, config);

        let mut assets = assets::Assets::new();
        let state = Box::new(state::game::GameState::new(&mut assets, &ctx,  &gfx));
        Self {
            gfx,
            state,
            ctx,
            assets,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.gfx.resize(new_size);
        self.state.resize(new_size);
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input: KeyboardInput {
                    virtual_keycode: Some(virtual_code),
                    state,
                    ..
                },
                is_synthetic: false,
                ..
            } => match state {
                ElementState::Pressed => {
                    self.state.handle_key_down(*virtual_code) || {
                        self.ctx.input.handle_key_down(*virtual_code);
                        false
                    }
                }
                ElementState::Released => {
                    self.state.handle_key_up(*virtual_code) || {
                        self.ctx.input.handle_key_up(*virtual_code);
                        false
                    }
                }
            }
            WindowEvent::MouseInput {
                button,
                state,
                ..
            } => match state {
                ElementState::Pressed => {
                    self.state.handle_mouse_down(*button) || {
                        self.ctx.input.handle_mouse_down(*button);
                        false
                    }
                }
                ElementState::Released => {
                    self.state.handle_mouse_up(*button) || {
                        self.ctx.input.handle_mouse_up(*button);
                        false
                    }
                }
            }
            WindowEvent::MouseWheel {
                delta,
                ..
            } => {
                self.state.handle_mouse_scroll(*delta) || {
                    self.ctx.input.handle_mouse_scroll(*delta);
                    false
                }
            }
            WindowEvent::CursorMoved {
                position,
                ..
            } => {
                self.state.handle_mouse_move(*position) || {
                    self.ctx.input.handle_mouse_move(*position);
                    false
                }
            }
            _ => false
       }
    }

    pub fn update(&mut self, game_time: f64, delta_time: f64) {
        self.ctx.game_time = game_time;
        self.ctx.delta_time = delta_time as f32;

        self.state.update(&self.assets, &mut self.ctx);

        self.ctx.input.clear();
    }

    pub fn draw(&mut self) {
        self.state.draw(&self.assets, &mut self.gfx);
    }
}

