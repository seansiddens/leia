use crate::{
    hittable::{HitPayload, Hittable},
    hittable_list::HittableList,
    util, Camera, Color, Ray,
};
use glam::{vec3a, Vec3, Vec3A};
use rand::{Rng, SeedableRng};
use rayon::prelude::*;
use std::{f32::consts::PI, time::Instant};

pub struct Renderer {
    image_width: usize,
    image_height: usize,

    image_data: Vec<u8>,
    accumulation_data: Vec<Vec3A>,
    frame_index: u64,
}

enum RenderMode {
    Default,
    Normals,
    // TimePerPixel,
}

// Maximum length of our light paths.
const NUM_BOUNCES: u32 = 3;

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

        // 32 bits per channel, RGB only.
        let accumulation_data = (0..image_width * image_height)
            .map(|_| Vec3A::ZERO)
            .collect();

        Self {
            image_data,
            accumulation_data,
            image_width,
            image_height,
            frame_index: 1,
        }
    }

    /// Get reference to final image buffer.
    pub fn get_final_image(&self) -> &Vec<u8> {
        &self.image_data
    }

    pub fn reset_accumulation_data(&mut self) {
        // Reset the frame index.
        self.frame_index = 1;

        // Reset acc data.
        self.accumulation_data.fill(Vec3A::ZERO);
    }

    pub fn get_frame_index(&self) -> u64 {
        self.frame_index
    }

    /// Render current scene to image buffer.
    /// master_rng is used for seeding the thread-level RNGs.
    pub fn render(&mut self, scene: &HittableList, cam: &Camera, master_rng: &mut impl Rng) {
        // Take the ownership of the image and accumulation data.
        let mut image_data = std::mem::take(&mut self.image_data);
        let mut accumulation_data = std::mem::take(&mut self.accumulation_data);

        // Generate seeds for each thread.
        let seeds = (0..self.image_width * self.image_height)
            .map(|_| master_rng.gen())
            .collect::<Vec<u64>>();

        // Split each pixel into a task.
        image_data
            .chunks_mut(4)
            .zip(accumulation_data.iter_mut())
            .zip(0..self.image_width * self.image_height)
            .map(|((pixel, acc), i)| (i, pixel, acc))
            .zip(seeds)
            .map(|((i, pixel, acc), seed)| (i, pixel, acc, seed))
            .collect::<Vec<(usize, &mut [u8], &mut Vec3A, u64)>>()
            .into_par_iter()
            .for_each(|(i, pixel, acc_data, seed)| {
                // Get x and y position into final image.
                let y = i / self.image_width;
                let x = i % self.image_width;

                // Create RNG
                let mut rng = rand_xoshiro::Xoroshiro128PlusPlus::seed_from_u64(seed);

                // Shoot ray and accumulate color data.
                let col = self.per_pixel(&scene, cam, &mut rng, x, y);
                *acc_data += col;

                // Average the accumulated data.
                let accumulated_color = *acc_data / self.frame_index as f32;

                // Clamp color values to prevent under/over-flow.
                accumulated_color.clamp(Vec3A::ZERO, Vec3A::ONE);
                let r = (accumulated_color.x * 255.0) as u8;
                let g = (accumulated_color.y * 255.0) as u8;
                let b = (accumulated_color.z * 255.0) as u8;

                // Write color to pixel
                pixel[0] = r;
                pixel[1] = g;
                pixel[2] = b;
                pixel[3] = 255;
            });

        // // Split image data into mutable chunks.
        // let image_bands: Vec<(usize, &mut [u8])> = image_data
        //     .chunks_mut(self.image_width * 4)
        //     .enumerate()
        //     .collect();

        // // Render each chunk of image data in parallel.
        // image_bands.into_par_iter().for_each(|(y, image_data)| {
        //     for x in 0..self.image_width {
        //         let col = self.per_pixel(&scene, cam, x, y);
        //         let r = (col.x * 255.0) as u8;
        //         let g = (col.y * 255.0) as u8;
        //         let b = (col.z * 255.0) as u8;

        //         // Index into slice
        //         let i = x * 4;
        //         image_data[i] = r;
        //         image_data[i + 1] = g;
        //         image_data[i + 2] = b;
        //     }
        // });

        // Give ownership back to self.
        self.image_data = image_data;
        self.accumulation_data = accumulation_data;

        // Increase frame index
        self.frame_index += 1;
    }

    /// RayGen shader
    fn per_pixel(
        &self,
        scene: &HittableList,
        cam: &Camera,
        rng: &mut impl Rng,
        x: usize,
        y: usize,
    ) -> Color {
        let t = Instant::now();

        // Initialize the view ray.
        let mut view_ray = Ray::new(Vec3A::ZERO, Vec3A::ZERO);
        view_ray.set_origin(*cam.get_position());
        view_ray.set_direction(cam.get_ray_directions()[(x + y * self.image_width)]);

        self.ray_color(&view_ray, 0, scene, rng)
        // // Begin integrating the light path.
        // let num_bounces = 1; // Just do direct lighting for now.
        // for i in 0..num_bounces + 1 {
        //     let hit_payload = self.trace_ray(scene, &view_ray);
        //     if hit_payload.hit_distance < 0.0 {
        //         // If ray hits nothing, return background color.
        //         let unit_dir = view_ray.direction().normalize();
        //         let t = 0.5 * (unit_dir.y + 1.0);

        //         // Returns a color lerped between white and blu-ish
        //         color += (1.0 - t) * Color::ONE + t * Color::new(0.5, 0.7, 1.0) * 0.5;
        //     }
        // }

        // TODO: Don't hardcode this.
        // let render_mode = RenderMode::Normals;
        // match render_mode {
        //     RenderMode::Normals => {
        //         // Map normals to a color
        //         return (hit_payload.world_normal + 1.0) * 0.5;
        //     }
        //     RenderMode::Default => {
        //         // TODO: Implement
        //         return Vec3A::ZERO;
        //     }
        // }
    }

    /// Given a
    /// TODO: Could v_inv just be a ray value?
    fn ray_color(
        &self,
        v_inv: &Ray,
        depth: u32,
        scene: &HittableList,
        rng: &mut impl Rng,
    ) -> Color {
        if depth > NUM_BOUNCES {
            // We've reached the maximum light path length.
            return Color::ZERO;
        }

        let hit_payload = self.trace_ray(scene, v_inv);
        if hit_payload.hit_distance < 0.0 {
            // Ray missed everything in our scene. Return the background color
            let unit_dir = v_inv.direction().normalize();
            let t = 0.5 * (unit_dir.y + 1.0);

            // Returns a color lerped between white and blu-ish
            return (1.0 - t) * Color::ONE + t * Color::new(0.5, 0.7, 1.0) * 0.5;
        }

        // Ray hit an object in the scene.
        let mut color = Color::ZERO;

        // TODO: For now, the only emmssive material in the scene is the background, so
        // for now the emmissive contribution is zero.
        let emmisive = Color::ZERO;
        color += emmisive;

        // Generate new sample.
        let omega = util::uniform_hemisphere_sample_world(rng, hit_payload.world_normal);
        let pdf = 1.0 / (2.0 * PI);

        // Create new outgoing ray.
        let outgoing_ray = Ray::new(hit_payload.world_position, omega);

        // Add contribution of new sample.
        let brdf = hit_payload.albedo / PI;
        color += brdf
            * hit_payload.world_normal.dot(omega) // Cosine term
            * (1.0 / pdf) // Monte-carlo compensation.
            * self.ray_color(&outgoing_ray, depth + 1, scene, rng); // Recursive call to new sample.

        color
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
