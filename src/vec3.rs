use std::ops::{Add, Sub};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

pub enum Axis {
    X,
    Y,
    Z,
}

impl Vec3 {
    pub const ZERO: Vec3 = Vec3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };

    pub const ONE: Vec3 = Vec3 {
        x: 1.0,
        y: 1.0,
        z: 1.0,
    };

    /// Creates a new three-component vector.
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Creates a unit vector along a given axis.
    pub fn unit(axis: Axis) -> Self {
        let mut ret = Vec3::ZERO;
        match axis {
            Axis::X => ret.x = 1.0,
            Axis::Y => ret.y = 1.0,
            Axis::Z => ret.z = 1.0,
        }

        ret
    }
}

// Binary Operators ----------------------------------------------------------------------
// Addition
// LHS: Vec3, RHS: Vec3
impl Add for Vec3 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

// LHS: &Vec3, RHS: &Vec3
impl Add for &Vec3 {
    type Output = Vec3;

    fn add(self, other: Self) -> Vec3 {
        Vec3 {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

// Subtraction
// LHS: Vec3, RHS: Vec3
impl Sub for Vec3 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

// LHS: &Vec3, RHS: &Vec3
impl Sub for &Vec3 {
    type Output = Vec3;

    fn sub(self, other: Self) -> Vec3 {
        Vec3 {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

// Tests ---------------------------------------------------------------------------------
mod tests {
    use super::*;

    #[test]
    fn add() {
        let v = Vec3::new(3.0, -1.0, 4.0);
        let u = Vec3::new(-2.0, 7.0, 6.0);

        assert!(
            v + u
                == Vec3 {
                    x: v.x + u.x,
                    y: v.y + u.y,
                    z: v.z + u.z
                }
        );

        assert!(
            &v + &u
                == Vec3 {
                    x: v.x + u.x,
                    y: v.y + u.y,
                    z: v.z + u.z
                }
        );
        assert!(
            v + Vec3::ONE
                == Vec3 {
                    x: v.x + 1.0,
                    y: v.y + 1.0,
                    z: v.z + 1.0
                }
        );
    }

    #[test]
    fn sub() {
        let v = Vec3::new(2.5, 6.7, -200.99);
        let u = Vec3::new(-17.234, 39.9, -20394.0);
        assert!(
            v - u
                == Vec3 {
                    x: v.x - u.x,
                    y: v.y - u.y,
                    z: v.z - u.z
                }
        );
        assert!(
            &v - &u
                == Vec3 {
                    x: v.x - u.x,
                    y: v.y - u.y,
                    z: v.z - u.z
                }
        );
        assert!(
            v - Vec3::ONE
                == Vec3 {
                    x: v.x - 1.0,
                    y: v.y - 1.0,
                    z: v.z - 1.0
                }
        );
    }

    #[test]
    fn unit() {
        let v = Vec3::unit(Axis::X);
        assert!(
            v == Vec3 {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
        );
        let u = Vec3::unit(Axis::Y);
        assert!(
            u == Vec3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
        );
        let w = Vec3::unit(Axis::Z);
        assert!(
            w == Vec3 {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
        );
    }
}
