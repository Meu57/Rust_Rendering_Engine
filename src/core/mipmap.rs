use crate::core::geometry::{Point2, Vector3};
use crate::core::spectrum::SampledSpectrum;
use std::sync::Arc;

// A simplified MIP Map that currently only holds the base image (Level 0).
// In Week 6 Day 6, we will extend this to hold the full pyramid.
pub struct MIPMap {
    resolution: Point2,
    texels: Vec<SampledSpectrum>,
}

impl MIPMap {
    pub fn new(resolution: Point2, texels: Vec<SampledSpectrum>) -> Self {
        MIPMap { resolution, texels }
    }

    // --- Bilinear Filtering ---
    // Looks up the color at (u, v) by blending the 4 nearest pixels.
    pub fn lookup(&self, st: Point2) -> SampledSpectrum {
        // 1. Scale UV to Image Coordinates
        // Subtract 0.5 to align pixel centers (Rasterization standard)
        let s = st.x * self.resolution.x - 0.5;
        let t = st.y * self.resolution.y - 0.5;

        // 2. Find the integer bottom-left corner
        let s0 = s.floor() as i32;
        let t0 = t.floor() as i32;

        // 3. Find the fractional weights (how close are we to the next pixel?)
        let ds = s - s0 as f32;
        let dt = t - t0 as f32;

        // 4. Get the 4 neighbor pixels
        // (s0, t0), (s0+1, t0), (s0, t0+1), (s0+1, t0+1)
        let v00 = self.get_texel(s0, t0);
        let v10 = self.get_texel(s0 + 1, t0);
        let v01 = self.get_texel(s0, t0 + 1);
        let v11 = self.get_texel(s0 + 1, t0 + 1);

        // 5. Bilinear Interpolation Formula
        // Lerp(t, Lerp(s, v00, v10), Lerp(s, v01, v11))
        (v00 * (1.0 - ds) * (1.0 - dt)) +
        (v10 * ds * (1.0 - dt)) +
        (v01 * (1.0 - ds) * dt) +
        (v11 * ds * dt)
    }

    // Safe Texel Access (Clamp to Edge)
    // Handles wrapping/clamping behavior
    fn get_texel(&self, s: i32, t: i32) -> SampledSpectrum {
        let w = self.resolution.x as i32;
        let h = self.resolution.y as i32;

        // Clamp Address Mode (Extend edge pixels)
        let x = s.clamp(0, w - 1) as usize;
        let y = t.clamp(0, h - 1) as usize;

        self.texels[y * (w as usize) + x]
    }
}