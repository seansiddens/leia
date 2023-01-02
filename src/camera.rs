use crate::input::*;
use glam::*;
use imgui::Ui;
use rand::{Rng, SeedableRng};
use winit::event::{MouseButton, VirtualKeyCode};

pub struct Camera {
    projection: Mat4,
    view: Mat4,
    inverse_projection: Mat4,
    inverse_view: Mat4,

    vertical_fov: f32,
    near_clip: f32,
    far_clip: f32,

    position: Vec3A,
    forward_direction: Vec3A,

    // Cached ray directions
    ray_directions: Vec<Vec3A>,

    viewport_width: u32,
    viewport_height: u32,

    movement_speed: f32,
    rotation_speed: f32,
    last_mouse_pos: (f32, f32),

    enable_aa: bool, // Flag to enable anti-aliasing.
}

impl Camera {
    pub fn new(
        vertical_fov: f32,
        near_clip: f32,
        far_clip: f32,
        viewport_width: u32,
        viewport_height: u32,
        enable_aa: bool,
    ) -> Self {
        // TODO: These defaults shouldn't be hard-coded like this.
        let forward_direction = vec3a(0.0, 0.0, -1.0);
        let position = vec3a(0.0, 1.0, 3.5);

        let view = Mat4::look_at_rh(
            position.into(),
            (position + forward_direction).into(),
            vec3(0.0, 1.0, 0.0),
        );
        let inverse_view = view.inverse();
        let projection = Mat4::perspective_rh(
            vertical_fov.to_radians(),
            viewport_width as f32 / viewport_height as f32,
            near_clip,
            far_clip,
        );
        let inverse_projection = projection.inverse();

        // Initialize the ray directions
        let mut rng = rand_xoshiro::Xoroshiro128PlusPlus::from_entropy();
        let mut ray_directions = Vec::with_capacity((viewport_width * viewport_height) as usize);
        for y in 0..viewport_height {
            for x in 0..viewport_width {
                let (jitter_x, jitter_y) = if enable_aa {
                    (rng.gen_range(0.0..1.0), rng.gen_range(0.0..1.0))
                } else {
                    (0.0, 0.0)
                };
                let mut coord = vec2(
                    (x as f32 + jitter_x) / viewport_width as f32,
                    (y as f32 + jitter_y) / viewport_height as f32,
                );
                coord = coord * 2.0 - 1.0; // -1 -> 1

                let target = inverse_projection * vec4(coord.x, coord.y, 1.0, 1.0);
                let ray_direction = Vec3A::from(
                    (inverse_view * ((target.truncate() / target.w).normalize()).extend(0.0))
                        .truncate(),
                ); // World space
                ray_directions.push(ray_direction);
            }
        }

        Self {
            view,
            inverse_view,
            projection,
            inverse_projection,

            vertical_fov,
            near_clip,
            far_clip,

            position,
            forward_direction,

            ray_directions,

            viewport_width,
            viewport_height,

            movement_speed: 4.0,
            rotation_speed: 0.3,

            last_mouse_pos: (0.0, 0.0),
            enable_aa,
        }
    }

    /// Update camera depending on input state.
    /// Returns true if camera moved and false otherwise.
    pub fn update(&mut self, input_state: &InputState, dt: f32) -> bool {
        let mouse_pos: winit::dpi::LogicalPosition<f32> = input_state.get_mouse_pos().into();
        let mut mouse_delta = (
            mouse_pos.x - self.last_mouse_pos.0,
            mouse_pos.y - self.last_mouse_pos.1,
        );
        mouse_delta.0 *= dt;
        mouse_delta.1 *= dt;

        self.last_mouse_pos = (mouse_pos.x, mouse_pos.y);
        if !input_state.is_mouse_button_down(MouseButton::Right) {
            // TODO: Change cursor mode?
            // Probably should be done by Application.
            return false;
        }

        let mut moved = false;

        let up_dir = vec3a(0.0, 1.0, 0.0);
        let right_dir = self.forward_direction.cross(up_dir);

        // Handle movement.
        if input_state.is_key_down(VirtualKeyCode::W) {
            self.position += self.forward_direction * self.movement_speed * dt;
            moved = true;
        }
        if input_state.is_key_down(VirtualKeyCode::S) {
            self.position -= self.forward_direction * self.movement_speed * dt;
            moved = true;
        }
        if input_state.is_key_down(VirtualKeyCode::D) {
            self.position += right_dir * self.movement_speed * dt;
            moved = true;
        }
        if input_state.is_key_down(VirtualKeyCode::A) {
            self.position -= right_dir * self.movement_speed * dt;
            moved = true;
        }
        if input_state.is_key_down(VirtualKeyCode::E) {
            self.position += up_dir * self.movement_speed * dt;
            moved = true;
        }
        if input_state.is_key_down(VirtualKeyCode::Q) {
            self.position -= up_dir * self.movement_speed * dt;
            moved = true;
        }

        // Handle rotation.
        if mouse_delta.0 != 0.0 || mouse_delta.1 != 0.0 {
            let pitch_delta = mouse_delta.1 * self.rotation_speed;
            let yaw_delta = mouse_delta.0 * self.rotation_speed;

            // Create quaternion from rotations.
            let q = (Quat::from_axis_angle(right_dir.into(), -pitch_delta)
                * Quat::from_axis_angle(up_dir.into(), -yaw_delta))
            .normalize();

            // Rotate forward direction.
            self.forward_direction = q * self.forward_direction;

            moved = true;
        }

        // If the camera moved we need to recompute the view matrix and ray directions.
        if moved {
            self.recalculate_view();
            self.recalculate_view_directions();
        }

        moved
    }

    pub fn get_ray_directions(&self) -> &Vec<Vec3A> {
        &self.ray_directions
    }

    pub fn get_position(&self) -> &Vec3A {
        &self.position
    }

    pub fn set_position(&mut self, position: Vec3A) {
        self.position = position;
        self.recalculate_view();
        self.recalculate_view_directions();
    }

    /// Recompute the view directions.
    fn recalculate_view_directions(&mut self) {
        self.ray_directions.resize(
            (self.viewport_width * self.viewport_height) as usize,
            Vec3A::ZERO,
        );

        let mut rng = rand_xoshiro::Xoroshiro128PlusPlus::from_entropy();
        for y in 0..self.viewport_height {
            for x in 0..self.viewport_width {
                let (jitter_x, jitter_y) = if self.enable_aa {
                    (rng.gen_range(0.0..1.0), rng.gen_range(0.0..1.0))
                } else {
                    (0.0, 0.0)
                };
                // TODO: The jittering should take place per-frame.
                let mut coord = vec2(
                    (x as f32 + jitter_x) / self.viewport_width as f32,
                    (y as f32 + jitter_y) / self.viewport_height as f32,
                );
                coord = coord * 2.0 - 1.0; // -1 -> 1

                let target = self.inverse_projection * vec4(coord.x, coord.y, 1.0, 1.0);
                let ray_direction = Vec3A::from(
                    (self.inverse_view * ((target.truncate() / target.w).normalize()).extend(0.0))
                        .truncate(),
                ); // World space
                self.ray_directions[(x + y * self.viewport_width) as usize] = ray_direction;
            }
        }
    }

    /// Recompute the view and inverse view matrices.
    /// This function should be called whenver the camera's position or
    /// orientation is modified.
    fn recalculate_view(&mut self) {
        self.view = Mat4::look_at_rh(
            self.position.into(),
            (self.position + self.forward_direction).into(),
            vec3(0.0, 1.0, 0.0),
        );
        self.inverse_view = self.view.inverse();
    }

    /// Recompute the projection and inverse projection matrices.
    /// This function should be called if the viewport dimensions, vertical fov,
    /// or near/far clip planes ever change.
    fn recalculate_projection(&mut self) {
        self.projection = Mat4::perspective_rh(
            self.vertical_fov.to_radians(),
            self.viewport_width as f32 / self.viewport_height as f32,
            self.near_clip,
            self.far_clip,
        );
        self.inverse_projection = self.projection.inverse();
    }
}
