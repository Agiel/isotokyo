pub mod game;

pub trait State {
    fn update(&mut self, assets: &crate::assets::Assets, ctx: &crate::context::Context);
    fn draw(&self, assets: &crate::assets::Assets, gfx: &mut crate::graphics::Graphics);

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {}

    fn handle_key_down(&mut self, virtual_keycode: winit::event::VirtualKeyCode) -> bool { false }
    fn handle_key_up(&mut self, virtual_keycode: winit::event::VirtualKeyCode) -> bool { false }
    fn handle_mouse_move(&mut self, position: winit::dpi::PhysicalPosition<f64>) -> bool { false }
    fn handle_mouse_down(&mut self, button: winit::event::MouseButton) -> bool { false }
    fn handle_mouse_up(&mut self, button: winit::event::MouseButton) -> bool { false }
    fn handle_mouse_scroll(&mut self, delta: winit::event::MouseScrollDelta) -> bool { false }
}
