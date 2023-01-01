use glam::*;
use rand::Rng;
use std::f32::consts::PI;

const INV_PI: f32 = 1.0 / PI;

/// Generates a random vector about the z axis where z = (0.0, 0.0, 1.0).
pub fn uniform_hemisphere_sample(rng: &mut impl Rng) -> Vec3A {
    // Draw two uniform random numbers in [0.0, 1.0)
    let x1: f32 = rng.gen_range(0.0..1.0);
    let x2: f32 = rng.gen_range(0.0..1.0);

    let cos_theta = x1;
    let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
    let cos_phi = f32::cos(2.0 * PI * x2);
    let sin_phi = f32::sin(2.0 * PI * x2);

    vec3a(cos_phi * sin_theta, sin_phi * sin_theta, cos_theta)
}

#[cfg(test)]
mod test {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn random_test() {
        let mut rng = rand_xoshiro::Xoroshiro128PlusPlus::from_entropy();

        let mut w;
        for _ in 0..10_000_000 {
            w = super::uniform_hemisphere_sample(&mut rng);
            assert!(w.is_normalized()); // Random vectors are always normalized.
            assert!(w.z > 0.0); // Always upward-facing
        }
    }
}
