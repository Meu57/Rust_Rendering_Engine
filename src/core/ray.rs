use crate::core::geometry::{Point3, Vector3};

#[derive(Debug, Clone, Copy)]
pub struct Ray {
    pub o: Point3,   // Origin
    pub d: Vector3,  // Direction
    pub time: f32,   // For Motion Blur
    // We will add 'medium' later when we tackle Volumetrics
}

impl Ray {
    pub fn new(o: Point3, d: Vector3, time: f32) -> Self {
        Ray { o, d, time }
    }

    // Calculate position at distance t
    pub fn at(&self, t: f32) -> Point3 {
        self.o + Vector3 {
            x: self.d.x * t,
            y: self.d.y * t,
            z: self.d.z * t,
        }
    }
}