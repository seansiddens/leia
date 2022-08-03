use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Copy, Clone, Debug)]
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

    /// Returns the dot produt of two vectors.
    pub fn dot(v: Self, u: Self) -> f32 {
        v.x * u.x + v.y * u.y + v.z * u.z
    }

    /// Returns the cross prodct between two vectors.
    pub fn cross(v: Self, u: Self) -> Self {
        Self {
            x: v.y * u.z - v.z * u.y,
            y: v.z * u.x - v.x * u.z,
            z: v.x * u.y - v.y * u.x,
        }
    }

    /// Return the length of the vector.
    pub fn length(self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    /// Returns the squared length of the vector.
    pub fn length_squared(self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    // Normalizes the vector.
    pub fn normalize(self) -> Self {
        self / self.length()
    }

    // Normalizes a vector.
    pub fn normalized(v: Self) -> Self {
        v / v.length()
    }
}

// Operators ----------------------------------------------------------------------
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

// Add Assignment
impl AddAssign for Vec3 {
    fn add_assign(&mut self, rhs: Self) {
        *self = Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
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

// Scalar multiplication.
// LHS: Vec3, RHS: f32
impl Mul<f32> for Vec3 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

// LHS: f32, RHS: Vec3
impl Mul<Vec3> for f32 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Vec3 {
        Vec3 {
            x: self * rhs.x,
            y: self * rhs.y,
            z: self * rhs.z,
        }
    }
}

// Scalar multiplication assignment
impl MulAssign<f32> for Vec3 {
    fn mul_assign(&mut self, rhs: f32) {
        *self = Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        };
    }
}

// Scalar division.
// LHS: Vec3, RHS: f32
impl Div<f32> for Vec3 {
    type Output = Self;

    fn div(self, rhs: f32) -> Self {
        self * (1.0 / rhs)
    }
}

// Scalar division Assignment
impl DivAssign<f32> for Vec3 {
    fn div_assign(&mut self, rhs: f32) {
        *self *= 1.0 / rhs
    }
}

// Negation
impl Neg for Vec3 {
    type Output = Vec3;

    fn neg(self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

// Tests ---------------------------------------------------------------------------------
#[cfg(test)]
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

    #[test]
    fn mul() {
        let v = Vec3::new(2.0, -3.0, 7.0);
        let a = -19.3;

        assert!(
            v * a
                == Vec3 {
                    x: v.x * a,
                    y: v.y * a,
                    z: v.z * a
                }
        );
        assert!(
            a * v
                == Vec3 {
                    x: v.x * a,
                    y: v.y * a,
                    z: v.z * a
                }
        );
    }

    #[test]
    fn div() {
        let mut v = Vec3::new(4.0, 2.0, -5.0);

        assert!(
            v / 2.0
                == Vec3 {
                    x: 2.0,
                    y: 1.0,
                    z: -2.5
                }
        );

        v /= 2.0;
        assert!(
            v == Vec3 {
                x: 2.0,
                y: 1.0,
                z: -2.5
            }
        );
    }

    #[test]
    fn neg() {
        let v = Vec3::new(-3.0, 2.0, 12313.324234);

        assert!(
            -v == Vec3 {
                x: 3.0,
                y: -2.0,
                z: -12313.324234
            }
        );
        assert!(
            -Vec3::ZERO
                == Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0
                }
        );

        {
            let mut v = Vec3::ONE;
            v *= 5.9;
            assert!(
                v == Vec3 {
                    x: 5.9,
                    y: 5.9,
                    z: 5.9
                }
            );
        }
    }

    #[test]
    fn length() {
        let v = Vec3::new(3.0, -19.9, 28.0);

        assert!(v.length() == (v.x * v.x + v.y * v.y + v.z * v.z).sqrt());
        assert!(v.length_squared() == (v.x * v.x + v.y * v.y + v.z * v.z));
    }

    #[test]
    fn dot() {
        let a = Vec3::new(2.0, 3.0, 5.0);
        let b = Vec3::new(7.0, 11.0, 13.0);
        assert_eq!(Vec3::dot(a, b), 2.0 * 7.0 + 3.0 * 11.0 + 5.0 * 13.0);
    }

    #[test]
    fn cross() {
        let i = Vec3::unit(Axis::X);
        let j = Vec3::unit(Axis::Y);

        assert!(Vec3::cross(i, j) == Vec3::unit(Axis::Z));
    }

    #[test]
    fn normalize() {
        let v = Vec3::new(3242.432, 2343.234, 1231.14);
        assert!(v.normalize().length() <= 1.0);
        assert!(Vec3::normalized(v).length() <= 1.0);
    }
}
