mod core;
mod shapes;

use std::sync::Arc;
use crate::core::geometry::{Point3, Vector3, Point2, Point2i};
use crate::core::transform::Transform;
use crate::core::camera::PerspectiveCamera;
use crate::core::primitive::{GeometricPrimitive, Primitive};
use crate::shapes::triangle::{TriangleMesh, Triangle};
use crate::core::film::Film;
use crate::core::integrator::render;
use crate::core::bsdf::{BSDF, BxDF, ThinDielectricBxDF}; // Use the BSDF system now

fn main() {
    println!("--- Month 2 Week 7: Soap Bubble Test ---");

    // 1. Setup Scene
    let vertices = vec![
        Point3::new(-1.0, -1.0, 0.0),
        Point3::new( 1.0, -1.0, 0.0),
        Point3::new( 0.0,  1.0, 0.0),
    ];
    let indices = vec![0, 1, 2];
    let mesh = Arc::new(TriangleMesh::new(indices, vertices, None, None));
    let shape = Arc::new(Triangle::new(mesh, 0));

    // 2. Material: SOAP BUBBLE
    // IOR = 1.33 (Water/Soap), Thickness = 500 nm (produces green/magenta fringes)
    // NOTE: We haven't connected Material -> BSDF in Primitive yet.
    // For this test, we must manually update integrator.rs to use this BSDF.
    
    let prim = GeometricPrimitive::new(shape, None, 1.0);

    // 3. Setup Camera
    let pos = Point3::new(0.0, 0.0, -3.0);
    let look = Point3::new(0.0, 0.0, 0.0);
    let up = Vector3::new(0.0, 1.0, 0.0);
    let transform = Transform::look_at(pos, look, up);
    let res = Point2 { x: 400.0, y: 300.0 };
    let camera = PerspectiveCamera::new(transform, res, 90.0);
    let mut film = Film::new(Point2i { x: 400, y: 300 });

    render(&prim, &camera, &mut film);
    film.write_image("bubble.ppm").expect("Error writing image");
    println!("Done! Check bubble.ppm");
}