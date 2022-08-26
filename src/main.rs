mod bvh;
mod camera;
mod hittable;
mod hittable_list;
mod mesh;
mod ray;
mod rng;
mod thread_pool;
mod triangle;
mod util;

use camera::*;
use glam::*;
use hittable::{HitRecord, Hittable};
use hittable_list::HittableList;
use image::{Rgb, RgbImage};
use mesh::*;
use ray::Ray;
use rng::*;
use std::f32::consts::PI;
use std::num;
use std::time::Instant;
use thread_pool::*;
use triangle::*;
use util::random_unit_vector;

type Color = Vec3;

const ASPECT_RATIO: f32 = 4.0 / 3.0;
const IMG_WIDTH: u32 = 800;
const IMG_HEIGHT: u32 = (IMG_WIDTH as f32 / ASPECT_RATIO) as u32;

/// Writes a color to a pixel.
/// Expects components of 'col' to be in [0.0, 1.0]
fn write_color(pixel: &mut Rgb<u8>, col: Color) {
    *pixel = Rgb([
        (col.x * 255.0) as u8,
        (col.y * 255.0) as u8,
        (col.z * 255.0) as u8,
    ]);
}

/// Returns the color of the surface a given ray is pointing at.
fn ray_color(r: &Ray, world: &HittableList, depth: u16, rng: &mut Rng) -> Color {
    if depth == 0 {
        // Exceeded path length.
        return Vec3::ZERO;
    }

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

    // Lambertian scattering.
    let mut scatter_direction = rec.normal + random_unit_vector(rng);

    // Catch degenerate scatter direction - if the random unit vector
    // generated is exactly opposite to the surface normal, then they will
    // sum to zero, resulting in a zero scatter direction vector. This will
    // lead to issues later on.
    if scatter_direction == Vec3::ZERO {
        scatter_direction = rec.normal;
    }

    // Initialize scattered ray.
    let scattered = Ray::new(rec.p, scatter_direction.normalize());

    0.5 * ray_color(&scattered, world, depth - 1, rng)

    // 0.5 * (rec.normal + Vec3::ONE)
}

// /// Returns a scene of 'n' random triangles.
// fn random_triangles(rng: &mut Rng, n: i32) -> Vec<Triangle> {
//     // let mut world = HittableList::new();
//     let mut list = Vec::new();

//     for _ in 0..n {
//         let r0 = vec3(rng.random_uniform(), rng.randomf32(), rng.randomf32());
//         let r1 = vec3(rng.randomf32(), rng.randomf32(), rng.randomf32());
//         let r2 = vec3(rng.randomf32(), rng.randomf32(), rng.randomf32());
//         let v0 = r0 * 9.0 - vec3(5.0, 5.0, 5.0);
//         let v1 = v0 + r1;
//         let v2 = v0 + r2;
//         list.push(Triangle::new(v0, v1, v2));
//     }

//     list
// }

fn main() {
    let mut rng = Rng::from_seed(727);
    // Scene
    let mut world = HittableList::new();

    // let triangles = Mesh::from_triangles(random_triangles(&mut rng, 10_000));
    // world.add(triangles);

    let mut cube1 = Mesh::from_gltf("assets/cube.glb").unwrap();
    println!("cube tri count: {}", cube1.num_triangles());
    cube1.transformation(
        Vec3::ONE,
        Quat::from_rotation_y(PI * 0.25),
        vec3(0.0, 0.0, 0.0),
    );
    world.add(cube1);

    let mut plane = Mesh::from_gltf("assets/plane.glb").unwrap();
    println!("plane tri count: {}", plane.num_triangles());
    plane.transformation(vec3(25.0, 1.0, 25.0), Quat::IDENTITY, vec3(0.0, -2.0, 0.0));
    world.add(plane);

    let mut bunny = Mesh::from_gltf("assets/bunny.glb").unwrap();
    println!("bunny tri count: {}", bunny.num_triangles());
    bunny.transformation(
        Vec3::ONE,
        Quat::from_rotation_y(PI * 0.25),
        vec3(0.0, 1.5, 0.0),
    );
    world.add(bunny);

    let mut monkey = Mesh::from_gltf("assets/monkey.glb").unwrap();
    monkey.transformation(
        Vec3::ONE,
        Quat::from_rotation_y(PI * 0.25),
        vec3(-4.0, 1.0, -1.0),
    );
    println!("monkey tri count: {}", monkey.num_triangles());
    world.add(monkey);

    let mut icosphere = Mesh::from_gltf("assets/icosphere.glb").unwrap();
    icosphere.transformation(
        Vec3::ONE,
        Quat::from_rotation_y(PI * 0.25),
        vec3(4.0, 1.0, -1.0),
    );

    world.add(icosphere);

    // Camera settings.
    let cam = Camera::new(
        vec3(0.0, 2.0, 6.0),
        vec3(0.0, 0.0, 0.0),
        vec3(0.0, 1.0, 0.0),
        80.0,
        ASPECT_RATIO,
    );

    let max_depth = 5;
    let num_samples = 16;

    let mut imgbuf = RgbImage::new(IMG_WIDTH, IMG_HEIGHT);

    let now = Instant::now();
    // Iterate over each pixel in the image.
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let mut col = Vec3::ZERO;

        for _ in 0..num_samples {
            let u = (x as f32 + rng.random_uniform()) / (IMG_WIDTH - 1) as f32;
            let v = 1.0 - ((y as f32 + rng.random_uniform()) / (IMG_HEIGHT - 1) as f32);

            let view_ray = cam.get_ray(u, v);

            col += ray_color(&view_ray, &world, max_depth, &mut rng);
        }

        write_color(pixel, col / num_samples as f32);
    }
    let elapsed = now.elapsed();
    eprintln!("Done!");
    eprintln!("Time taken: {}ms", elapsed.as_millis());

    imgbuf.save("out.png").unwrap();
}
