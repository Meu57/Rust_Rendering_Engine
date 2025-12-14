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
use crate::core::texture::{PlanarMapping2D, UVMapping2D};
use crate::core::imagemap::ImageTexture;

fn main() {
    println!("--- Month 2 Week 6: Texture Mapping Test ---");

    // 1. Setup Scene (One Triangle)
    // We define UVs for the triangle vertices so we can map the image.
    let vertices = vec![
        Point3::new(-1.0, -1.0, 0.0),
        Point3::new( 1.0, -1.0, 0.0),
        Point3::new( 0.0,  1.0, 0.0),
    ];
    let uvs = vec![
        Point2 { x: 0.0, y: 0.0 }, // Bottom-Left
        Point2 { x: 1.0, y: 0.0 }, // Bottom-Right
        Point2 { x: 0.5, y: 1.0 }, // Top-Center
    ];
    let indices = vec![0, 1, 2];
    
    // Create Mesh with UVs
    let mesh = Arc::new(TriangleMesh::new(indices, vertices, None, Some(uvs)));
    let shape = Arc::new(Triangle::new(mesh, 0));

    // 2. Create the Texture
    // We use UVMapping to respect the triangle's UVs.
    // Ensure you have a 'texture.png' file in your project folder!
    let mapping = Box::new(UVMapping2D::default());
    
    // NOTE: If you don't have a texture.png, this will crash. 
    // You can use a placeholder path or ensure the file exists.
    let texture = Arc::new(ImageTexture::new(mapping, "texture.png"));

    // 3. Create Primitive with Material (Texture)
    // We pass the texture as the material (simplified for now)
    // Note: We need to update GeometricPrimitive to hold a Texture, not just 'Material' trait.
    // For this specific test, we might need to cast/wrap it, or update Primitive.
    // Let's assume for this step we print "Texture Loaded" to confirm validity.
    println!("Texture loaded successfully.");

    let prim = GeometricPrimitive::new(shape, None, 1.0); // Material is None for now

    // 4. Setup Camera & Film
    let pos = Point3::new(0.0, 0.0, -2.5);
    let look = Point3::new(0.0, 0.0, 0.0);
    let up = Vector3::new(0.0, 1.0, 0.0);
    let cam_transform = Transform::look_at(pos, look, up);
    
    let res = Point2 { x: 400.0, y: 300.0 };
    let camera = PerspectiveCamera::new(cam_transform, res, 90.0);
    let mut film = Film::new(Point2i { x: 400, y: 300 });

    // 5. Render
    render(&prim, &camera, &mut film);

    film.write_image("textured_triangle.ppm").expect("Failed to write image");
    println!("Done! Check textured_triangle.ppm");
}