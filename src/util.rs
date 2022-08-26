use crate::rng::*;
use glam::*;

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
