mod core;
mod shapes;

use std::sync::Arc;
use crate::core::geometry::{Point3, Vector3, Point2, Point2i};
use crate::core::transform::Transform;
use crate::core::camera::PerspectiveCamera;
use crate::core::primitive::{GeometricPrimitive, Primitive, PrimitiveList}; // Import List
use crate::shapes::triangle::{TriangleMesh, Triangle};
use crate::core::film::Film;
use crate::core::integrator::render;
use crate::core::material::{MatteMaterial, EmissiveMaterial};
use crate::core::texture::{ConstantTexture, MarbleTexture}; 
use crate::core::spectrum::SampledSpectrum;

fn main() {
    println!("--- Month 3 Week 9: Path Tracing Test ---");

    // 1. Materials
    let light_emit = Arc::new(ConstantTexture::new(SampledSpectrum::new(20.0))); 
    let light_mat = Arc::new(EmissiveMaterial::new(light_emit));

    let marble_tex = Arc::new(MarbleTexture::new(4.0));
    let sigma_zero = Arc::new(ConstantTexture::new(SampledSpectrum::new(0.0)));
    let marble_mat = Arc::new(MatteMaterial::new(marble_tex, sigma_zero.clone()));

    // 2. Geometry
    
    // A. Marble Triangle (The Object)
    // Vertices: Same as before
    let v_obj = vec![
        Point3::new(-1.0, -1.0, 0.0),
        Point3::new( 1.0, -1.0, 0.0),
        Point3::new( 0.0,  1.0, 0.0),
    ];
    // FIX: Winding order swapped (0, 2, 1) so Normal points -Z (towards camera)
    let idx_obj = vec![0, 2, 1]; 
    let mesh_obj = Arc::new(TriangleMesh::new(idx_obj, v_obj, None, None));
    let tri_obj = Arc::new(Triangle::new(mesh_obj, 0));
    let prim_obj = Arc::new(GeometricPrimitive::new(tri_obj, Some(marble_mat), 1.0));
    
    // B. Light Source (Small triangle above)
    let v_light = vec![
        Point3::new(-0.5, 1.5, 1.0),
        Point3::new( 0.5, 1.5, 1.0),
        Point3::new( 0.0, 1.5, 2.0),
    ];
    let idx_light = vec![0, 1, 2]; // Points Down (-Y)
    let mesh_light = Arc::new(TriangleMesh::new(idx_light, v_light, None, None));
    let tri_light = Arc::new(Triangle::new(mesh_light, 0));
    let prim_light = Arc::new(GeometricPrimitive::new(tri_light, Some(light_mat), 1.0));

    // 3. Scene List
    let mut prims: Vec<Arc<dyn Primitive>> = Vec::new();
    prims.push(prim_obj);
    prims.push(prim_light);
    let scene = PrimitiveList::new(prims);

    // 4. Camera
    let pos = Point3::new(0.0, 0.0, -3.0);
    let look = Point3::new(0.0, 0.0, 0.0);
    let up = Vector3::new(0.0, 1.0, 0.0);
    let transform = Transform::look_at(pos, look, up);
    let res = Point2 { x: 400.0, y: 300.0 };
    let camera = PerspectiveCamera::new(transform, res, 90.0);
    let mut film = Film::new(Point2i { x: 400, y: 300 });

    // Render the Scene List
    render(&scene, &camera, &mut film);
    
    film.write_image("bubble.ppm").expect("Error writing image");
    println!("Done! Check bubble.ppm");
}