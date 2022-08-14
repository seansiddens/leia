use crate::hittable::*;
use crate::ray::*;

pub struct HittableList {
    objects: Vec<Box<dyn Hittable>>,
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
    pub fn add<H: Hittable + 'static>(&mut self, object: H) {
        self.objects.push(Box::new(object));
    }
}

impl Hittable for HittableList {
    fn hit(&self, r: &Ray, t_min: f32, t_max: f32, rec: &mut HitRecord) -> bool {
        let mut temp_rec = HitRecord::new();
        temp_rec.t = t_max;
        let mut hit_anything = false;
        let mut closest_so_far = t_max;

        for obj in &self.objects {
            if obj.hit(r, t_min, closest_so_far, &mut temp_rec) {
                // Hit something!
                hit_anything = true;
                if temp_rec.t < closest_so_far {
                    // Record the closest hit.
                    closest_so_far = temp_rec.t;

                    rec.p = temp_rec.p;
                    rec.normal = temp_rec.normal;
                    rec.t = temp_rec.t;
                    rec.front_face = temp_rec.front_face;
                }
            }
        }

        hit_anything
    }
}
