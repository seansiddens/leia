pub struct Renderer {
    pub image_data: Vec<u8>,
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
        Self {
            image_data,
            image_width,
            image_height,
        }
    }

    pub fn render(&mut self) -> &Vec<u8>{
        for x in 0..self.image_width {
            for y in 0..self.image_height {
                let i = y * self.image_width + x;
                self.image_data[i] += 1;
                self.image_data[i] %= 255;
            }
        }

        &self.image_data
    }
}
