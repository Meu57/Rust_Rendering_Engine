mod core; // This tells Rust: "Go check src/core/mod.rs"

// Now we import from the core module we just declared
use crate::core::geometry::{Point3, Vector3};
use crate::core::transform::{Transform, Matrix4x4};
use crate::core::math::{Interval, solve_quadratic};

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


    println!("--- Week 1 Day 5: Numerical Robustness ---");

    // 1. Demonstrate Catastrophic Cancellation
    let a = 1.0;
    let b = 0.99999994; // Just enough difference to be tricky in f32
    let naive_diff = a - b;
    println!("Naive Float Subtraction: {:.20}", naive_diff);
    // You'll likely see garbage digits at the end if you print enough precision

    // 2. Demonstrate Interval "Safety Box"
    let i_a = Interval::new(a);
    let i_b = Interval::new(b);
    let i_diff = i_a - i_b;

    println!("Interval Subtraction:   [{:.20}, \n                         {:.20}]", i_diff.min, i_diff.max);
    
    // Check if the interval contains the result (naive_diff)
    // In a real scenario, we'd check against the 'true' mathematical value (f64)
    if naive_diff >= i_diff.min && naive_diff <= i_diff.max {
        println!("Result: SAFE. The interval contains the float result.");
    } else {
        println!("Result: UNSAFE. (This shouldn't happen with correct logic)");
    }

    // 3. Test Robust Quadratic
    println!("\n--- Testing Quadratic Solver ---");
    // Example: x^2 - 2x + 1 = 0 (Roots should be 1, 1)
    match solve_quadratic(1.0, -2.0, 1.0) {
        Some((t0, t1)) => println!("Roots: t0={}, t1={}", t0, t1),
        None => println!("No solution"),
    }
}