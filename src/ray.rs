use crate::vec3::{Point3, Vec3};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Ray {
    orig: Point3,
    dir: Vec3,
}

impl Ray {
    /// Creates a new ray with a given origin and direction.
    pub fn new(origin: Point3, direction: Vec3) -> Self {
        Self {
            orig: origin,
            dir: direction,
        }
    }

    pub fn origin(&self) -> Point3 {
        self.orig
    }

    pub fn direction(&self) -> Vec3 {
        self.dir
    }
}
