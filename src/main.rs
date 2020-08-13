use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use std::time::{Duration, SystemTime};

mod camera;
mod graphics;
mod utils;
mod state;
mod context;
mod input;
mod config;
mod assets;

fn main() {
    let event_loop = EventLoop::new();
    let config = config::Config::new();
    let window = std::rc::Rc::new(WindowBuilder::new()
        .with_title("Isotokyo")
        .with_resizable(false)
        .with_inner_size(winit::dpi::PhysicalSize::new(config.graphics.resolution.0 as i32, config.graphics.resolution.1 as i32))
        .build(&event_loop)
        .unwrap()
    );

    let target_frame_time = if config.graphics.target_fps > 0 {
        Some(Duration::from_secs_f64(1.0 / config.graphics.target_fps as f64))
    } else {
        None
    };

    use futures::executor::block_on;

    // Since main can't be async, we're going to need to block
    let mut ctx = block_on(context::MainContext::new(window.clone(), config));

    let mut begin_frame = SystemTime::now();
    let mut game_time = 0.0;
    let mut frame_time = 0.0;

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
                // Update
                game_time += frame_time;
                ctx.update(game_time, frame_time);

                // Draw
                ctx.draw();

                // Sleep to match target_fps
                if let Some(target_frame_time) = target_frame_time {
                    let actual_frame_time = begin_frame.elapsed().unwrap();
                    if frame_time > 0. && actual_frame_time.as_secs_f64() < target_frame_time.as_secs_f64() {
                        if std::env::consts::OS == "windows" {
                            while begin_frame.elapsed().unwrap().as_secs_f64() < target_frame_time.as_secs_f64() {
                                std::thread::sleep(Duration::from_secs(0));
                            }
                        } else {
                            let sleep_time = target_frame_time - actual_frame_time;
                            std::thread::sleep(sleep_time);
                        }
                    }
                }

                // Begin next frame (events -> update -> draw -> sleep)
                frame_time = begin_frame.elapsed().unwrap().as_secs_f64();
                begin_frame = SystemTime::now();
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
