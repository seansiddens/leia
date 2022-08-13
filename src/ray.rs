use glam::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Ray {
    orig: Vec3,
    dir: Vec3,
}

impl Ray {
    /// Creates a new ray with a given origin and direction.
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self {
            orig: origin,
            dir: direction,
        }
    }

    pub fn origin(&self) -> Vec3 {
        self.orig
    }

    pub fn direction(&self) -> Vec3 {
        self.dir
    }
}