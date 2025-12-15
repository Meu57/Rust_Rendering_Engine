use std::sync::Arc;
use crate::core::interaction::SurfaceInteraction;
use crate::core::bsdf::{BSDF, BxDF, DiffuseBxDF, MicrofacetReflection, FresnelBlend, FresnelDielectric};
use crate::core::texture::Texture;
use crate::core::spectrum::SampledSpectrum;
use crate::core::geometry::Vector3;
use crate::core::microfacet::TrowbridgeReitzDistribution;
use crate::core::bsdf::Fresnel; // For trait types

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
    pub sigma: Arc<dyn Texture>, // Roughness (unused in basic matte)
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

// --- Principled PBR Material (Metalness workflow) ---
pub struct PrincipledMaterial {
    pub base_color: Arc<dyn Texture>,
    pub metallic: Arc<dyn Texture>, // 0.0 = Dielectric, 1.0 = Metal
    pub roughness: Arc<dyn Texture>,
}

impl PrincipledMaterial {
    pub fn new(
        base_color: Arc<dyn Texture>,
        metallic: Arc<dyn Texture>,
        roughness: Arc<dyn Texture>,
    ) -> Self {
        Self { base_color, metallic, roughness }
    }
}

fn lerp_spec(a: SampledSpectrum, b: SampledSpectrum, t: f32) -> SampledSpectrum {
    a * (1.0 - t) + b * t
}

impl Material for PrincipledMaterial {
    fn compute_scattering(&self, si: &SurfaceInteraction) -> Option<BSDF> {
        // 1. Sample Textures
        let base_color_val = self.base_color.evaluate(si);
        let metallic_val = self.metallic.evaluate(si).values[0]; // Assume grayscale
        let roughness_val = self.roughness.evaluate(si).values[0];

        // 2. Remap Roughness (Perceptual -> Alpha)
        let alpha = roughness_val * roughness_val;
        let distribution = TrowbridgeReitzDistribution::new(alpha, alpha);

        // 3. Determine F0 (Fresnel at normal incidence)
        let f0_dielectric = SampledSpectrum::splat(0.04);
        let f0 = lerp_spec(f0_dielectric, base_color_val, metallic_val);

        // 4. Determine Diffuse Color
        let diffuse_color = lerp_spec(base_color_val, SampledSpectrum::new(0.0), metallic_val);

        // 5. Construct BxDF
        let bxdf = if metallic_val > 0.5 {
            // --- METAL (Conductor) ---
            // Use Schlick-like Fresnel encoded by F0: approximate with FresnelDielectric by deriving eta.
            // This is an approximation; ideally we'd use FresnelConductor with spectral eta/k.
            // Solve approx: ((eta-1)/(eta+1))^2 = F0 => eta = (1 + sqrt(F0)) / (1 - sqrt(F0))
            let avg = (f0.values[0] + f0.values[1] + f0.values[2]) / 3.0;
            let sqrt_f0 = avg.max(0.0).sqrt();
            let eta = if (1.0 - sqrt_f0).abs() < 1e-6 { 1e6 } else { (1.0 + sqrt_f0) / (1.0 - sqrt_f0) };
            let fresnel = Box::new(FresnelDielectric { eta_i: 1.0, eta_t: eta as f32 });
            // Use full specular microfacet with tint from base_color (approximate)
            BxDF::Microfacet(MicrofacetReflection::new(f0, distribution, fresnel))
        } else {
            // --- PLASTIC / DIELECTRIC ---
            let fresnel = Box::new(FresnelDielectric { eta_i: 1.0, eta_t: 1.5 }); // IOR 1.5 standard
            let spec = MicrofacetReflection::new(SampledSpectrum::splat(1.0), distribution, fresnel);
            let diff = DiffuseBxDF::new(diffuse_color);
            BxDF::FresnelBlend(FresnelBlend::new(diff, spec))
        };

        Some(BSDF::new(Vector3::from(si.shading.n), bxdf))
    }
}
