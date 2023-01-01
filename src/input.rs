use winit::event::{ElementState, WindowEvent, VirtualKeyCode};

pub struct InputState {
    key_state: [ElementState; 255],
}

impl InputState {
    pub fn new() -> Self {
        Self {
            key_state: [ElementState::Released; 255],
        }
    }

    pub fn update(&mut self, event: WindowEvent) {
        match event {
            WindowEvent::KeyboardInput { input, .. } => {
                if let Some(keycode) = input.virtual_keycode {
                    match input.state {
                        ElementState::Pressed => {
                            self.key_state[keycode as usize] = ElementState::Pressed;
                        }
                        ElementState::Released => {
                            self.key_state[keycode as usize] = ElementState::Released;
                        }
                    }
                }
            }
            _ => {}
        }
    }

    pub fn is_key_down(&self, key: VirtualKeyCode) -> bool {
        self.key_state[key as usize] == ElementState::Pressed
    }
}
