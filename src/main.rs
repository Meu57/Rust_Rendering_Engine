mod core;
mod shapes;

use crate::core::geometry::{Vector3, Point3, Point2};
use crate::core::bsdf::{BSDF, BxDF, DiffuseBxDF};
use crate::core::spectrum::SampledSpectrum;

fn main() {
    println!("--- Month 2 Week 6: BSDF System Test ---");

    // 1. Setup a Surface Normal (Pointing Up)
    let normal = Vector3::new(0.0, 1.0, 0.0);
    
    // 2. Create a Red Diffuse Material
    let red = SampledSpectrum::splat(0.8); // 80% reflective
    let bxdf = BxDF::Diffuse(DiffuseBxDF::new(red));
    let bsdf = BSDF::new(normal, bxdf);

    // 3. Test Evaluation (f)
    // Light coming from 45 degrees, Camera at 45 degrees
    let wo = Vector3::new(0.707, 0.707, 0.0);
    let wi = Vector3::new(-0.707, 0.707, 0.0);
    
    let f = bsdf.f(wo, wi);
    println!("BRDF Value (f): {:?}", f.values[0]);
    // Lambertian f = R / PI = 0.8 / 3.14159 = ~0.254
    
    if (f.values[0] - 0.254).abs() < 0.01 {
        println!("[SUCCESS] Lambertian reflection matches theory.");
    } else {
        println!("[FAIL] BRDF calculation is wrong.");
    }

    // 4. Test Sampling (sample_f)
    // We ask the material: "Generate a random light ray"
    let u = Point2 { x: 0.5, y: 0.5 }; // Pseudo-random sample
    if let Some((f_sample, wi_sample, pdf)) = bsdf.sample_f(wo, u) {
        println!("Sampled Direction: {:?}", wi_sample);
        println!("Sampled PDF: {:.4}", pdf);
        
        // Check coordinate transformation
        // Since normal is Y-up, the sampled Z in Local space should map to Y in World space.
        if wi_sample.y > 0.0 {
            println!("[SUCCESS] Sampled ray is in the upper hemisphere.");
        }
    }
}