use crate::ray::*;
use crate::Color;
use glam::*;

// TODO: Should this store material information?
pub struct HitPayload {
    pub world_position: Vec3A,
    pub world_normal: Vec3A,
    pub hit_distance: f32,
    pub front_face: bool, // Whether the hit was on the "front face" of the object.
    pub object_index: usize, // Index of the hittable object which was hit.
    pub albedo: Color,
    pub emissive: Color,
}

impl HitPayload {
    /// Create a new hit record.
    pub fn new() -> Self {
        Self {
            world_position: Vec3A::ZERO,
            world_normal: Vec3A::ZERO,
            hit_distance: -1.0,
            front_face: false,
            object_index: usize::MAX, // This represents an invalid index.
            albedo: Color::new(0.0, 1.0, 0.0), // TODO: Should the default be something else?
            emissive: Color::ZERO,
        }
    }

    /// We want normals recorded in hits to always be opposite of incident rays.
    /// This function will determine whether a hit occurred on the front or back face
    /// of an object, and will ensure that the recorded normal of the hit is opposite
    /// of the incident ray.
    pub fn set_face_normal(&mut self, r: &Ray, object_normal: Vec3A) {
        // Determine if hit was on front or back face.
        self.front_face = Vec3A::dot(r.direction(), object_normal) < 0.0;

        // Set normal to point against incident ray.
        self.world_normal = if self.front_face {
            object_normal
        } else {
            -object_normal
            // vec3(1.0, 0.0, 0.0)
        };
    }
}

pub trait Hittable {
    fn hit(&self, r: &Ray, t_min: f32, t_max: f32, rec: &mut HitPayload) -> bool;
}
