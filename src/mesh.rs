use crate::{hittable::*, ray::*, triangle::*};
use easy_gltf::model::Mode;
use glam::*;
use std::error::Error;

// TODO: Handle other data (normals, UV, materials, etc).

/// For now, a Mesh is just a vector of Triangles.
#[derive(Debug)]
pub struct Mesh {
    // TODO: It would be more memory efficient to store an arrays of floats
    //       while also having an array of indices
    triangles: Vec<Triangle>,

    scale: Vec3,
    rotation: Quat,
    translation: Vec3,

    model_to_world: Mat4,
    world_to_model: Mat4,
}

impl Mesh {
    pub fn from_gltf(path: &str) -> Result<Self, Box<dyn Error>> {
        let mut triangles = Vec::new();
        let scenes = easy_gltf::load(path)?;
        for scene in scenes {
            for model in scene.models {
                match model.mode() {
                    Mode::TriangleFan | Mode::TriangleStrip | Mode::Triangles => {
                        for tri in model.triangles().unwrap() {
                            // Get vertex positions.
                            let v0 =
                                Vec3::new(tri[0].position.x, tri[0].position.y, tri[0].position.z);
                            let v1 =
                                Vec3::new(tri[1].position.x, tri[1].position.y, tri[1].position.z);
                            let v2 =
                                Vec3::new(tri[2].position.x, tri[2].position.y, tri[2].position.z);

                            triangles.push(Triangle::new(v0, v1, v2));
                        }
                    }
                    _ => panic!("Mesh must be a triangle mesh!"),
                }
            }
        }

        Ok(Self {
            triangles,

            scale: Vec3::ONE,
            rotation: Quat::IDENTITY,
            translation: Vec3::ZERO,
            model_to_world: Mat4::IDENTITY,
            world_to_model: Mat4::IDENTITY,
        })
    }

    pub fn num_triangles(&self) -> usize {
        self.triangles.len()
    }

    /// Set the translation for the mesh.
    pub fn translation(&mut self, translation: Vec3) {
        // Update transform.
        self.translation = translation;
        self.model_to_world =
            Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation);
        self.world_to_model = self.model_to_world.inverse();
    }

    pub fn transformation(&mut self, scale: Vec3, rotation: Quat, translation: Vec3) {
        self.scale = scale;
        self.rotation = rotation;
        self.translation = translation;
        self.model_to_world = Mat4::from_scale_rotation_translation(scale, rotation, translation);
        self.world_to_model = self.model_to_world.inverse();
    }
}

impl Hittable for Mesh {
    fn hit(&self, r: &Ray, t_min: f32, t_max: f32, rec: &mut HitRecord) -> bool {
        // Transform the ray to model space.
        let ray = Ray::new(
            self.world_to_model.transform_point3(r.origin()),
            self.world_to_model
                .transform_vector3(r.direction().normalize()),
        );

        let mut temp_rec = HitRecord::new();
        let mut hit_anything = false;
        let mut closest_so_far = t_max;

        for triangle in &self.triangles {
            if triangle.hit(&ray, t_min, closest_so_far, &mut temp_rec) {
                hit_anything = true;
                closest_so_far = temp_rec.t;

                rec.p = temp_rec.p;
                rec.normal = temp_rec.normal;
                rec.t = temp_rec.t;
                rec.front_face = temp_rec.front_face;
            }
        }

        // Transform the hit position and hit surface normal back to world space.
        rec.p = self.model_to_world.transform_point3(rec.p);
        rec.normal = self.model_to_world.transform_vector3(rec.normal);

        hit_anything
    }
}
