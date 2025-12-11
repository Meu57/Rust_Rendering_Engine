use crate::core::geometry::{Vector3, Point2};
use std::f32::consts::PI;

// --- Helper Functions for Trigonometry on Vectors ---
// Assumes vector is in "Shading Space" (Z is normal)
fn cos_theta(w: Vector3) -> f32 { w.z }
fn cos2_theta(w: Vector3) -> f32 { w.z * w.z }
fn sin2_theta(w: Vector3) -> f32 { (1.0 - cos2_theta(w)).max(0.0) }
fn tan2_theta(w: Vector3) -> f32 { sin2_theta(w) / cos2_theta(w) }

fn cos_phi(w: Vector3) -> f32 {
    let sin_theta = sin2_theta(w).sqrt();
    if sin_theta == 0.0 { 1.0 } else { (w.x / sin_theta).clamp(-1.0, 1.0) }
}
fn sin_phi(w: Vector3) -> f32 {
    let sin_theta = sin2_theta(w).sqrt();
    if sin_theta == 0.0 { 0.0 } else { (w.y / sin_theta).clamp(-1.0, 1.0) }
}
fn cos2_phi(w: Vector3) -> f32 { let c = cos_phi(w); c * c }
fn sin2_phi(w: Vector3) -> f32 { let s = sin_phi(w); s * s }

// --- The GGX Distribution ---
#[derive(Debug, Clone, Copy)]
pub struct TrowbridgeReitzDistribution {
    pub alpha_x: f32,
    pub alpha_y: f32,
}

impl TrowbridgeReitzDistribution {
    // Constructor: Maps roughness (0..1) to alpha (roughness^2)
    // We clamp alpha to a small epsilon to prevent division by zero
    pub fn new(rough_x: f32, rough_y: f32) -> Self {
        Self {
            alpha_x: (rough_x * rough_x).max(0.001),
            alpha_y: (rough_y * rough_y).max(0.001),
        }
    }

    // 1. Normal Distribution Function D(wh)
    // Probability of microfacet alignment
    pub fn d(&self, wh: Vector3) -> f32 {
        let tan2 = tan2_theta(wh);
        if tan2.is_infinite() { return 0.0; }

        let cos4 = cos2_theta(wh) * cos2_theta(wh);
        
        let e = (cos2_phi(wh) / (self.alpha_x * self.alpha_x) +
                 sin2_phi(wh) / (self.alpha_y * self.alpha_y)) * tan2;
        
        1.0 / (PI * self.alpha_x * self.alpha_y * cos4 * (1.0 + e) * (1.0 + e))
    }

    // Auxiliary Lambda function for Smith G
    fn lambda(&self, w: Vector3) -> f32 {
        let abs_tan_theta =  w.z.abs().max(1e-9).recip() * (1.0 - w.z * w.z).max(0.0).sqrt(); 
        if abs_tan_theta.is_infinite() { return 0.0; }

        let alpha = (cos2_phi(w) * self.alpha_x * self.alpha_x +
                     sin2_phi(w) * self.alpha_y * self.alpha_y).sqrt();

        ( (1.0 + (alpha * abs_tan_theta).powi(2)).sqrt() - 1.0 ) / 2.0
    }

    // 2. Geometry Term G(wo, wi)
    // Probability that microfacet is NOT blocked
    pub fn g(&self, wo: Vector3, wi: Vector3) -> f32 {
        1.0 / (1.0 + self.lambda(wo) + self.lambda(wi))
    }

    // Single-sided G1 (for importance sampling weights)
    pub fn g1(&self, w: Vector3) -> f32 {
        1.0 / (1.0 + self.lambda(w))
    }

    // 3. Sample_wh (VNDF Importance Sampling)
    // Samples a microfacet normal 'wh' visible from direction 'wo'
    // 3. Sample_wh (VNDF Importance Sampling)
    pub fn sample_wh(&self, wo: Vector3, u: Point2) -> Vector3 {
        // A. Transform to unit hemisphere configuration (stretch)
        let mut wh = Vector3 { 
            x: self.alpha_x * wo.x, 
            y: self.alpha_y * wo.y, 
            z: wo.z 
        }.normalize();
        
        if wh.z < 0.0 { wh = -wh; }

        // B. Orthonormal basis
        let t1 = if wh.z < 0.99999 {
            Vector3 { x: 0.0, y: 0.0, z: 1.0 }.cross(wh).normalize()
        } else {
            Vector3 { x: 1.0, y: 0.0, z: 0.0 }
        };
        let t2 = wh.cross(t1);

        // C. Sample Uniform Disk (Polar) -> USING SHARED HELPER
        let mut p = crate::core::math::sample_uniform_disk_polar(u);

        // D. Warp to Hemisphere
        let h = (1.0 - p.x * p.x).max(0.0).sqrt();
        p.y = (1.0 - h) / 2.0 * (1.0 + wh.z) + h * p.y;
        let pz = (1.0 - p.x*p.x - p.y*p.y).max(0.0).sqrt();

        // E. Reproject
        let nh = t1 * p.x + t2 * p.y + wh * pz;

        // F. Unstretch
        Vector3 {
            x: self.alpha_x * nh.x,
            y: self.alpha_y * nh.y,
            z: nh.z.max(1e-6)
        }.normalize()
    }
}