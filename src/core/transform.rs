use crate::{Point3, Vector3};
use crate::core::geometry::Normal3;

// A simple 4x4 Matrix placeholder for demonstration
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Matrix4x4 {
    pub m: [[f32; 4]; 4],
}

impl Matrix4x4 {
    // Basic identity matrix
    pub fn identity() -> Self {
        Matrix4x4 {
            m: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    // Creating a matrix filled with NaNs for the "Poison" state
    pub fn new_nan() -> Self {
        let nan = f32::NAN;
        Matrix4x4 {
            m: [[nan; 4]; 4],
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Transform {
    m: Matrix4x4,
    m_inv: Matrix4x4,
}

impl Transform {
    // The Constructor: This is where we ensure Robustness [cite: 309]
    pub fn new(m: Matrix4x4) -> Self {
        // In a real implementation, we would calculate the actual inverse here.
        // For this lesson, we assume a function `try_inverse(&m)` exists.
        let inverse_result = try_inverse(&m); 

        match inverse_result {
            Some(inv) => Transform { m, m_inv: inv },
            None => {
                // ROBUSTNESS CHECK:
                // If the matrix is singular (non-invertible), we explicit poison the inverse.
                // Any calculation using this inverse will now propagate NaNs.
                Transform {
                    m,
                    m_inv: Matrix4x4::new_nan(),
                }
            }
        }
    }

    // Applying Transform to a Vector (w = 0) [cite: 303]
    // Translation is IGNORED because w is 0.
    pub fn transform_vector(&self, v: Vector3) -> Vector3 {
        let x = v.x; let y = v.y; let z = v.z;
        
        // Standard Matrix-Vector multiplication, but assuming w=0 implicitly
        Vector3 {
            x: self.m.m[0][0] * x + self.m.m[0][1] * y + self.m.m[0][2] * z,
            y: self.m.m[1][0] * x + self.m.m[1][1] * y + self.m.m[1][2] * z,
            z: self.m.m[2][0] * x + self.m.m[2][1] * y + self.m.m[2][2] * z,
        }
    }

    // Applying Transform to a Point (w = 1) [cite: 305, 306]
    // Translation is APPLIED. Also handles Projective Divide.
    pub fn transform_point(&self, p: Point3) -> Point3 {
        let x = p.x; let y = p.y; let z = p.z;
        
        // Implicit w = 1
        let xp = self.m.m[0][0]*x + self.m.m[0][1]*y + self.m.m[0][2]*z + self.m.m[0][3];
        let yp = self.m.m[1][0]*x + self.m.m[1][1]*y + self.m.m[1][2]*z + self.m.m[1][3];
        let zp = self.m.m[2][0]*x + self.m.m[2][1]*y + self.m.m[2][2]*z + self.m.m[2][3];
        let wp = self.m.m[3][0]*x + self.m.m[3][1]*y + self.m.m[3][2]*z + self.m.m[3][3];

        // ROBUSTNESS: Handle projective division
        if wp == 1.0 {
            Point3 { x: xp, y: yp, z: zp }
        } else {
            // If w is 0 (point at infinity) or very small, this could be unstable.
            // PBRT suggests checking for near-zero w here.
            Point3 { x: xp / wp, y: yp / wp, z: zp / wp }
        }
    }

    pub fn transform_normal(&self, n: Normal3) -> Normal3 {
        let x = n.x;
        let y = n.y;
        let z = n.z;

        // We use the columns of m_inv as the rows for the multiplication
        // This effectively multiplies by (m_inv)^T
        Normal3 {
            x: self.m_inv.m[0][0] * x + self.m_inv.m[1][0] * y + self.m_inv.m[2][0] * z,
            y: self.m_inv.m[0][1] * x + self.m_inv.m[1][1] * y + self.m_inv.m[2][1] * z,
            z: self.m_inv.m[0][2] * x + self.m_inv.m[1][2] * y + self.m_inv.m[2][2] * z,
        }
    }

    // We also need this helper for Instancing (World -> Object)
    pub fn inverse(&self) -> Transform {
        Transform {
            m: self.m_inv,
            m_inv: self.m,
        }
    }
}

// Mock function for inversion logic
fn try_inverse(m: &Matrix4x4) -> Option<Matrix4x4> {
    // In real code, calculate determinant. If det.abs() < 1e-6, return None.
    // For this example, let's assume if the first element is 0, it's singular.
    if m.m[0][0] == 0.0 {
        None
    } else {
        Some(Matrix4x4::identity()) // Mock success
    }
}