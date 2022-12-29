use crate::{
    hittable::{HitPayload, Hittable},
    hittable_list::HittableList,
    Camera, Color, Ray,
};
use glam::Vec3A;

pub struct Renderer {
    image_data: Vec<u8>,
    image_width: usize,
    image_height: usize,
}

impl Renderer {
    /// Create a new renderer.
    pub fn new(image_width: usize, image_height: usize) -> Self {
        let image_data = (0..image_width * image_height * 4)
            .map(|i| {
                // Set every 4th value to 255, all else 0.
                if i % 4 == 3 {
                    return 255;
                } else {
                    return 0;
                }
            })
            .collect();
        Self {
            image_data,
            image_width,
            image_height,
        }
    }

    /// Get reference to final image buffer.
    pub fn get_final_image(&self) -> &Vec<u8> {
        &self.image_data
    }

    /// Render current scene to image buffer.
    pub fn render(&mut self, scene: &HittableList, cam: &Camera) {
        for y in 0..self.image_height {
            for x in 0..self.image_width {

                let col = self.per_pixel(scene, cam, x, y);
                let r = (col.x * 255.0) as u8;
                let g = (col.y * 255.0) as u8;
                let b = (col.z * 255.0) as u8;

                // Write colors to buffer.
                // Index into image buffer.
                let i = y * self.image_width * 4 + (x * 4);
                self.image_data[i] = r;
                self.image_data[i + 1] = g;
                self.image_data[i + 2] = b;
            }
        }
    }

    /// RayGen shader
    fn per_pixel(&self, scene: &HittableList, cam: &Camera, x: usize, y: usize) -> Color {
        // Initialize the view ray.
        let mut view_ray = Ray::new(Vec3A::ZERO, Vec3A::ZERO);
        view_ray.set_origin(*cam.get_position());
        view_ray.set_direction(cam.get_ray_directions()[(x + y * self.image_width)]);

        let hit_payload = self.trace_ray(scene, &view_ray);
        if hit_payload.hit_distance < 0.0 {
            // If ray hits nothing, return background color.
            let unit_dir = view_ray.direction().normalize();
            let t = 0.5 * (unit_dir.y + 1.0);

            // Returns a color lerped between white and blu-ish
            return (1.0 - t) * Color::ONE + t * Color::new(0.5, 0.7, 1.0);
        }

        // Map normals to a color
        (hit_payload.world_normal + 1.0) * 0.5
    }

    fn trace_ray(&self, scene: &HittableList, ray: &Ray) -> HitPayload {
        // Check if ray intersects world.
        let mut hit_payload = HitPayload::new();
        hit_payload.hit_distance = f32::INFINITY;

        if !scene.hit(ray, 0.0, f32::INFINITY, &mut hit_payload) {
            // Invoke the miss function.
            return self.miss(ray);
        }

        // Will contain hit information about the closest intersection.
        hit_payload
    }

    /// Invoked every time a ray misses every object in the scene.
    fn miss(&self, ray: &Ray) -> HitPayload {
        let mut hit_payload = HitPayload::new();
        hit_payload.hit_distance = -1.0;
        hit_payload
    }
}
