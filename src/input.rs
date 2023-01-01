use winit::event::{ElementState, MouseButton, VirtualKeyCode, WindowEvent};

pub struct InputState {
    // Index via virtual keycode.
    key_state: [ElementState; 255],
    // Index via MouseButton enum?
    mouse_state: [ElementState; 3], // Left, Right, or Middle
    mouse_position: (f32, f32),
    last_mouse_position: Option<(f32, f32)>,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            key_state: [ElementState::Released; 255],
            mouse_state: [ElementState::Released; 3],
            mouse_position: (0.0, 0.0),
            last_mouse_position: None,
        }
    }

    pub fn update(&mut self, event: WindowEvent) {
        match event {
            WindowEvent::KeyboardInput { input, .. } => {
                if let Some(keycode) = input.virtual_keycode {
                    self.key_state[keycode as usize] = input.state;
                }
            }
            WindowEvent::MouseInput { state, button, .. } => match button {
                MouseButton::Left => self.mouse_state[0] = state,
                MouseButton::Right => self.mouse_state[1] = state,
                MouseButton::Middle => self.mouse_state[2] = state,
                _ => {}
            },
            WindowEvent::CursorMoved { position, .. } => {
                // Update last mouse pos.
                self.last_mouse_position = Some(self.mouse_position);
                // Set current mouse pos.
                self.mouse_position = (position.x as f32, position.y as f32);
            }
            _ => {}
        }
    }

    pub fn is_key_down(&self, key: VirtualKeyCode) -> bool {
        self.key_state[key as usize] == ElementState::Pressed
    }

    pub fn is_mouse_button_down(&self, button: MouseButton) -> bool {
        let state = match button {
            MouseButton::Left => self.mouse_state[0],
            MouseButton::Right => self.mouse_state[1],
            MouseButton::Middle => self.mouse_state[2],
            _ => ElementState::Released,
        };

        state == ElementState::Pressed
    }

    pub fn get_mouse_pos(&self) -> (f32, f32) {
        self.mouse_position
    }

    /// Get the change in mouse coordinates.
    /// TODO: This doesn't seem to be working?
    pub fn get_mouse_delta(&self) -> (f32, f32) {
        match self.last_mouse_position {
            Some(last_mouse_pos) => (
                self.mouse_position.0 - last_mouse_pos.0,
                self.mouse_position.1 - last_mouse_pos.1,
            ),
            None => (0.0, 0.0),
        }
    }
}
