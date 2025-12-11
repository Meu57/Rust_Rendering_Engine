use std::ops::{Add, Sub};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

// Rule 1: Point + Vector = Point (Moving a location) [cite: 302, 303]
impl Add<Vector3> for Point3 {
    type Output = Point3;
    fn add(self, v: Vector3) -> Point3 {
        Point3 {
            x: self.x + v.x,
            y: self.y + v.y,
            z: self.z + v.z,
        }
    }
}

// Rule 2: Point - Point = Vector (Distance/Direction between locations) [cite: 302]
impl Sub<Point3> for Point3 {
    type Output = Vector3;
    fn sub(self, other: Point3) -> Vector3 {
        Vector3 {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

// Rule 3: Vector + Vector = Vector (Accumulating forces/offsets)
impl Add<Vector3> for Vector3 {
    type Output = Vector3;
    fn add(self, other: Vector3) -> Vector3 {
        Vector3 {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

// NOTE: We do NOT implement Add<Point3> for Point3. 
// Attempting "p1 + p2" will now cause a compile-time error.