mod core;
mod shapes;

use crate::core::geometry::{Point3, Vector3, Point2};
use crate::core::transform::Transform;
use crate::core::camera::PerspectiveCamera;

fn main() {
    println!("--- Month 1 Week 4: Camera Test ---");

    // 1. Setup Camera at (0, 0, -5) looking at (0, 0, 0)
    let pos = Point3::new(0.0, 0.0, -5.0);
    let look = Point3::new(0.0, 0.0, 0.0);
    let up = Vector3::new(0.0, 1.0, 0.0);
    let cam_to_world = Transform::look_at(pos, look, up);

    // 2. Screen Resolution 800x600, FOV 90
    let res = Point2 { x: 800.0, y: 600.0 };
    let fov = 90.0;
    let camera = PerspectiveCamera::new(cam_to_world, res, fov);

    // 3. Generate Ray for Center Pixel (400, 300)
    let center_pixel = Point2 { x: 400.0, y: 300.0 };
    let center_ray = camera.generate_ray(center_pixel, res, fov);

    println!("Center Ray Origin: {:?}", center_ray.o);
    println!("Center Ray Dir:    {:?}", center_ray.d);

    // Expectation: Origin = (0,0,-5). Dir = (0,0,1) roughly.
    if (center_ray.d.z - 1.0).abs() < 0.01 {
        println!("[SUCCESS] Center ray points forward.");
    }

    // 4. Generate Ray for Top-Left Pixel (0, 0)
    let tl_pixel = Point2 { x: 0.0, y: 0.0 };
    let tl_ray = camera.generate_ray(tl_pixel, res, fov);
    println!("Top-Left Ray Dir:  {:?}", tl_ray.d);

    // Expectation: X should be negative (left), Y should be positive (up)
    if tl_ray.d.x < 0.0 && tl_ray.d.y > 0.0 {
        println!("[SUCCESS] Top-left ray points left-up.");
    }
    
    // 5. Check Differentials
    if tl_ray.has_differentials {
        println!("[SUCCESS] Ray differentials are active.");
        println!("  Rx Dir: {:?}", tl_ray.rx_direction);
    }
}