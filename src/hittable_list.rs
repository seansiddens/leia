use std::sync::Arc;

use crate::hittable::*;
use crate::ray::*;

pub struct HittableList {
    objects: Vec<Box<dyn Hittable + Send + Sync>>,
}

#[allow(dead_code)]
impl HittableList {
    pub fn new() -> Self {
        let objects = Vec::new();
        Self { objects }
    }

    pub fn clear(&mut self) {
        self.objects.clear();
    }

    // TODO: Why do I need a static lifetime bound?
    pub fn add<H: Hittable + Send + Sync + 'static>(&mut self, object: H) {
        self.objects.push(Box::new(object));
    }
}

impl Hittable for HittableList {
    fn hit(&self, r: &Ray, t_min: f32, t_max: f32, rec: &mut HitPayload) -> bool {
        let mut temp_rec = HitPayload::new();
        temp_rec.hit_distance = t_max;
        let mut hit_anything = false;
        let mut closest_so_far = t_max;

        for (i, obj) in self.objects.iter().enumerate() {
            if obj.hit(r, t_min, closest_so_far, &mut temp_rec) {
                // Hit something!
                hit_anything = true;
                if temp_rec.hit_distance < closest_so_far {
                    // Record the closest hit.
                    closest_so_far = temp_rec.hit_distance;

                    rec.world_position = temp_rec.world_position;
                    rec.world_normal = temp_rec.world_normal;
                    rec.hit_distance = temp_rec.hit_distance;
                    rec.front_face = temp_rec.front_face;
                    rec.object_index = i;
                }
            }
        }

        hit_anything
    }
}
