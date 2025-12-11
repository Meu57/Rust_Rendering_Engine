use crate::core::geometry::{Point3, Vector3, Point2};
use crate::core::transform::Transform;
use crate::core::ray::Ray;

pub struct PerspectiveCamera {
    camera_to_world: Transform,
    raster_to_camera: Transform,
    dx_camera: Vector3,
    dy_camera: Vector3,
}

impl PerspectiveCamera {
    pub fn new(
        camera_to_world: Transform,
        resolution: Point2, // x=width, y=height
        fov: f32, // Field of view in degrees
    ) -> Self {
        let aspect = resolution.x / resolution.y;
        let scale = (fov.to_radians() / 2.0).tan();
        
        let screen_min_x = -aspect * scale;
        let screen_max_x =  aspect * scale;
        let screen_min_y = -scale;
        let screen_max_y =  scale;

        let dx_camera = Vector3::new(
            (screen_max_x - screen_min_x) / resolution.x,
            0.0, 0.0
        );
        let dy_camera = Vector3::new(
            0.0,
            (screen_min_y - screen_max_y) / resolution.y,
            0.0
        );

        PerspectiveCamera {
            camera_to_world,
            raster_to_camera: Transform::new(crate::core::transform::Matrix4x4::identity()),
            dx_camera,
            dy_camera,
        }
    }

    pub fn generate_ray(&self, pixel: Point2, resolution: Point2, fov: f32) -> Ray {
        let aspect = resolution.x / resolution.y;
        let scale = (fov.to_radians() / 2.0).tan();
        
        let u = pixel.x / resolution.x;
        let v = pixel.y / resolution.y;
        
        let p_camera_x = (u * 2.0 - 1.0) * aspect * scale;
        let p_camera_y = (1.0 - v * 2.0) * scale;
        
        let p_camera = Point3::new(p_camera_x, p_camera_y, 1.0);

        let mut ray = Ray::new(
            Point3::new(0.0, 0.0, 0.0),
            Vector3::from(p_camera).normalize(),
            0.0
        );

        ray.has_differentials = true;
        ray.rx_origin = ray.o;
        ray.ry_origin = ray.o;
        ray.rx_direction = (Vector3::from(p_camera) + self.dx_camera).normalize();
        ray.ry_direction = (Vector3::from(p_camera) + self.dy_camera).normalize();

        // Use the method defined in transform.rs
        self.camera_to_world.transform_ray(&ray)
    }
}