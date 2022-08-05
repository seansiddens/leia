mod ray;
mod vec3;
use image::{Rgb, RgbImage};
use ray::Ray;
use vec3::{Color, Point3, Vec3};

const ASPECT_RATIO: f32 = 4.0 / 3.0;
const IMG_WIDTH: u32 = 400;
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
fn ray_color(r: &Ray) -> Color {
    let unit_dir = r.direction().normalize();
    let t = 0.5 * (unit_dir.y + 1.0);

    // Returns a color lerpbed between white and blu-ish
    (1.0 - t) * Color::ONE + t * Color::new(0.5, 0.7, 1.0)
}

fn main() {
    // Camera settings.
    let viewport_height = 2.0;
    let viewport_width = ASPECT_RATIO * viewport_height;
    let focal_length = 1.0;

    // +Y is up, -Z is forward.
    let origin = Point3::ZERO;
    let horizontal = Vec3::new(viewport_width, 0.0, 0.0);
    let vertical = Vec3::new(0.0, viewport_height, 0.0);
    let lower_left = origin - horizontal / 2.0 - vertical / 2.0 - Vec3::new(0.0, 0.0, focal_length);

    let mut imgbuf = RgbImage::new(IMG_WIDTH, IMG_HEIGHT);

    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let u = x as f32 / (IMG_WIDTH - 1) as f32;
        let v = 1.0 - (y as f32 / (IMG_HEIGHT - 1) as f32);

        let view_ray = Ray::new(origin, lower_left + u * horizontal + v * vertical - origin);

        let col = ray_color(&view_ray);

        write_color(pixel, col);

        // Report progress.
        if x == 0 {
            eprintln!("Scanlines remaining: {}", IMG_HEIGHT - y);
        }
    }
    eprintln!("Done!");

    imgbuf.save("out.png").unwrap();
}
