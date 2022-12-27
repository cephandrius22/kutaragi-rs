use glam::{Vec2, Vec3, Vec4};

pub struct Triangle {
    pub v1: Vec3,
    pub v2: Vec3,
    pub v3: Vec3,
}

impl Triangle {
    pub fn new(v1: Vec3, v2: Vec3, v3: Vec3) -> Triangle {
        return Triangle { v1, v2, v3 };
    }
}

pub fn perspective_divide(v: Vec4) -> Vec2 {
    Vec2::new(v.x / v.z, v.y / v.z)
}
