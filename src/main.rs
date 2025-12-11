mod core;
mod shapes;

use std::sync::Arc;
use crate::core::geometry::{Point3, Vector3, Normal3, Point2}; 
use crate::core::transform::{Transform, Matrix4x4};
use crate::core::ray::Ray;
use crate::shapes::triangle::{Triangle, TriangleMesh};
// We import 'Shape' so Rust knows Triangles have bounds/intersections
use crate::core::primitive::{Primitive, GeometricPrimitive, TransformedPrimitive, Shape}; 

fn main() {
    println!("--- Week 2: Instancing & Primitives Test ---");

    // 1. Create the shared Mesh (A single triangle at the origin)
    // Vertices: Bottom-Left(-1,-1,0), Bottom-Right(1,-1,0), Top-Center(0,1,0)
    let vertices = vec![
        Point3 { x: -1.0, y: -1.0, z: 0.0 },
        Point3 { x: 1.0,  y: -1.0, z: 0.0 },
        Point3 { x: 0.0,  y: 1.0,  z: 0.0 },
    ];
    let indices = vec![0, 1, 2];
    
    // Shared Mesh Data
    let mesh = Arc::new(TriangleMesh::new(indices, vertices, None, None));
    
    // The 'Shape' (Triangle #0 of that mesh)
    let tri_shape = Arc::new(Triangle::new(mesh.clone(), 0));

    // 2. Wrap it in a GeometricPrimitive (The "Prototype")
    // This represents the physical object "Tree"
    let base_prim = Arc::new(GeometricPrimitive::new(tri_shape, None));

    // 3. Create an INSTANCE moved 10 units up (+Y)
    // This represents "Tree #1" placed at (0, 10, 0)
    let mut matrix = Matrix4x4::identity();
    matrix.m[1][3] = 10.0; // Translate Y by 10
    let transform = Transform::new(matrix);

    let instance = TransformedPrimitive::new(base_prim.clone(), transform);

    // 4. Fire a Ray at the INSTANCE (target: 0, 10, 0)
    // Ray starts at (0, 10, -5) and looks forward (+Z)
    let ray = Ray::new(
        Point3 { x: 0.0, y: 10.0, z: -5.0 }, 
        Vector3 { x: 0.0, y: 0.0, z: 1.0 },
        0.0
    );

    println!("Firing ray at (0, 10, 0)...");
    
    match instance.intersect(&ray) {
        Some((t, interaction)) => {
            println!("[SUCCESS] Instanced Hit!");
            println!("  Distance: {:.4} (Expected: 5.0000)", t);
            println!("  Hit Point (World): {:?}", interaction.core.p);
            
            // Verification: The hit point should be (0, 10, 0)
            // If Instancing is broken, it might report (0, 0, 0) or miss entirely.
            if (interaction.core.p.y - 10.0).abs() < 0.001 {
                println!("  [Verified] Point correctly transformed to World Space.");
            } else {
                println!("  [ERROR] Point is in Object Space! Transform logic is missing.");
            }
        },
        None => println!("[FAIL] Missed the instance."),
    }
}