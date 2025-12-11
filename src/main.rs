mod core; // This tells Rust: "Go check src/core/mod.rs"

// Now we import from the core module we just declared
use crate::core::geometry::{Point3, Vector3};
use crate::core::transform::{Transform, Matrix4x4};

fn main() {
    println!("--- Week 1 Day 1 Verification ---");

    // 1. Test Type Safety
    let p1 = Point3 { x: 0.0, y: 0.0, z: 0.0 };
    let v1 = Vector3 { x: 1.0, y: 2.0, z: 3.0 };
    
    let p2 = p1 + v1; // Point + Vector -> Point
    println!("Type Safety Test: p1({:?}) + v1({:?}) = p2({:?})", p1, v1, p2);

    // 2. Test Degenerate Matrix (The Poison State)
    println!("\n--- Testing Robustness (Poison State) ---");
    
    let bad_matrix_data = Matrix4x4 { 
        m: [[0.0; 4]; 4] 
    };
    
    let transform_bad = Transform::new(bad_matrix_data);
    
    // We expect the Inverse to be poisoned with NaNs
    println!("Degenerate Transform Inverse: {:?}", transform_bad);
}