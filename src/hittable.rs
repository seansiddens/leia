use crate::{ray::*, vec3::*};

pub struct HitRecord {
    pub p: Point3,
    pub normal: Vec3,
    pub t: f32,
}

impl HitRecord {
    /// Create a new hit record.
    pub fn new() -> Self {
        Self {
            p: Point3::ZERO,
            normal: Point3::ZERO,
            t: 0.0,
        }
    }
}

pub trait Hittable {
    fn hit(&self, r: &Ray, t_min: f32, t_max: f32, rec: &mut HitRecord) -> bool;
}
