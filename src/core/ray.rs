use crate::core::geometry::{Point3, Vector3};

#[derive(Debug, Clone, Copy)]
pub struct Ray {
    pub o: Point3,   // Origin
    pub d: Vector3,  // Direction
    pub time: f32,   // For Motion Blur
    pub t_max: f32,  // For intersection limits

    // --- Ray Differentials (Month 1, Week 4) ---
    // Used for Texture Filtering in Month 2
    pub has_differentials: bool,
    pub rx_origin: Point3,
    pub ry_origin: Point3,
    pub rx_direction: Vector3,
    pub ry_direction: Vector3,
}

impl Ray {
    pub fn new(o: Point3, d: Vector3, time: f32) -> Self {
        // Default: No differentials (pinhole center ray)
        Ray { 
            o, d, time, 
            t_max: std::f32::INFINITY,
            has_differentials: false,
            rx_origin: Point3 { x: 0.0, y: 0.0, z: 0.0 },
            ry_origin: Point3 { x: 0.0, y: 0.0, z: 0.0 },
            rx_direction: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
            ry_direction: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
        }
    }

    // Calculate position at distance t
    pub fn at(&self, t: f32) -> Point3 {
        self.o + self.d * t
    }

    // Scale differentials based on Samples Per Pixel (SPP)
    // If we take 16 samples, the "pixel footprint" is smaller.
    pub fn scale_differentials(&mut self, spp: f32) {
        if !self.has_differentials { return; }
        let s = (1.0 / spp.sqrt()).max(0.125); 
        self.rx_origin = self.o + (self.rx_origin - self.o) * s;
        self.ry_origin = self.o + (self.ry_origin - self.o) * s;
        self.rx_direction = self.d + (self.rx_direction - self.d) * s;
        self.ry_direction = self.d + (self.ry_direction - self.d) * s;
    }
}