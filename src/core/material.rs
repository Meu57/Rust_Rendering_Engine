use std::sync::Arc;
use crate::core::interaction::SurfaceInteraction;
use crate::core::bsdf::{BSDF, BxDF, DiffuseBxDF};
use crate::core::texture::Texture;
use crate::core::spectrum::SampledSpectrum;
use crate::core::geometry::Vector3; // Import Vector3

// The Material Trait: Determines how light interacts with the surface
pub trait Material: Send + Sync {
    // 1. Scattering: Creates the BSDF (BRDF/BTDF) for the hit point
    fn compute_scattering(&self, si: &SurfaceInteraction) -> Option<BSDF>;
    
    // 2. Emission: Does this material emit light? (Le)
    fn emitted(&self, _si: &SurfaceInteraction) -> SampledSpectrum {
        SampledSpectrum::new(0.0)
    }
}

// --- Matte Material (Lambertian) ---
pub struct MatteMaterial {
    pub kd: Arc<dyn Texture>, // Diffuse Reflectance (Texture)
    pub sigma: Arc<dyn Texture>, // Roughness
}

impl MatteMaterial {
    pub fn new(kd: Arc<dyn Texture>, sigma: Arc<dyn Texture>) -> Self {
        MatteMaterial { kd, sigma }
    }
}

impl Material for MatteMaterial {
    fn compute_scattering(&self, si: &SurfaceInteraction) -> Option<BSDF> {
        // Evaluate textures at the hit point
        let r = self.kd.evaluate(si);
        
        // Create the BSDF
        // FIX: Convert Normal3 to Vector3 for the BSDF Frame
        let bxdf = BxDF::Diffuse(DiffuseBxDF::new(r));
        Some(BSDF::new(Vector3::from(si.shading.n), bxdf))
    }
}

// --- Emissive Material (Light Source) ---
pub struct EmissiveMaterial {
    pub emit: Arc<dyn Texture>,
}

impl EmissiveMaterial {
    pub fn new(emit: Arc<dyn Texture>) -> Self {
        EmissiveMaterial { emit }
    }
}

impl Material for EmissiveMaterial {
    fn compute_scattering(&self, _si: &SurfaceInteraction) -> Option<BSDF> {
        // Lights don't scatter, they just emit.
        None 
    }

    fn emitted(&self, si: &SurfaceInteraction) -> SampledSpectrum {
        self.emit.evaluate(si)
    }
}