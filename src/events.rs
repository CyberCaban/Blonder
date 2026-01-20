use glfw::{Action, Key, WindowEvent};

use crate::{
    render::framebuffer::ViewportScaleStrategy,
    state::{Events, State},
};

pub fn process_events(window: &mut glfw::Window, events: &Events, state: &mut State) {
    let light_pos_speed = 8.55;
    if matches!(window.get_key(glfw::Key::K), Action::Press | Action::Repeat) {
        state.light_pos.x += 0.1 * state.delta_time * light_pos_speed;
    }
    if matches!(window.get_key(glfw::Key::I), Action::Press | Action::Repeat) {
        state.light_pos.x -= 0.1 * state.delta_time * light_pos_speed;
    }
    if matches!(window.get_key(glfw::Key::L), Action::Press | Action::Repeat) {
        state.light_pos.z -= 0.1 * state.delta_time * light_pos_speed;
    }
    if matches!(window.get_key(glfw::Key::J), Action::Press | Action::Repeat) {
        state.light_pos.z += 0.1 * state.delta_time * light_pos_speed;
    }

    if matches!(window.get_key(glfw::Key::U), Action::Press | Action::Repeat) {
        state.light_pos.y += 0.1 * state.delta_time * light_pos_speed;
    }
    if matches!(window.get_key(glfw::Key::O), Action::Press | Action::Repeat) {
        state.light_pos.y -= 0.1 * state.delta_time * light_pos_speed;
    }

    for (msg, event) in glfw::flush_messages(events) {
        match event {
            WindowEvent::FileDrop(param) => {
                for p in param {
                    println!("{}", p.to_string_lossy());
                    state.model_path_to_load = Some(p.to_string_lossy().to_string());
                }
            }
            glfw::WindowEvent::FramebufferSize(width, height) => unsafe {
                state.screen.width = width as u32;
                state.screen.height = height as u32;
                state.window_size_changed = true;
                gl::Viewport(0, 0, width, height);
            },
            glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                window.set_should_close(true)
            }
            WindowEvent::Key(Key::Up, _, Action::Press | Action::Repeat, _) => {}
            WindowEvent::Key(Key::Down, _, Action::Press | Action::Repeat, _) => {}
            WindowEvent::Scroll(w, h) => {
                state.color.2 += (h * 0.01) as f32;
                state.color.2 = state.color.2.clamp(0.0, 1.0);
                println!("color BLUE {}", state.color.2);
            }
            WindowEvent::CursorPos(x, y) => {
                state.cursor_pos_x = x as f32;
                state.cursor_pos_y = (state.screen.height as f64 - y) as f32;
            }
            WindowEvent::MouseButton(glfw::MouseButtonLeft, Action::Press, _) => {
                state.mouse_pressed = true;
            }
            WindowEvent::MouseButton(glfw::MouseButtonLeft, Action::Release, _) => {
                state.mouse_pressed = false;
            }
            WindowEvent::Key(Key::F, _, Action::Press | Action::Repeat, _) => {
                dbg!(&state);
            }
            WindowEvent::Key(Key::G, _, Action::Press | Action::Repeat, _) => {
                state.is_lowres = !state.is_lowres;
            }
            WindowEvent::Key(Key::X, _, Action::Press | Action::Repeat, _) => {
                state.wireframe = !state.wireframe;
            }
            WindowEvent::Key(Key::R, _, Action::Press | Action::Repeat, _) => {
                state.scale_strategy = match state.scale_strategy {
                    ViewportScaleStrategy::Fit => ViewportScaleStrategy::PixelPerfect,
                    ViewportScaleStrategy::PixelPerfect => ViewportScaleStrategy::Stretch,
                    ViewportScaleStrategy::Stretch => ViewportScaleStrategy::Fit,
                }
            }
            WindowEvent::MouseButton(glfw::MouseButton::Middle, Action::Press, _) => {
                state.camera.process_capture(window);
            }
            _ => {}
        }
    }
}
