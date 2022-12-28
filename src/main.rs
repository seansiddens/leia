mod application;
mod renderer;
mod bvh;
mod camera;
mod hittable;
mod hittable_list;
mod mesh;
mod onb;
mod ray;
mod rng;
mod thread_pool;
mod triangle;
mod util;
mod imgui_dock;

use application::Application;
use camera::*;
use glam::*;
use hittable::{HitRecord, Hittable};
use hittable_list::HittableList;
use image::{Rgb, RgbImage};
use mesh::*;
use onb::*;
use ray::Ray;
use rng::*;
use std::f32::consts::PI;
use std::num;
use std::time::Instant;
use thread_pool::*;
use triangle::*;
use util::*;

type Color = Vec3;

const ASPECT_RATIO: f32 = 4.0 / 3.0;
const IMG_WIDTH: u32 = 800;
const IMG_HEIGHT: u32 = (IMG_WIDTH as f32 / ASPECT_RATIO) as u32;

/// Returns a scene of 'n' random triangles.
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
    let app = Application::init(file!(), 1920, 1080);
    app.main_loop();

    // let mut rng = Rng::from_seed(727);
    // Scene
    // let mut world = HittableList::new();

    // let triangles = Mesh::from_triangles(random_triangles(&mut rng, 10_000));
    // world.add(triangles);

    // let mut cornell = Mesh::from_gltf("assets/cornell.glb").unwrap();
    // println!("cube tri count: {}", cornell.num_triangles());
    // world.add(cornell);

    // let mut cube1 = Mesh::from_gltf("assets/cube.glb").unwrap();
    // println!("cube tri count: {}", cube1.num_triangles());
    // cube1.transformation(
    //     Vec3::ONE,
    //     Quat::from_rotation_y(PI * 0.25),
    //     vec3(0.0, 0.0, 0.0),
    // );
    // world.add(cube1);

    // let mut plane = Mesh::from_gltf("assets/plane.glb").unwrap();
    // println!("plane tri count: {}", plane.num_triangles());
    // plane.transformation(vec3(25.0, 1.0, 25.0), Quat::IDENTITY, vec3(0.0, -2.0, 0.0));
    // world.add(plane);

    // let mut bunny = Mesh::from_gltf("assets/bunny.glb").unwrap();
    // println!("bunny tri count: {}", bunny.num_triangles());
    // bunny.transformation(
    //     Vec3::ONE,
    //     Quat::from_rotation_y(PI * 0.25),
    //     vec3(0.0, 1.5, 0.0),
    // );
    // world.add(bunny);

    // let mut monkey = Mesh::from_gltf("assets/monkey.glb").unwrap();
    // monkey.transformation(
    //     Vec3::ONE,
    //     Quat::from_rotation_y(PI * 0.25),
    //     vec3(-4.0, 1.0, -1.0),
    // );
    // println!("monkey tri count: {}", monkey.num_triangles());
    // world.add(monkey);

    // let mut icosphere = Mesh::from_gltf("assets/icosphere.glb").unwrap();
    // icosphere.transformation(
    //     Vec3::ONE,
    //     Quat::from_rotation_y(PI * 0.25),
    //     vec3(4.0, 1.0, -1.0),
    // );

    // world.add(icosphere);

    // // Camera settings.
    // let cam = Camera::new(
    //     vec3(0.0, 1.0, 3.0),
    //     vec3(0.0, 1.0, 0.0),
    //     vec3(0.0, 1.0, 0.0),
    //     60.0,
    //     ASPECT_RATIO,
    // );

    // let max_depth = 5;
    // let num_samples = 1;

    // let mut imgbuf = RgbImage::new(IMG_WIDTH, IMG_HEIGHT);

    // let now = Instant::now();
    // // Iterate over each pixel in the image.
    // for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
    //     let mut col = Vec3::ZERO;
    //     for _ in 0..num_samples {
    //         let u = (x as f32 + rng.random_uniform()) / (IMG_WIDTH - 1) as f32;
    //         let v = 1.0 - ((y as f32 + rng.random_uniform()) / (IMG_HEIGHT - 1) as f32);

    //         let view_ray = cam.get_ray(u, v);

    //         col += ray_color(&view_ray, &world, max_depth, &mut rng);
    //     }
    //     write_color(pixel, col / num_samples as f32);

    //     if x == 0 {
    //         println!("Scanlines remaining: {}", IMG_HEIGHT - y);
    //     }
    // }
    // let elapsed = now.elapsed();
    // eprintln!("Done!");
    // eprintln!("Time taken: {}ms", elapsed.as_millis());

    // imgbuf.save("out.png").unwrap();
}
