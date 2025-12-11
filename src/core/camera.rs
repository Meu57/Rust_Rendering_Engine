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
        // 1. Compute Screen Window
        // Screen is at z=1. Height ranges from -tan(fov/2) to +tan(fov/2)
        let aspect = resolution.x / resolution.y;
        let scale = (fov.to_radians() / 2.0).tan();
        
        let screen_min_x = -aspect * scale;
        let screen_max_x =  aspect * scale;
        let screen_min_y = -scale;
        let screen_max_y =  scale;

        // 2. Build RasterToCamera Transform
        // Raster (0..w, 0..h) -> Screen (-aspect..aspect, -1..1) -> Camera (z=1)
        // Note: We skip the full Matrix construction for brevity and do the math in generate_ray
        // But for differentials, we need the "shift per pixel"
        
        let dx_camera = Vector3::new(
            (screen_max_x - screen_min_x) / resolution.x,
            0.0,
            0.0
        );
        
        let dy_camera = Vector3::new(
            0.0,
            (screen_min_y - screen_max_y) / resolution.y, // Flip Y (Raster Y is down, Camera Y is up)
            0.0
        );

        // We store the mapping logic:
        // Raster X (0) -> Screen Min X
        // Raster Y (0) -> Screen Max Y (Top-left origin)
        // This is a specialized Transform we'll apply manually
        
        PerspectiveCamera {
            camera_to_world,
            raster_to_camera: Transform::new(crate::core::transform::Matrix4x4::identity()), // Placeholder
            dx_camera,
            dy_camera,
        }
    }

    pub fn generate_ray(&self, pixel: Point2, resolution: Point2, fov: f32) -> Ray {
        // Re-calculate screen bounds (cleaner for this snippet)
        let aspect = resolution.x / resolution.y;
        let scale = (fov.to_radians() / 2.0).tan();
        
        // Map Raster (0..W, 0..H) to Camera Screen (-A..A, 1..-1)
        // Normalized 0..1
        let u = pixel.x / resolution.x;
        let v = pixel.y / resolution.y;
        
        // Remap to [-1, 1] then scale
        let p_camera_x = (u * 2.0 - 1.0) * aspect * scale;
        let p_camera_y = (1.0 - v * 2.0) * scale; // Flip Y
        
        let p_camera = Point3::new(p_camera_x, p_camera_y, 1.0);

        // Transform to World
        let mut ray = Ray::new(
            Point3::new(0.0, 0.0, 0.0), // Camera is at origin in Camera Space
            Vector3::from(p_camera).normalize(),
            0.0
        );

        // Differentials
        ray.has_differentials = true;
        ray.rx_origin = ray.o;
        ray.ry_origin = ray.o;
        ray.rx_direction = (Vector3::from(p_camera) + self.dx_camera).normalize();
        ray.ry_direction = (Vector3::from(p_camera) + self.dy_camera).normalize();

        // Apply CameraToWorld
        self.camera_to_world.transform_ray(&mut ray)
    }
}

// Add transform_ray to Transform struct
impl Transform {
    pub fn transform_ray(&self, ray: &mut Ray) -> Ray {
        let mut o = self.transform_point(ray.o);
        let mut d = self.transform_vector(ray.d);
        
        // Differentials
        let mut rx_o = self.transform_point(ray.rx_origin);
        let mut ry_o = self.transform_point(ray.ry_origin);
        let mut rx_d = self.transform_vector(ray.rx_direction);
        let mut ry_d = self.transform_vector(ray.ry_direction);

        Ray {
            o, d,
            time: ray.time,
            t_max: ray.t_max,
            has_differentials: ray.has_differentials,
            rx_origin: rx_o,
            ry_origin: ry_o,
            rx_direction: rx_d,
            ry_direction: ry_d,
        }
    }
}