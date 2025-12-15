mod core;
mod shapes;

use std::sync::Arc;

use crate::core::geometry::{Point3, Vector3, Point2, Point2i};
use crate::core::transform::Transform;
use crate::core::camera::PerspectiveCamera;
use crate::core::primitive::{GeometricPrimitive, Primitive, PrimitiveList};
use crate::shapes::triangle::{TriangleMesh, Triangle};
use crate::core::film::Film;
use crate::core::integrator::render;
use crate::core::material::{MatteMaterial, EmissiveMaterial};
use crate::core::texture::{ConstantTexture, MarbleTexture};
use crate::core::spectrum::SampledSpectrum;
use crate::core::light::{Light, DiffuseAreaLight};

fn main() {
    println!("--- Month 3 Week 9: Direct Lighting + NEE ---");

    // --------------------------------------------------
    // 1. Materials
    // --------------------------------------------------
    let marble_tex = Arc::new(MarbleTexture::new(4.0));
    let sigma_zero = Arc::new(ConstantTexture::new(SampledSpectrum::new(0.0)));
    let marble_mat = Arc::new(MatteMaterial::new(marble_tex, sigma_zero));

    // High intensity because the light is small
    let light_emit = Arc::new(ConstantTexture::new(SampledSpectrum::new(50.0)));
    let light_mat = Arc::new(EmissiveMaterial::new(light_emit));

    // --------------------------------------------------
    // 2. Geometry
    // --------------------------------------------------

    // A. Marble Triangle (Object)
    let v_obj = vec![
        Point3::new(-1.0, -1.0, 0.0),
        Point3::new( 1.0, -1.0, 0.0),
        Point3::new( 0.0,  1.0, 0.0),
    ];
    let idx_obj = vec![0, 2, 1]; // Normal points -Z (Towards Camera)
    let mesh_obj = Arc::new(TriangleMesh::new(idx_obj, v_obj, None, None));
    let tri_obj = Arc::new(Triangle::new(mesh_obj, 0));
    let prim_obj = Arc::new(GeometricPrimitive::new(tri_obj, Some(marble_mat), 1.0));

    // B. Area Light
    // FIX 1: Move Light to Z = -1.0 (Between Camera and Object)
    // Previously it was at Z = +1.0 (Behind Object)
    let v_light = vec![
        Point3::new(-0.5, 1.5, -1.0),
        Point3::new( 0.5, 1.5, -1.0),
        Point3::new( 0.0, 1.5, -0.5), // Angled slightly back towards object
    ];
    // Winding 0,1,2 for this setup points mostly Down (-Y) and Back (+Z)
    let idx_light = vec![0, 1, 2]; 
    let mesh_light = Arc::new(TriangleMesh::new(idx_light, v_light, None, None));
    
    // Shared Shape: We need the shape for both the Light Logic AND the Scene Geometry
    let tri_light_shape = Arc::new(Triangle::new(mesh_light, 0));

    // The Logic (for Integrator sampling)
    let area_light = Box::new(DiffuseAreaLight::new(
        tri_light_shape.clone(),
        SampledSpectrum::new(50.0),
    ));
    let lights: Vec<Box<dyn Light>> = vec![area_light];

    // The Geometry (for Camera visibility)
    let prim_light = Arc::new(GeometricPrimitive::new(
        tri_light_shape, 
        Some(light_mat), 
        1.0
    ));

    // --------------------------------------------------
    // 3. Scene List
    // --------------------------------------------------
    // FIX 2: Add prim_light to the scene so it is visible to the camera
    let scene = PrimitiveList::new(vec![prim_obj, prim_light]);

    // --------------------------------------------------
    // 4. Camera
    // --------------------------------------------------
    let pos = Point3::new(0.0, 0.0, -3.0);
    let look = Point3::new(0.0, 0.0, 0.0);
    let up = Vector3::new(0.0, 1.0, 0.0);

    let transform = Transform::look_at(pos, look, up);
    let res = Point2 { x: 400.0, y: 300.0 };
    let camera = PerspectiveCamera::new(transform, res, 90.0);

    let mut film = Film::new(Point2i { x: 400, y: 300 });

    // --------------------------------------------------
    // Render
    // --------------------------------------------------
    render(&scene, &lights, &camera, &mut film);

    film.write_image("bubble.ppm").expect("Error writing image");
    println!("Done! Check bubble.ppm");
}