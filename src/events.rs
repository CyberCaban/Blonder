use glfw::{Action, Key, WindowEvent};

use crate::state::{Events, State};

pub fn process_events(window: &mut glfw::Window, events: &Events, state: &mut State) {
    let State { color, .. } = state;
    for (msg, event) in glfw::flush_messages(events) {
        println!("Message: {}\nEvent: {:?}", msg, event);
        match event {
            WindowEvent::FileDrop(param) => {
                for p in param {
                    println!("{}", p.to_string_lossy());
                }
                println!("Decrement color BLUE {}", color.2);
            }
            glfw::WindowEvent::FramebufferSize(width, height) => unsafe {
                gl::Viewport(0, 0, width, height);
            },
            glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                window.set_should_close(true)
            }

            WindowEvent::Key(Key::A, _, Action::Press, _) => {
                color.0 += 0.1;
                color.0 = color.0.clamp(0.0, 1.0);
                println!("Increment color RED {}", color.0);
            }
            WindowEvent::Key(Key::D, _, Action::Press, _) => {
                color.0 -= 0.1;
                color.0 = color.0.clamp(0.0, 1.0);
                println!("Decrement color RED {}", color.0);
            }
            WindowEvent::Key(Key::W, _, Action::Press, _) => {
                color.1 += 0.1;
                color.1 = color.1.clamp(0.0, 1.0);
                println!("Increment color GREEN {}", color.1);
            }
            WindowEvent::Key(Key::S, _, Action::Press, _) => {
                color.1 -= 0.1;
                color.1 = color.1.clamp(0.0, 1.0);
                println!("Decrement color GREEN {}", color.1);
            }
            WindowEvent::Key(Key::Up, _, Action::Press | Action::Repeat, _) => {
                color.3 += 0.1;
                println!("Increment param {}", color.3);
            }
            WindowEvent::Key(Key::Down, _, Action::Press | Action::Repeat, _) => {
                color.3 -= 0.1;
                println!("Decrement param {}", color.3);
            }
            WindowEvent::Scroll(w, h) => {
                color.2 += (h * 0.01) as f32;
                color.2 = color.2.clamp(0.0, 1.0);
                println!("color BLUE {}", color.2);
            }
            WindowEvent::Key(Key::Space, _, Action::Press, _)
            | WindowEvent::Key(Key::Space, _, Action::Repeat, _) => {
                state.wireframe = !state.wireframe;
                println!("Pressed space")
            }
            _ => {}
        }
    }
}
