use crate::input::*;
use glam::*;
use imgui::Ui;
use winit::event::VirtualKeyCode;

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

    // Effects how fast the camera moves.
    speed: f32,
}

impl Camera {
    pub fn new(
        vertical_fov: f32,
        near_clip: f32,
        far_clip: f32,
        viewport_width: u32,
        viewport_height: u32,
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
        let mut ray_directions = Vec::with_capacity((viewport_width * viewport_height) as usize);
        for y in 0..viewport_height {
            for x in 0..viewport_width {
                let mut coord = vec2(
                    x as f32 / viewport_width as f32,
                    y as f32 / viewport_height as f32,
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

            speed: 4.0,
        }
    }

    /// Update camera depending on input state.
    /// Returns true if camera moved and false otherwise.
    pub fn update(&mut self, input_state: &InputState, dt: f32) -> bool {
        // TODO: Check if right mouse button is held down.
        let mut moved = false;

        let up_dir = vec3a(0.0, 1.0, 0.0);
        let right_dir = self.forward_direction.cross(up_dir);

        // Movement.
        if input_state.is_key_down(VirtualKeyCode::W) {
            self.position += self.forward_direction * self.speed * dt;
            moved = true;
        }
        if input_state.is_key_down(VirtualKeyCode::S) {
            self.position -= self.forward_direction * self.speed * dt;
            moved = true;
        }
        if input_state.is_key_down(VirtualKeyCode::D) {
            self.position += right_dir * self.speed * dt;
            moved = true;
        }
        if input_state.is_key_down(VirtualKeyCode::A) {
            self.position -= right_dir * self.speed * dt;
            moved = true;
        }
        if input_state.is_key_down(VirtualKeyCode::E) {
            self.position += up_dir * self.speed * dt;
            moved = true;
        }
        if input_state.is_key_down(VirtualKeyCode::Q) {
            self.position -= up_dir * self.speed * dt;
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

        for y in 0..self.viewport_height {
            for x in 0..self.viewport_width {
                let mut coord = vec2(
                    x as f32 / self.viewport_width as f32,
                    y as f32 / self.viewport_height as f32,
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
