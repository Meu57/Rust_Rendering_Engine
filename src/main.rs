mod core;
mod shapes;

use std::sync::Arc;
use crate::core::geometry::{Point3, Vector3, Normal3, Point2}; 
use crate::core::ray::Ray;
use crate::shapes::triangle::{Triangle, TriangleMesh};
use crate::core::primitive::{Primitive, GeometricPrimitive};

fn main() {
    println!("--- Week 2 Final: Recursive Alpha Test ---");

    // 1. The "Window" (Alpha 0.5) at Z=0
    let window_verts = vec![
        Point3 { x: -5.0, y: -5.0, z: 0.0 },
        Point3 { x: 5.0,  y: -5.0, z: 0.0 },
        Point3 { x: 0.0,  y: 5.0,  z: 0.0 },
    ];
    let window_mesh = Arc::new(TriangleMesh::new(vec![0,1,2], window_verts, None, None));
    let window_shape = Arc::new(Triangle::new(window_mesh, 0));
    let window_prim = GeometricPrimitive::new(window_shape, None, 0.5); 

    // 2. The "Wall" (Solid) at Z=10
    let wall_verts = vec![
        Point3 { x: -10.0, y: -10.0, z: 10.0 },
        Point3 { x: 10.0,  y: -10.0, z: 10.0 },
        Point3 { x: 0.0,   y: 10.0,  z: 10.0 },
    ];
    let wall_mesh = Arc::new(TriangleMesh::new(vec![0,1,2], wall_verts, None, None));
    let wall_shape = Arc::new(Triangle::new(wall_mesh, 0));
    let wall_prim = GeometricPrimitive::new(wall_shape, None, 1.0); 

    // 3. Fire Rays
    println!("Firing rays through the window...");
    
    let mut hit_window = 0;
    let mut hit_wall = 0;
    let total_rays = 100;

    for i in 0..total_rays {
        let offset = (i as f32) * 0.02; 
        let ray = Ray::new(
            Point3 { x: 0.0 + offset, y: 0.0 + offset, z: -10.0 }, 
            Vector3 { x: 0.0, y: 0.0, z: 1.0 }, 
            0.0
        );

        // Manually simulate a Scene loop (Ray hits window? If miss/pass-through, check wall?)
        // Note: In a real engine, the BVH does this loop. Here we do it manually.
        
        let mut hit_something = false;

        // Check Window First
        if let Some(_) = window_prim.intersect(&ray) {
            hit_window += 1;
            hit_something = true;
        } 
        
        // If window reported "None" (either geometric miss OR alpha hole), check Wall
        if !hit_something {
             if let Some(_) = wall_prim.intersect(&ray) {
                hit_wall += 1;
            }
        }
    }

    println!("Total Rays: {}", total_rays);
    println!("Stopped at Window: {} (Expected ~50)", hit_window);
    println!("Passed through to Wall: {} (Expected ~50)", hit_wall);
    
    if hit_wall > 0 {
        println!("[SUCCESS] X-Ray Vision Active! Rays passed through the holes.");
    }
}