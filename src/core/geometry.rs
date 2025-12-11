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

// --- New Additions for Week 2 ---

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Normal3 {
    pub x: f32, pub y: f32, pub z: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point2 {
    pub x: f32, pub y: f32,
}

// Explicit conversion: Vector -> Normal
impl From<Vector3> for Normal3 {
    fn from(v: Vector3) -> Self {
        Normal3 { x: v.x, y: v.y, z: v.z }
    }
}

// Explicit conversion: Normal -> Vector (for math)
impl From<Normal3> for Vector3 {
    fn from(n: Normal3) -> Self {
        Vector3 { x: n.x, y: n.y, z: n.z }
    }
}

use std::ops::Neg;

impl Neg for Vector3 {
    type Output = Vector3;
    fn neg(self) -> Vector3 {
        Vector3 {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

// --- Bounding Box (AABB) for Acceleration Structures ---
#[derive(Debug, Clone, Copy)]
pub struct Bounds3 {
    pub min: Point3,
    pub max: Point3,
}

impl Bounds3 {
    // Create a box that encloses two points
    pub fn new(p1: Point3, p2: Point3) -> Self {
        Bounds3 {
            min: Point3 { x: p1.x.min(p2.x), y: p1.y.min(p2.y), z: p1.z.min(p2.z) },
            max: Point3 { x: p1.x.max(p2.x), y: p1.y.max(p2.y), z: p1.z.max(p2.z) },
        }
    }

    // Expand the box to include a third point
    pub fn union_point(self, p: Point3) -> Self {
        Bounds3 {
            min: Point3 { x: self.min.x.min(p.x), y: self.min.y.min(p.y), z: self.min.z.min(p.z) },
            max: Point3 { x: self.max.x.max(p.x), y: self.max.y.max(p.y), z: self.max.z.max(p.z) },
        }
    }
}

use std::ops::Mul;

// Enable "Vector * Float" (Scaling)
impl Mul<f32> for Vector3 {
    type Output = Vector3;
    fn mul(self, scalar: f32) -> Vector3 {
        Vector3 {
            x: self.x * scalar,
            y: self.y * scalar,
            z: self.z * scalar,
        }
    }
}

impl Vector3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Vector3 { x, y, z }
    }

    pub fn cross(self, other: Vector3) -> Vector3 {
        Vector3 {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    pub fn dot(self, other: Vector3) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn length_squared(self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub fn length(self) -> f32 {
        self.length_squared().sqrt()
    }

    pub fn normalize(self) -> Vector3 {
        let len = self.length();
        if len > 0.0 {
            self * (1.0 / len)
        } else {
            self // Handle zero vector gracefully
        }
    }
}

// Add this impl block for Point3
impl Point3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Point3 { x, y, z }
    }
}

// Add this impl for Vector subtraction
impl Sub for Vector3 {
    type Output = Vector3;
    fn sub(self, other: Vector3) -> Vector3 {
        Vector3 {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

// Allow explicit casting from Point3 to Vector3
impl From<Point3> for Vector3 {
    fn from(p: Point3) -> Self {
        Vector3 { x: p.x, y: p.y, z: p.z }
    }
}

