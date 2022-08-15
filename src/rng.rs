pub struct Rng {
    rnd_state: u32,
}

impl Rng {
    pub fn from_seed(seed: u32) -> Self {
        Self { rnd_state: seed }
    }

    // From https://en.wikipedia.org/wiki/Xorshift
    pub fn xor_shift(&mut self) -> u32 {
        let mut x = self.rnd_state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 15;
        self.rnd_state = x;
        x
    }

    /// Returns a random f32 in [0.0, 1.0)
    pub fn randomf32(&mut self) -> f32 {
        let max: u64 = u32::MAX as u64 + 1;

        self.xor_shift() as f32 / max as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xor_shift() {
        let mut rng = Rng::from_seed(69);

        let mut _x;
        for _ in 0..1_000_000 {
            _x = rng.xor_shift();
        }
    }

    #[test]
    fn randomf32() {
        let mut rng = Rng::from_seed(512);

        let mut x;
        for _ in 0..1_000_000 {
            x = rng.randomf32();
            assert!(x >= 0.0 && x < 1.0);
        }
    }
}
