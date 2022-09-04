use glam::*;

pub type Onb = OrthonormalBasis;

pub struct OrthonormalBasis {
    u: Vec3,
    v: Vec3,
    w: Vec3,
}

impl OrthonormalBasis {
    pub fn from_w(n: Vec3) -> Self {
        let w = n.normalize();
        // a cannnot be parallel to the axis we choose.
        let a = if w.x.abs() > 0.9 {
            vec3(0.0, 1.0, 0.0)
        } else {
            vec3(1.0, 0.0, 0.0)
        };
        let v = w.cross(a).normalize();
        let u = w.cross(v);

        Self { u, v, w }
    }

    pub fn local(&self, a: Vec3) -> Vec3 {
        a.x * self.u + a.y * self.v + a.z * self.w
    }
}
