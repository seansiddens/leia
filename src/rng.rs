pub struct Rng {
    rnd_state: u32,
}

#[allow(dead_code)]
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
    pub fn random_uniform(&mut self) -> f32 {
        let max: u64 = u32::MAX as u64 + 1;

        self.xor_shift() as f32 / max as f32
    }

    /// Returns a random f32 in [min, max)
    pub fn random_range(&mut self, min: f32, max: f32) -> f32 {
        min + (max - min) * self.random_uniform()
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
    fn random_uniform() {
        let mut rng = Rng::from_seed(512);

        let mut x;
        for _ in 0..10_000_000 {
            x = rng.random_uniform();
            assert!(x >= 0.0 && x < 1.0);
        }
    }

    #[test]
    fn random_range() {
        let mut rng = Rng::from_seed(727);

        let mut x;
        for _ in 0..10_000_000 {
            x = rng.random_range(-1.2, 5.3);
            assert!(x >= -1.2 && x < 5.3);
        }
    }
}
