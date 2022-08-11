use crate::{ray::*, vec3::*};

pub struct HitRecord {
    pub p: Point3,
    pub normal: Vec3,
    pub t: f32,
    pub front_face: bool, // Whether the hit was on the "front face" of the object.
}

impl HitRecord {
    /// Create a new hit record.
    pub fn new() -> Self {
        Self {
            p: Point3::ZERO,
            normal: Point3::ZERO,
            t: 0.0,
            front_face: false,
        }
    }

    /// We want normals recorded in hits to always be opposite of incident rays.
    /// This function will determine whether a hit occurred on the front or back face
    /// of an object, and will ensure that the recorded normal of the hit is opposite
    /// of the incident ray.
    pub fn set_face_normal(&mut self, r: &Ray, object_normal: Vec3) {
        // Determine if hit was on front or back face.
        self.front_face = Vec3::dot(r.direction(), object_normal) < 0.0;

        // Set normal to point against incident ray.
        self.normal = if self.front_face {
            object_normal
        } else {
            -object_normal
        };
    }
}

pub trait Hittable {
    fn hit(&self, r: &Ray, t_min: f32, t_max: f32, rec: &mut HitRecord) -> bool;
}
