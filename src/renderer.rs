use crate::{
    hittable::{HitRecord, Hittable},
    hittable_list::HittableList,
    Camera, Color, Ray,
};
use glam::Vec3;

/// Returns the color of the surface a given ray is pointing at.
fn ray_color(r: &Ray, world: &HittableList) -> Color {
    // if depth <= 0 {
    //     // Exceeded path length.
    //     return Vec3::ZERO;
    // }

    // Check if ray intersects world.
    let mut rec = HitRecord::new();
    rec.t = f32::INFINITY;

    if !world.hit(r, 0.0, f32::INFINITY, &mut rec) {
        // If ray hits nothing, return background color.
        let unit_dir = r.direction().normalize();
        let t = 0.5 * (unit_dir.y + 1.0);

        // Returns a color lerped between white and blu-ish
        return (1.0 - t) * Color::ONE + t * Color::new(0.5, 0.7, 1.0);
    }

    // Ray hit something.
    // Build ONB from normal.
    // let uvw = Onb::from_w(rec.normal);
    // let direction = uvw.local(cosine_sample_hemisphere(
    //     rng.random_uniform(),
    //     rng.random_uniform(),
    // ));

    // let scattered = Ray::new(rec.p, direction.normalize());

    // // Color based on surface normal
    return 0.5 * (rec.normal + Vec3::ONE);

    // 0.8 * ray_color(&scattered, world, depth - 1, rng)
}

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

    pub fn render(&mut self, scene: &HittableList, cam: &Camera) {
        let mut view_ray = Ray::new(Vec3::ZERO, Vec3::ZERO);
        view_ray.set_origin(*cam.get_position());
        let ray_directions = cam.get_ray_directions();

        for y in 0..self.image_height {
            for x in 0..self.image_width {
                // Index into image buffer.
                let i = y * self.image_width * 4 + (x * 4);

                // // Normalized texture coords into the image
                // let u = (x as f32) / (self.image_width - 1) as f32;
                // let v = 1.0 - ((y as f32) / (self.image_height - 1) as f32);

                // let view_ray = cam.get_ray(u, v);
                view_ray.set_direction(ray_directions[x + y * self.image_width]);

                let col = ray_color(&view_ray, scene);
                let r = (col.x * 255.0) as u8;
                let g = (col.y * 255.0) as u8;
                let b = (col.z * 255.0) as u8;

                // Write colors to buffer.
                self.image_data[i] = r;
                self.image_data[i + 1] = g;
                self.image_data[i + 2] = b;
            }
        }
    }

    fn per_pixel(&self, u: f32, v: f32) -> Color {
        Color {
            x: 1.0,
            y: 0.5,
            z: 0.0,
        }
    }

    pub fn get_final_image(&self) -> &Vec<u8> {
        &self.image_data
    }
}
