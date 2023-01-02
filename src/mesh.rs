use crate::{bvh::*, hittable::*, ray::*, triangle::*, Color};
use easy_gltf::model::Mode;
use glam::*;
use rand::{Rng, SeedableRng};
use std::error::Error;

// TODO: Handle other data (normals, UV, materials, etc).

/// For now, a Mesh is just a vector of Triangles.
#[derive(Debug)]
pub struct Mesh {
    // TODO: It would be more memory efficient to store an arrays of floats
    //       while also having an array of indices
    triangles: Vec<Triangle>,
    bvh: Bvh,

    scale: Vec3A,
    rotation: Quat,
    translation: Vec3A,

    model_to_world: Affine3A,
    world_to_model: Affine3A,
}

#[allow(dead_code)]
impl Mesh {
    pub fn from_gltf(path: &str) -> Result<Self, Box<dyn Error>> {
        // TODO: Change this. Currently just setting triangles to random colors.
        let mut rng = rand_xoshiro::Xoroshiro128PlusPlus::from_entropy();

        let mut triangles = Vec::new();
        let scenes = easy_gltf::load(path)?;
        for scene in scenes {
            for model in scene.models {
                match model.mode() {
                    Mode::TriangleFan | Mode::TriangleStrip | Mode::Triangles => {
                        for tri in model.triangles().unwrap() {
                            // Get vertex positions.
                            let v0 =
                                Vec3A::new(tri[0].position.x, tri[0].position.y, tri[0].position.z);
                            let v1 =
                                Vec3A::new(tri[1].position.x, tri[1].position.y, tri[1].position.z);
                            let v2 =
                                Vec3A::new(tri[2].position.x, tri[2].position.y, tri[2].position.z);

                            // let albedo = Color::new(
                            //     rng.gen_range(0.0..1.0),
                            //     rng.gen_range(0.0..1.0),
                            //     rng.gen_range(0.0..1.0),
                            // );
                            // let albedo = Color::new(1.0, 0.0, 0.0);
                            let material = model.material();
                            let albedo = material.get_base_color(tri[0].tex_coords);
                            let emissive = material.get_emissive(tri[0].tex_coords);

                            triangles.push(Triangle::new(
                                v0,
                                v1,
                                v2,
                                Color::new(albedo.x, albedo.y, albedo.z),
                                Color::new(emissive.x, emissive.y, emissive.z),
                            ));
                        }
                    }
                    _ => panic!("Mesh must be a triangle mesh!"),
                }
            }
        }

        // Build bvh from triangles.
        let bvh = Bvh::new(&triangles);

        Ok(Self {
            triangles,
            bvh,
            scale: Vec3A::ONE,
            rotation: Quat::IDENTITY,
            translation: Vec3A::ZERO,
            model_to_world: Affine3A::IDENTITY,
            world_to_model: Affine3A::IDENTITY,
        })
    }

    pub fn from_triangles(triangles: Vec<Triangle>) -> Self {
        let bvh = Bvh::new(&triangles);
        Self {
            triangles,
            bvh,
            scale: Vec3A::ONE,
            rotation: Quat::IDENTITY,
            translation: Vec3A::ZERO,
            model_to_world: Affine3A::IDENTITY,
            world_to_model: Affine3A::IDENTITY,
        }
    }

    pub fn num_triangles(&self) -> usize {
        self.triangles.len()
    }

    /// Set the translation for the mesh.
    pub fn translation(&mut self, translation: Vec3A) {
        // Update transform.
        self.translation = translation;
        self.model_to_world = Affine3A::from_scale_rotation_translation(
            self.scale.into(),
            self.rotation.into(),
            self.translation.into(),
        );
        self.world_to_model = self.model_to_world.inverse();
    }

    pub fn transformation(&mut self, scale: Vec3A, rotation: Quat, translation: Vec3A) {
        self.scale = scale;
        self.rotation = rotation;
        self.translation = translation;
        self.model_to_world = Affine3A::from_scale_rotation_translation(
            scale.into(),
            rotation.into(),
            translation.into(),
        );
        self.world_to_model = self.model_to_world.inverse();
    }
}

impl Hittable for Mesh {
    fn hit(&self, r: &Ray, t_min: f32, t_max: f32, rec: &mut HitPayload) -> bool {
        // Transform the ray to model space.
        let ray = Ray::new(
            self.world_to_model.transform_point3a(r.origin()),
            self.world_to_model
                .transform_vector3a(r.direction().normalize()),
        );

        let use_bvh = true;
        if use_bvh {
            let hit_anything = if self.bvh.hit(&ray, t_min, t_max, rec) {
                // Transform the hit position and hit surface normal back to world space.
                rec.world_position = self.model_to_world.transform_point3a(rec.world_position);
                rec.world_normal = self.model_to_world.transform_vector3a(rec.world_normal);

                true
            } else {
                false
            };
            hit_anything
        } else {
            let hit_anything = false;
            let mut temp_rec = HitPayload::new();
            let mut hit_anything = false;
            let mut closest_so_far = t_max;

            for triangle in &self.triangles {
                if triangle.hit(&ray, t_min, closest_so_far, &mut temp_rec) {
                    hit_anything = true;
                    closest_so_far = temp_rec.hit_distance;

                    rec.world_position = temp_rec.world_position;
                    rec.world_normal = temp_rec.world_normal;
                    rec.hit_distance = temp_rec.hit_distance;
                    rec.front_face = temp_rec.front_face;
                }
            }
            // Transform the hit position and hit surface normal back to world space.
            rec.world_position = self.model_to_world.transform_point3a(rec.world_position);
            rec.world_normal = self.model_to_world.transform_vector3a(rec.world_normal);

            hit_anything
        }
    }
}
