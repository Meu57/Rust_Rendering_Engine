use crate::core::math::RNG;
use crate::core::geometry::{Point2, Point2i};

pub struct StratifiedSampler {
    x_samples: usize,
    y_samples: usize,
    rng: RNG,
    current_pixel: Point2i,
    current_sample: usize,
}

impl StratifiedSampler {
    pub fn new(x_samples: usize, y_samples: usize) -> Self {
        Self {
            x_samples,
            y_samples,
            rng: RNG::new(0, 0), // Will be re-seeded per pixel
            current_pixel: Point2i { x: 0, y: 0 },
            current_sample: 0,
        }
    }

    pub fn samples_per_pixel(&self) -> usize {
        self.x_samples * self.y_samples
    }

    pub fn start_pixel(&mut self, p: Point2i) {
        self.current_pixel = p;
        self.current_sample = 0;
        // Deterministic Seeding: Hash pixel coordinates to get a seed
        let seed = (p.x as u64) << 32 | (p.y as u64);
        self.rng = RNG::new(seed, 1);
    }

    pub fn get_2d(&mut self) -> Point2 {
        if self.current_sample >= self.samples_per_pixel() {
            self.current_sample = 0;
        }

        // Compute grid cell (stratum) indices
        let stratum_x = self.current_sample % self.x_samples;
        let stratum_y = self.current_sample / self.x_samples;
        self.current_sample += 1;

        // Jitter within the stratum
        let dx = self.rng.next_f32();
        let dy = self.rng.next_f32();

        Point2 {
            x: (stratum_x as f32 + dx) / self.x_samples as f32,
            y: (stratum_y as f32 + dy) / self.y_samples as f32,
        }
    }
}