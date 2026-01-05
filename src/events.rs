use glfw::{Action, Key, WindowEvent};

use crate::state::{Events, State};

pub fn process_events(window: &mut glfw::Window, events: &Events, state: &mut State) {
    for (msg, event) in glfw::flush_messages(events) {
        match event {
            WindowEvent::FileDrop(param) => {
                for p in param {
                    println!("{}", p.to_string_lossy());
                }
                println!("Decrement color BLUE {}", state.color.2);
            }
            glfw::WindowEvent::FramebufferSize(width, height) => unsafe {
                state.screen.width = width as u32;
                state.screen.height = height as u32;
                gl::Viewport(0, 0, width, height);
            },
            glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                window.set_should_close(true)
            }

            WindowEvent::Key(Key::A, _, Action::Press, _) => {
                state.color.0 += 0.1;
                state.color.0 = state.color.0.clamp(0.0, 1.0);
                println!("Increment color RED {}", state.color.0);
            }
            WindowEvent::Key(Key::D, _, Action::Press, _) => {
                state.color.0 -= 0.1;
                state.color.0 = state.color.0.clamp(0.0, 1.0);
                println!("Decrement color RED {}", state.color.0);
            }
            WindowEvent::Key(Key::W, _, Action::Press, _) => {
                state.color.1 += 0.1;
                state.color.1 = state.color.1.clamp(0.0, 1.0);
                println!("Increment color GREEN {}", state.color.1);
            }
            WindowEvent::Key(Key::S, _, Action::Press, _) => {
                state.color.1 -= 0.1;
                state.color.1 = state.color.1.clamp(0.0, 1.0);
                println!("Decrement color GREEN {}", state.color.1);
            }
            WindowEvent::Key(Key::Up, _, Action::Press | Action::Repeat, _) => {
                println!("{:?}", state.transform_matrix);
                // state.transform_matrix =
                //     state.transform_matrix + Mat4::from_translation(Vec3::unit_y() * 0.1);
                println!("Increment param {:?}", state.transform_matrix.y);
            }
            WindowEvent::Key(Key::Down, _, Action::Press | Action::Repeat, _) => {
                println!("{:?}", state.transform_matrix);
                // state.transform_matrix =
                //     state.transform_matrix - Mat4::from_translation(Vec3::unit_y() * 0.1);
                println!("Decrement param {:?}", state.transform_matrix.y);
            }
            WindowEvent::Scroll(w, h) => {
                state.color.2 += (h * 0.01) as f32;
                state.color.2 = state.color.2.clamp(0.0, 1.0);
                println!("color BLUE {}", state.color.2);
            }
            WindowEvent::Key(Key::Space, _, Action::Press | Action::Repeat, _) => {
                state.wireframe = !state.wireframe;
                println!("Pressed space")
            }
            WindowEvent::Key(Key::X, _, Action::Press, _) => {
                println!("{:?}", state);
            }
            _ => {}
        }
    }
}
