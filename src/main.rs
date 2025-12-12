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

fn main() {
    println!("--- Month 1 Final: Render Loop Test ---");

    // 1. Setup Scene (One Triangle)
    let vertices = vec![
        Point3::new(-1.0, -1.0, 0.0),
        Point3::new( 1.0, -1.0, 0.0),
        Point3::new( 0.0,  1.0, 0.0),
    ];
    let indices = vec![0, 1, 2];
    let mesh = Arc::new(TriangleMesh::new(indices, vertices, None, None));
    let shape = Arc::new(Triangle::new(mesh, 0));
    let prim = GeometricPrimitive::new(shape, None, 1.0);

    // 2. Setup Camera
    let pos = Point3::new(0.0, 0.0, -2.0); // Moved back to see the triangle
    let look = Point3::new(0.0, 0.0, 0.0);
    let up = Vector3::new(0.0, 1.0, 0.0);
    let cam_transform = Transform::look_at(pos, look, up);
    
    let res_x = 200;
    let res_y = 150;
    let camera = PerspectiveCamera::new(
        cam_transform, 
        Point2 { x: res_x as f32, y: res_y as f32 }, 
        90.0
    );

    // 3. Setup Film
    let mut film = Film::new(Point2i { x: res_x, y: res_y });

    // 4. Render
    render(&prim, &camera, &mut film);

    // 5. Output
    film.write_image("debug_triangle.ppm").expect("Failed to write image");
    println!("Image saved to 'debug_triangle.ppm'. Open it to verify!");
}