use crate::{hittable::*, ray::*, vec3::*};

/// Triangle's vertices are defined in CCW winding.
pub struct Triangle {
    v0: Point3,
    v1: Point3,
    v2: Point3,
    normal: Vec3, // Triangle's surface normal.
    d: f32,       // Distance from origin to the plane.
}

impl Triangle {
    pub fn new(v0: Point3, v1: Point3, v2: Point3) -> Self {
        // Compute the surface normal of the plane defined by the triangle.
        let normal = Vec3::cross(v1 - v0, v2 - v0).normalize();
        // Distance from the origin to the plane.
        let d = -Vec3::dot(normal, v0);

        Self {
            v0,
            v1,
            v2,
            normal,
            d,
        }
    }
}

impl Hittable for Triangle {
    // TODO: Implement Möller-Trumbore algorithm
    fn hit(&self, r: &Ray, t_min: f32, t_max: f32, rec: &mut HitRecord) -> bool {
        // Check if the ray is parallel to the plane.
        if Vec3::dot(self.normal, r.direction()) == 0.0 {
            return false;
        }

        // Calculate where the ray interesects the plane.
        let t =
            -(Vec3::dot(self.normal, r.origin()) + self.d) / Vec3::dot(self.normal, r.direction());
        // Check if triangle is behind the ray.
        if t < t_min || t > t_max {
            return false;
        }
        let p = r.origin() + t * r.direction();

        // Determine if intersection point is in triangle using the inside-outside test.
        // AB edge.
        let edge0 = self.v1 - self.v0;
        if Vec3::dot(Vec3::cross(edge0, p - self.v0), self.normal) < 0.0 {
            return false;
        }
        // BC edge.
        let edge1 = self.v2 - self.v1;
        if Vec3::dot(Vec3::cross(edge1, p - self.v1), self.normal) < 0.0 {
            return false;
        }
        // CA edge.
        let edge2 = self.v0 - self.v2;
        if Vec3::dot(Vec3::cross(edge2, p - self.v2), self.normal) < 0.0 {
            return false;
        }

        // Record hit information
        rec.t = t;
        rec.p = p;
        rec.normal = self.normal;

        true
    }
}
