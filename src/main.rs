mod bvh;
mod camera;
mod hittable;
mod hittable_list;
mod mesh;
mod ray;
mod triangle;

use bvh::*;
use camera::*;
use glam::*;
use hittable::{HitRecord, Hittable};
use hittable_list::HittableList;
use image::{Rgb, RgbImage};
use mesh::*;
use ray::Ray;
use std::f32::consts::PI;
use std::time::Instant;
use triangle::*;

type Color = Vec3;

const ASPECT_RATIO: f32 = 1.0 / 1.0;
const IMG_WIDTH: u32 = 600;
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

/// Returns the color of the ray.
fn ray_color(r: &Ray, world: &HittableList) -> Color {
    // Check if ray intersects world.
    let mut rec = HitRecord::new();
    if world.hit(r, 0.0, f32::INFINITY, &mut rec) {
        // TODO: I think mesh normals are being recorded incorrectly.
        // They might be switched around?
        return vec3(1.0 - (rec.t / 10.0), 0.0, 0.0);
        // return 0.5 * (rec.normal + Vec3::ONE);
    }

    let unit_dir = r.direction().normalize();
    let t = 0.5 * (unit_dir.y + 1.0);

    // Returns a color lerped between white and blu-ish
    (1.0 - t) * Color::ONE + t * Color::new(0.5, 0.7, 1.0)
}

/// Returns a scene of 'n' random triangles.
fn random_triangles(n: i32) -> Vec<Triangle> {
    // let mut world = HittableList::new();
    let mut list = Vec::new();

    for _ in 0..n {
        let r0 = vec3(
            rand::random::<f32>(),
            rand::random::<f32>(),
            rand::random::<f32>(),
        );
        let r1 = vec3(
            rand::random::<f32>(),
            rand::random::<f32>(),
            rand::random::<f32>(),
        );
        let r2 = vec3(
            rand::random::<f32>(),
            rand::random::<f32>(),
            rand::random::<f32>(),
        );
        let v0 = r0 * 9.0 - vec3(5.0, 5.0, 5.0);
        let v1 = v0 + r1;
        let v2 = v0 + r2;
        list.push(Triangle::new(v0, v1, v2));
    }

    list
}

fn main() {
    // Scene
    let mut world = HittableList::new();

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
    // plane.transformation(vec3(10.0, 1.0, 10.0), Quat::IDENTITY, vec3(0.0, -2.0, 0.0));
    // world.add(plane);

    // let mut bunny = Mesh::from_gltf("assets/bunny.glb").unwrap();
    // println!("bunny tri count: {}", bunny.num_triangles());
    // bunny.transformation(
    //     Vec3::ONE * 3.0,
    //     Quat::from_rotation_y(PI * 0.25),
    //     vec3(0.0, 0.0, 0.0),
    // );
    // world.add(bunny);

    let mut monkey = Mesh::from_gltf("assets/monkey.glb").unwrap();
    println!("monkey tri count: {}", monkey.num_triangles());
    world.add(monkey);

    // Camera settings.
    let cam = Camera::new(
        vec3(0.0, 0.0, 4.0),
        vec3(0.0, 0.0, 0.0),
        vec3(0.0, 1.0, 0.0),
        80.0,
        ASPECT_RATIO,
    );

    let mut imgbuf = RgbImage::new(IMG_WIDTH, IMG_HEIGHT);

    let now = Instant::now();
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let u = x as f32 / (IMG_WIDTH - 1) as f32;
        let v = 1.0 - (y as f32 / (IMG_HEIGHT - 1) as f32);

        let view_ray = cam.get_ray(u, v);

        let col = ray_color(&view_ray, &world);

        write_color(pixel, col);
    }
    let elapsed = now.elapsed();
    eprintln!("Done!");
    eprintln!("Time taken: {}ms", elapsed.as_millis());

    imgbuf.save("out.png").unwrap();
}
