use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

mod camera;
mod graphics;
mod utils;
mod state;
mod context;
mod input;
mod config;

const TIME_STEP: f64 = 1.0 / 60.0;

fn main() {
    let event_loop = EventLoop::new();
    let window = std::rc::Rc::new(WindowBuilder::new()
        .with_title("Isotokyo")
        .with_resizable(false)
        .with_inner_size(winit::dpi::PhysicalSize::new(1280, 720))
        .build(&event_loop)
        .unwrap()
    );

    use futures::executor::block_on;

    // Since main can't be async, we're going to need to block
    let mut ctx = block_on(context::MainContext::new(window.clone()));

    let mut current_time = std::time::SystemTime::now();
    let mut game_time: f64 = 0.0;
    let mut accumulator: f64 = 0.0;

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                if !ctx.input(event) {
                    match event {
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        WindowEvent::KeyboardInput { input, .. } => match input {
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            } => *control_flow = ControlFlow::Exit,
                            _ => {}
                        },
                        WindowEvent::Resized(physical_size) => {
                            ctx.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            // new_inner_size is &mut so we have to dereference it twice
                            ctx.resize(**new_inner_size);
                        }
                        _ => {}
                    }
                }
            }
            Event::RedrawRequested(_) => {
                let frame_time = current_time.elapsed().unwrap().as_secs_f64();
                current_time = std::time::SystemTime::now();
                
                accumulator += frame_time;

                while accumulator >= TIME_STEP {
                    ctx.update(game_time, TIME_STEP);
                    game_time += TIME_STEP;
                    accumulator -= TIME_STEP;
                }
                ctx.draw();
            }
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                window.request_redraw();
            }
            _ => {}
        }
    });

}
