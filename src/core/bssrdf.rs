use crate::core::geometry::{Point3, Vector3, Normal3};
use crate::core::spectrum::SampledSpectrum;
use crate::core::reflection::fr_dielectric;
use std::f32::consts::PI;

// --- 1. The BSSRDF Trait Definition ---
pub trait BSSRDF: Send + Sync {
    fn eval_spatial(&self, r: f32) -> SampledSpectrum;
    fn eval_directional(&self, cos_theta: f32) -> f32;
}

// --- 2. Separable BSSRDF Implementation ---
pub struct SeparableBSSRDF {
    pub eta: f32, 
    weights: Vec<SampledSpectrum>,
    variances: Vec<f32>,
}

impl SeparableBSSRDF {
    pub fn new_skin(eta: f32) -> Self {
        // FIX: Create a "Skin-like" spectrum (Reddish)
        // Index 0 = 400nm (Blue), Index 3 = 700nm (Red)
        // Values: [Blue=0.1, Green=0.3, Yellow=0.6, Red=0.9]
        let skin_color = SampledSpectrum { values: [0.1, 0.3, 0.6, 0.9] };

        let weights = vec![
            skin_color, // Use the red spectrum
        ];
        
        let variances = vec![0.5]; 

        SeparableBSSRDF { eta, weights, variances }
    }

    fn gaussian(v: f32, r2: f32) -> f32 {
        (1.0 / (2.0 * PI * v)) * (-r2 / (2.0 * v)).exp()
    }
}

impl BSSRDF for SeparableBSSRDF {
    fn eval_directional(&self, cos_theta: f32) -> f32 {
        let f = fr_dielectric(cos_theta, 1.0, self.eta);
        (1.0 - f) / PI
    }

    fn eval_spatial(&self, r: f32) -> SampledSpectrum {
        let r2 = r * r;
        let mut result = SampledSpectrum::new(0.0);

        for (i, w) in self.weights.iter().enumerate() {
            let v = self.variances[i];
            let g = SeparableBSSRDF::gaussian(v, r2);
            result = result + (*w * g);
        }
        result
    }
}