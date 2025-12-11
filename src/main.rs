pub mod geometry;
pub mod transform;
pub mod math;

mod core; // Tells Rust to look for the 'core' folder

use crate::core::geometry::{Point3, Vector3};
use crate::core::transform::{Transform, Matrix4x4};

fn main() {
    println!("--- Week 1 Day 1 Verification ---");

    // 1. Test Type Safety
    let p1 = Point3 { x: 0.0, y: 0.0, z: 0.0 };
    let v1 = Vector3 { x: 1.0, y: 2.0, z: 3.0 };
    
    let p2 = p1 + v1; // Point + Vector -> Point
    println!("Type Safety Test: p1({:?}) + v1({:?}) = p2({:?})", p1, v1, p2);
    // let p3 = p1 + p2; // This would fail to compile!

    // 2. Test Degenerate Matrix (The Poison State)
    println!("\n--- Testing Robustness (Poison State) ---");
    
    // Create a "bad" matrix (first element 0 triggers our mock singular check)
    let bad_matrix_data = Matrix4x4 { 
        m: [[0.0; 4]; 4] 
    };
    
    let transform_bad = Transform::new(bad_matrix_data);
    
    // We expect the Inverse to be poisoned with NaNs [cite: 309]
    println!("Degenerate Transform Inverse: {:?}", transform_bad);

    // If we try to use the inverse (e.g., for normal transformation), 
    // the result should be NaN, alerting us to the bug immediately.
    // (In a full implementation, we would access transform_bad.m_inv to transform a normal)
}