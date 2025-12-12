use crate::core::geometry::{Point2i, Vector3};
use crate::core::spectrum::SampledSpectrum;
use std::fs::File;
use std::io::Write;

pub struct Film {
    pub resolution: Point2i,
    pixels: Vec<Vector3>, // Storing simplified RGB for now
}

impl Film {
    pub fn new(resolution: Point2i) -> Self {
        let count = (resolution.x * resolution.y) as usize;
        Film {
            resolution,
            pixels: vec![Vector3 { x: 0.0, y: 0.0, z: 0.0 }; count],
        }
    }

    pub fn set_pixel(&mut self, p: Point2i, color: Vector3) {
        let idx = (p.y * self.resolution.x + p.x) as usize;
        self.pixels[idx] = color;
    }

    // Output to a simple PPM image format (readable by most viewers)
    pub fn write_image(&self, filename: &str) -> std::io::Result<()> {
        let mut file = File::create(filename)?;
        write!(file, "P3\n{} {}\n255\n", self.resolution.x, self.resolution.y)?;

        for p in &self.pixels {
            let r = (p.x.sqrt().clamp(0.0, 1.0) * 255.99) as u8; // Gamma correction (sqrt)
            let g = (p.y.sqrt().clamp(0.0, 1.0) * 255.99) as u8;
            let b = (p.z.sqrt().clamp(0.0, 1.0) * 255.99) as u8;
            writeln!(file, "{} {} {}", r, g, b)?;
        }
        Ok(())
    }
}