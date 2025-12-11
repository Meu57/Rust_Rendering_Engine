mod core; // This tells Rust: "Go check src/core/mod.rs"

// Now we import from the core module we just declared
use crate::core::geometry::{Point3, Vector3};
use crate::core::transform::{Transform, Matrix4x4};
use crate::core::math::{Interval, solve_quadratic};
mod shapes;

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

    println!("Interval Subtraction:   [{:.20}, \n {:.20}]", i_diff.min, i_diff.max);
    
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




    println!("--- Week 2: Watertight Triangle Intersection Test ---");

    // 1. Setup the Mesh Data
    // A simple triangle in the XY plane (z=0)
    // Vertices: Bottom-Left(-1,-1,0), Bottom-Right(1,-1,0), Top-Center(0,1,0)
    let vertices = vec![
        Point3 { x: -1.0, y: -1.0, z: 0.0 },
        Point3 { x: 1.0,  y: -1.0, z: 0.0 },
        Point3 { x: 0.0,  y: 1.0,  z: 0.0 },
    ];
    let indices = vec![0, 1, 2];

    // Create the Mesh (wrapped in Arc for shared ownership)
    let mesh = Arc::new(TriangleMesh::new( //<<----- Error Here
        indices, 
        vertices, 
        None, // No normals yet
        None  // No UVs yet
    ));

    // Create the Triangle Object (pointing to triangle #0 in the mesh)
    let tri = Triangle::new(mesh.clone(), 0); //<<----------Error Here

    // 2. Test 1: A Hit
    // Ray starts at (0,0,-5) and points forward (+Z). Should hit at t=5.0.
    let ray_hit = Ray::new(//<<--------Error Here
        Point3 { x: 0.0, y: 0.0, z: -5.0 },
        Vector3 { x: 0.0, y: 0.0, z: 1.0 },
        0.0 // time
    );

    match tri.intersect(&ray_hit, 1000.0) {
        Some((t, interaction)) => {
            println!("[SUCCESS] Ray Hit!");
            println!("  Distance t: {:.4} (Expected: 5.0000)", t);
            println!("  Hit Point:  {:?}", interaction.core.p);
        },
        None => println!("[FAIL] Ray Missed (Should have hit)"),
    }

    // 3. Test 2: A Miss
    // Ray starts at (0,0,-5) but points to the right. Should miss.
    let ray_miss = Ray::new(//<<--------- Error Here
        Point3 { x: 0.0, y: 0.0, z: -5.0 },
        Vector3 { x: 1.0, y: 0.0, z: 0.0 }, // Points right
        0.0
    );

    println!("\nTest 2: Firing Ray to the side...");
    match tri.intersect(&ray_miss, 1000.0) {
        Some(_) => println!("[FAIL] Ray Hit (Should have missed)"),
        None => println!("[SUCCESS] Ray Missed as expected."),
    }
}