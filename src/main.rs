mod vec3;
use crate::vec3::{Color, Vec3};
use image::{Rgb, RgbImage};

fn write_color(col: Color) {
    todo!();
}

fn main() {
    const IMG_WIDTH: u32 = 512;
    const IMG_HEIGHT: u32 = 512;

    let mut imgbuf = RgbImage::new(IMG_WIDTH, IMG_HEIGHT);

    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let r = x as f32 / IMG_WIDTH as f32;
        let g = y as f32 / IMG_HEIGHT as f32;
        let b = 0.0;

        *pixel = Rgb([(r * 256.0) as u8, (g * 256.0) as u8, (b * 256.0) as u8]);

        // Report progress.
        if x == 0 {
            eprintln!("Scanlines remaining: {}", IMG_HEIGHT - y);
        }
    }
    eprintln!("Done!");

    imgbuf.save("out.png").unwrap();
}
