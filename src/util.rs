use glam::*;
use rand::Rng;
use std::f32::consts::PI;

const INV_PI: f32 = 1.0 / PI;

/// Generates a random vector on the hemissphere about the z axis
/// where z = (0.0, 0.0, 1.0).
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

/// Generates a random vector on the hemisphere in world space where n is the up direction.
pub fn uniform_hemisphere_sample_world(rng: &mut impl Rng, n: Vec3A) -> Vec3A {
    // Build an orthonormal basis from the surface normal.
    // Choose an arbitrary vector non-parallel to n.
    let a = if n.x.abs() > 0.9 {
        vec3a(0.0, 1.0, 0.0)
    } else {
        vec3a(1.0, 0.0, 0.0)
    };
    // Create vectors s and t which are orthongonal to n.
    let t = a.cross(n).normalize();
    let s = t.cross(n);

    // Create local-to-world matrix.
    // Since M is orthonormal, it's inverse is equal to it's transpose.
    let local_to_world = Mat3A::from_cols(s, t, n).transpose();

    // Generate random sample on hemisphere in local space and return it in 
    // world space.
    local_to_world * uniform_hemisphere_sample(rng)
}

pub fn random_in_unit_sphere(rng: &mut impl Rng) -> Vec3A {
    loop {
        let x: f32 = rng.gen_range(-1.0..1.0);
        let y: f32 = rng.gen_range(-1.0..1.0);
        let z: f32 = rng.gen_range(-1.0..1.0);
        let v = vec3a(x, y, z);
        if v.length_squared() >= 1.0 {
            continue;
        }
        return v;
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn uniform_hemisphere_sample_test() {
        let mut rng = rand_xoshiro::Xoroshiro128PlusPlus::from_entropy();

        let mut w;
        for _ in 0..100_000_000 {
            w = super::uniform_hemisphere_sample(&mut rng);
            assert!(w.is_normalized()); // Random vectors are always normalized.
            assert!(w.z >= 0.0); // Always upward-facing
        }
    }

    #[test]
    fn uniform_hemisphere_sample_world_test() {
        let mut rng = rand_xoshiro::Xoshiro128PlusPlus::from_entropy();

        // Set our surface normal.
        let n = vec3a(0.0, 1.0, 0.0);

        let mut w;
        for _ in 0..10_000_000 {
            w = super::uniform_hemisphere_sample_world(&mut rng, n);
            // Generated rays should be on same side of hemisphere as n.
            assert!(w.dot(n) >= 0.0); 
        }
    }
}
