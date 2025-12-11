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

    pub fn inverse(&self) -> Option<Matrix4x4> {
        let m = self.m;
        
        // 1. Calculate the Determinant
        // (Using the expansion by minors for the first row)
        // Sub-determinants for 3x3 matrices
        let s0 = m[0][0] * m[1][1] - m[1][0] * m[0][1];
        let s1 = m[0][0] * m[1][2] - m[1][0] * m[0][2];
        let s2 = m[0][0] * m[1][3] - m[1][0] * m[0][3];
        let s3 = m[0][1] * m[1][2] - m[1][1] * m[0][2];
        let s4 = m[0][1] * m[1][3] - m[1][1] * m[0][3];
        let s5 = m[0][2] * m[1][3] - m[1][2] * m[0][3];

        let c5 = m[2][2] * m[3][3] - m[3][2] * m[2][3];
        let c4 = m[2][1] * m[3][3] - m[3][1] * m[2][3];
        let c3 = m[2][1] * m[3][2] - m[3][1] * m[2][2];
        let c2 = m[2][0] * m[3][3] - m[3][0] * m[2][3];
        let c1 = m[2][0] * m[3][2] - m[3][0] * m[2][2];
        let c0 = m[2][0] * m[3][1] - m[3][0] * m[2][1];

        let det = s0 * c5 - s1 * c4 + s2 * c3 + s3 * c2 - s4 * c1 + s5 * c0;

        if det.abs() < 1e-6 {
            return None; // Singular matrix
        }

        let inv_det = 1.0 / det;

        // 2. Calculate the Adjugate Matrix scaled by 1/det
        let mut inv = [[0.0; 4]; 4];

        inv[0][0] = ( m[1][1] * c5 - m[1][2] * c4 + m[1][3] * c3) * inv_det;
        inv[0][1] = (-m[0][1] * c5 + m[0][2] * c4 - m[0][3] * c3) * inv_det;
        inv[0][2] = ( m[3][1] * s5 - m[3][2] * s4 + m[3][3] * s3) * inv_det;
        inv[0][3] = (-m[2][1] * s5 + m[2][2] * s4 - m[2][3] * s3) * inv_det;

        inv[1][0] = (-m[1][0] * c5 + m[1][2] * c2 - m[1][3] * c1) * inv_det;
        inv[1][1] = ( m[0][0] * c5 - m[0][2] * c2 + m[0][3] * c1) * inv_det;
        inv[1][2] = (-m[3][0] * s5 + m[3][2] * s2 - m[3][3] * s1) * inv_det;
        inv[1][3] = ( m[2][0] * s5 - m[2][2] * s2 + m[2][3] * s1) * inv_det;

        inv[2][0] = ( m[1][0] * c4 - m[1][1] * c2 + m[1][3] * c0) * inv_det;
        inv[2][1] = (-m[0][0] * c4 + m[0][1] * c2 - m[0][3] * c0) * inv_det;
        inv[2][2] = ( m[3][0] * s4 - m[3][1] * s2 + m[3][3] * s0) * inv_det;
        inv[2][3] = (-m[2][0] * s4 + m[2][1] * s2 - m[2][3] * s0) * inv_det;

        inv[3][0] = (-m[1][0] * c3 + m[1][1] * c1 - m[1][2] * c0) * inv_det;
        inv[3][1] = ( m[0][0] * c3 - m[0][1] * c1 + m[0][2] * c0) * inv_det;
        inv[3][2] = (-m[3][0] * s3 + m[3][1] * s1 - m[3][3] * s0) * inv_det;
        inv[3][3] = ( m[2][0] * s3 - m[2][1] * s1 + m[2][2] * s0) * inv_det;

        Some(Matrix4x4 { m: inv })
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
// Update the helper function to actually use it
fn try_inverse(m: &Matrix4x4) -> Option<Matrix4x4> {
    m.inverse()
}