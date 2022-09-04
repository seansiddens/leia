use std::f32::consts::PI;

use crate::rng::*;
use glam::*;

const INV_PI: f32 = 1.0 / PI;

///
/// Returns a random unit vector.
///
/// True Lambertian reflectance still has higher probability near the normal, but
/// the distribution is more uniform. This can be achieved by picking random
/// points ON the unit sphere, which is done by normalizing the vector WITHIN the
/// unit sphere. Diffuse objects will appear lighter due to more light bouncing
/// towards the camera. Shadows will also appear less pronounced due to less
/// light bouncing directly up from objects directly underneath other objects.
///
pub fn random_unit_vector(rng: &mut Rng) -> Vec3 {
    random_in_unit_sphere(rng).normalize()
}

///
/// Returns a random point in the unit sphere.
///
/// Approximation of Lambertian reflectance
/// We pick a random point the unit cube until we get a valid point within the
/// unit sphere.
/// Probability is higher close to the normal (scales w/ cos^3(Φ), where Φ is the
/// angle from the normal).
///
pub fn random_in_unit_sphere(rng: &mut Rng) -> Vec3 {
    let mut p;
    loop {
        p = vec3(
            rng.random_range(-1.0, 1.0),
            rng.random_range(-1.0, 1.0),
            rng.random_range(-1.0, 1.0),
        );

        if p.length_squared() < 1.0 {
            break;
        }
    }
    p
}

/// Returns a cosine-weighted random vector given two uniform rnadom numbers r1 and r2.
pub fn cosine_sample_hemisphere(r1: f32, r2: f32) -> Vec3 {
    let u = (1.0 - r2).sqrt();
    let theta = 2.0 * PI * r1;
    let x = f32::cos(theta) * u;
    let y = f32::sin(theta) * u;
    let z = r2.sqrt();
    Vec3 { x, y, z }
}

pub fn cosine_hemisphere_pdf(cos_theta: f32) -> f32 {
    cos_theta * INV_PI
}
