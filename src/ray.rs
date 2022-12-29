use glam::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Ray {
    orig: Vec3A,
    dir: Vec3A,
}

impl Ray {
    /// Creates a new ray with a given origin and direction.
    pub fn new(origin: Vec3A, direction: Vec3A) -> Self {
        Self {
            orig: origin,
            dir: direction,
        }
    }

    pub fn origin(&self) -> Vec3A {
        self.orig
    }

    pub fn set_origin(&mut self, origin: Vec3A) {
        self.orig = origin;
    }

    pub fn direction(&self) -> Vec3A {
        self.dir
    }

    pub fn set_direction(&mut self, direction: Vec3A) {
        self.dir = direction;
    }
}
