pub struct Renderer {
    image_data: Vec<u8>,
    image_width: usize,
    image_height: usize,
}

impl Renderer {
    pub fn new(image_width: usize, image_height: usize) -> Self {
        let image_data = (0..image_width * image_height * 4).map(|i| {
            // Set every 4th value to 255, all else 0.
            if i % 4 == 3 {
                return 255;
            } else {
                return 0;
            }
        }).collect();
        println!("{:?}", image_data);
        Self {
            image_data,
            image_width,
            image_height,
        }

    }
}
