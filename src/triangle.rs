use crate::{hittable::*, ray::*, Color};
use glam::*;

/// Triangle's vertices are defined in CCW winding.
#[derive(Debug)]
pub struct Triangle {
    v0: Vec3A,
    v1: Vec3A,
    v2: Vec3A,
    normal: Vec3A,   // Triangle's surface normal.
    centroid: Vec3A, // Used for BVH construction.
    albedo: Color,
    emissive: Color,
}

impl Triangle {
    pub fn new(v0: Vec3A, v1: Vec3A, v2: Vec3A, albedo: Color, emissive: Color) -> Self {
        // Compute the surface normal of the plane defined by the triangle.
        let normal = Vec3A::cross(v1 - v0, v2 - v0).normalize();

        let centroid = (v0 + v1 + v2) * (1.0 / 3.0);

        Self {
            v0,
            v1,
            v2,
            normal,
            centroid,
            albedo,
            emissive,
        }
    }

    pub fn vertices(&self) -> [Vec3A; 3] {
        [self.v0, self.v1, self.v2]
    }

    pub fn centroid(&self) -> Vec3A {
        self.centroid
    }

    pub fn albedo(&self) -> Color {
        self.albedo
    }

    pub fn emissive(&self) -> Color {
        self.emissive
    }
}

impl Hittable for Triangle {
    /// Calculate ray-triangle intersection using the Möller-Trumbore algorithm.
    /// Source: https://en.wikipedia.org/wiki/M%C3%B6ller%E2%80%93Trumbore_intersection_algorithm
    fn hit(&self, r: &Ray, t_min: f32, t_max: f32, rec: &mut HitPayload) -> bool {
        let r_dir = r.direction();
        let r_orig = r.origin();

        let edge1 = self.v1 - self.v0;
        let edge2 = self.v2 - self.v0;

        let h = r_dir.cross(edge2);
        let a = edge1.dot(h);
        if a > -f32::EPSILON && a < f32::EPSILON {
            return false; // Ray is parallel to this triangle.
        }

        let f = 1.0 / a;
        let s = r_orig - self.v0;
        let u = f * s.dot(h);
        if u < 0.0 || u > 1.0 {
            return false;
        }

        let q = s.cross(edge1);
        let v = f * r_dir.dot(q);
        if v < 0.0 || u + v > 1.0 {
            return false;
        }
        // u and v are barycentric coordinates of the hit.

        // At this stage we can compute t to find out where the intersection is on the line.
        let t = f * edge2.dot(q);
        if t < t_min || t > t_max {
            return false;
        }

        // Record hit information
        rec.hit_distance = t;
        rec.world_position = r_orig + t * r_dir;
        // rec.normal = self.normal;
        rec.set_face_normal(r, self.normal);
        rec.albedo = self.albedo;
        rec.emissive = self.emissive;

        true
    }

    // Inside-outside intersection test.
    // fn hit(&self, r: &Ray, t_min: f32, t_max: f32, rec: &mut HitRecord) -> bool {
    //     // Check if the ray is parallel to the plane.
    //     if Vec3::dot(self.normal, r.direction()) == 0.0 {
    //         return false;
    //     }

    //     // Calculate where the ray interesects the plane.
    //     let t =
    //         -(Vec3::dot(self.normal, r.origin()) + self.d) / Vec3::dot(self.normal, r.direction());
    //     // Check if triangle is behind the ray.
    //     if t < t_min || t > t_max {
    //         return false;
    //     }
    //     let p = r.origin() + t * r.direction();

    //     // Determine if intersection point is in triangle using the inside-outside test.
    //     // AB edge.
    //     let edge0 = self.v1 - self.v0;
    //     if Vec3::dot(Vec3::cross(edge0, p - self.v0), self.normal) < 0.0 {
    //         return false;
    //     }
    //     // BC edge.
    //     let edge1 = self.v2 - self.v1;
    //     if Vec3::dot(Vec3::cross(edge1, p - self.v1), self.normal) < 0.0 {
    //         return false;
    //     }
    //     // CA edge.
    //     let edge2 = self.v0 - self.v2;
    //     if Vec3::dot(Vec3::cross(edge2, p - self.v2), self.normal) < 0.0 {
    //         return false;
    //     }

    //     // Record hit information
    //     rec.t = t;
    //     rec.p = p;
    //     rec.set_face_normal(r, self.normal);

    //     true
    // }
}
