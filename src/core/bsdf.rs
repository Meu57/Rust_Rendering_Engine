use crate::core::geometry::{Vector3, Point2};
use crate::core::spectrum::SampledSpectrum;
use std::f32::consts::PI;
use crate::core::microfacet::TrowbridgeReitzDistribution;
use crate::core::reflection::{fr_conductor, fr_dielectric};

// --- Small helper ---
fn lerp_spectrum(a: SampledSpectrum, b: SampledSpectrum, t: f32) -> SampledSpectrum {
    a * (1.0 - t) + b * t
}

// --- Helper Functions ---
fn cos_theta(w: Vector3) -> f32 { w.z }
fn abs_cos_theta(w: Vector3) -> f32 { w.z.abs() }

// --- 1. The Local Coordinate Frame ---
#[derive(Debug, Clone, Copy)]
pub struct Frame {
    x: Vector3, y: Vector3, z: Vector3,
}

impl Frame {
    pub fn from_z(z: Vector3) -> Self {
        let (x, y) = coordinate_system(z);
        Frame { x, y, z }
    }
    pub fn to_local(&self, v: Vector3) -> Vector3 {
        Vector3 { x: v.dot(self.x), y: v.dot(self.y), z: v.dot(self.z) }
    }
    pub fn from_local(&self, v: Vector3) -> Vector3 {
        self.x * v.x + self.y * v.y + self.z * v.z
    }
}

fn coordinate_system(v1: Vector3) -> (Vector3, Vector3) {
    let sign = if v1.z >= 0.0 { 1.0 } else { -1.0 };
    let a = -1.0 / (sign + v1.z);
    let b = v1.x * v1.y * a;
    let v2 = Vector3 { x: 1.0 + sign * v1.x * v1.x * a, y: sign * b, z: -sign * v1.x };
    let v3 = Vector3 { x: b, y: sign + v1.y * v1.y * a, z: -v1.y };
    (v2, v3)
}

// --- 2. The Fresnel Trait & Implementations ---
pub trait Fresnel: Send + Sync {
    fn evaluate(&self, cos_theta_i: f32) -> SampledSpectrum;
}

pub struct FresnelConductor {
    pub eta: SampledSpectrum,
    pub k: SampledSpectrum,
}

impl Fresnel for FresnelConductor {
    fn evaluate(&self, cos_theta_i: f32) -> SampledSpectrum {
        // Conductors (Metals) use complex IOR.
        fr_conductor(cos_theta_i.abs(), self.eta, self.k)
    }
}

pub struct FresnelDielectric {
    pub eta_i: f32,
    pub eta_t: f32,
}

impl Fresnel for FresnelDielectric {
    fn evaluate(&self, cos_theta_i: f32) -> SampledSpectrum {
        let f = fr_dielectric(cos_theta_i, self.eta_i, self.eta_t);
        SampledSpectrum::splat(f)
    }
}

// --- 3. The Cook-Torrance Microfacet BRDF ---
pub struct MicrofacetReflection {
    r: SampledSpectrum, // Reflectance (Albedo/Tint)
    distribution: TrowbridgeReitzDistribution,
    fresnel: Box<dyn Fresnel>,
}

impl MicrofacetReflection {
    pub fn new(
        r: SampledSpectrum,
        distribution: TrowbridgeReitzDistribution,
        fresnel: Box<dyn Fresnel>,
    ) -> Self {
        Self { r, distribution, fresnel }
    }

    pub fn f(&self, wo: Vector3, wi: Vector3) -> SampledSpectrum {
        let cos_theta_o = abs_cos_theta(wo);
        let cos_theta_i = abs_cos_theta(wi);
        
        // Edge Case: Grazing angles cause division by zero or NaN.
        if cos_theta_i == 0.0 || cos_theta_o == 0.0 {
            return SampledSpectrum::new(0.0);
        }

        // Half-vector
        let mut wh = wo + wi;
        // Handle degenerate case where wo and wi are exactly opposite
        if wh.x == 0.0 && wh.y == 0.0 && wh.z == 0.0 {
            return SampledSpectrum::new(0.0);
        }
        wh = wh.normalize();

        // 1. Distribution D(h)
        let d = self.distribution.d(wh);

        // 2. Fresnel F(o, h)
        // Note: Fresnel is evaluated using dot(wo, wh)
        let f = self.fresnel.evaluate(wo.dot(wh));

        // 3. Geometry G(o, i)
        let g = self.distribution.g(wo, wi);

        // Cook-Torrance Denominator: 4 * (n.i) * (n.o)
        let denom = 4.0 * cos_theta_i * cos_theta_o;

        // Result: (R * D * F * G) / Denom
        self.r * f * (d * g / denom)
    }

    // UPDATED: return (f, wi, pdf, is_delta)
    pub fn sample_f(&self, wo: Vector3, u: Point2) -> Option<(SampledSpectrum, Vector3, f32, bool)> {
        // 1. Sample Microfacet Normal (wh)
        if wo.z == 0.0 { return None; }

        let wh = self.distribution.sample_wh(wo, u);

        // 2. Reflect wo about wh to get wi
        let wi = Vector3::from(wh) * (2.0 * wo.dot(wh)) - wo;

        // Ensure we are still in the upper hemisphere
        if wo.z * wi.z < 0.0 { return None; }

        // 3. Compute PDF
        let pdf = self.pdf(wo, wi);
        if pdf <= 0.0 { return None; }

        // 4. Evaluate f()
        let f = self.f(wo, wi);

        // Microfacet is scattering (not delta)
        Some((f, wi, pdf, false))
    }
    
    pub fn pdf(&self, wo: Vector3, wi: Vector3) -> f32 {
        if wo.z * wi.z < 0.0 { return 0.0; } // Different hemispheres
        
        let mut wh = wo + wi;
        if wh.x == 0.0 && wh.y == 0.0 && wh.z == 0.0 { return 0.0; }
        wh = wh.normalize();

        // PDF of sampling wh: D(wh) * cos_theta_wh
        let pdf_wh = self.distribution.d(wh) * abs_cos_theta(wh);
        
        // Jacobian change of variables: d_wh -> d_wi
        // pdf_wi = pdf_wh / (4 * dot(wo, wh))
        pdf_wh / (4.0 * wo.dot(wh).abs())
    }
}

// --- 5. Diffuse BxDF ---
pub struct DiffuseBxDF {
    pub r: SampledSpectrum,
}

impl DiffuseBxDF {
    pub fn new(r: SampledSpectrum) -> Self { Self { r } }
    pub fn f(&self, _wo: Vector3, _wi: Vector3) -> SampledSpectrum {
        self.r * (1.0 / PI)
    }

    pub fn sample_f(&self, wo: Vector3, u: Point2) -> Option<(SampledSpectrum, Vector3, f32, bool)> {
        let wi = cosine_sample_hemisphere(u);
        if wo.z * wi.z < 0.0 { return None; }
        Some((self.f(wo, wi), wi, self.pdf(wo, wi), false))
    }
    pub fn pdf(&self, _wo: Vector3, wi: Vector3) -> f32 {
        if wi.z <= 0.0 { 0.0 } else { wi.z * (1.0 / PI) }
    }
}

// --- 6. Thin Dielectric BxDF (Window / Bubble) ---
pub struct ThinDielectricBxDF {
    pub eta: f32,       // IOR (e.g., 1.5)
    pub thickness: f32, // 0.0 = Window (Incoherent), >0.0 = Bubble (Coherent, nm)
}

impl ThinDielectricBxDF {
    pub fn new(eta: f32, thickness: f32) -> Self { 
        Self { eta, thickness } 
    }

    pub fn f(&self, _wo: Vector3, _wi: Vector3) -> SampledSpectrum {
        SampledSpectrum::new(0.0)
    }
    pub fn pdf(&self, _wo: Vector3, _wi: Vector3) -> f32 { 0.0 }

    // UPDATED: return signature includes is_delta flag
    pub fn sample_f(&self, wo: Vector3, u: Point2) -> Option<(SampledSpectrum, Vector3, f32, bool)> {
        // Fresnel for the single interface
        let f = crate::core::reflection::fr_dielectric(cos_theta(wo), 1.0, self.eta);

        // --- INTERFERENCE LOGIC ---
        let (r_spectrum, t_spectrum) = if self.thickness > 0.0 {
            // -- COHERENT (Bubble) --
            let lambdas = [437.5, 512.5, 587.5, 662.5]; 
            let mut r_vals = [0.0; 4];
            let mut t_vals = [0.0; 4];

            let sin_theta_i2 = 1.0 - wo.z * wo.z;
            let sin_theta_t2 = sin_theta_i2 / (self.eta * self.eta);
            let cos_theta_t = (1.0 - sin_theta_t2).max(0.0).sqrt();
            
            let path_diff = 2.0 * self.eta * self.thickness * cos_theta_t;

            for i in 0..4 {
                let lambda = lambdas[i];
                let phase = (2.0 * PI * path_diff) / lambda;
                let s = (phase / 2.0).sin();
                let r_coherent = 4.0 * f * s * s;
                r_vals[i] = r_coherent.clamp(0.0, 1.0);
                t_vals[i] = 1.0 - r_vals[i];
            }
            
            (SampledSpectrum { values: r_vals }, SampledSpectrum { values: t_vals })
        } else {
            // -- INCOHERENT (Window) --
            let r_val = (2.0 * f) / (1.0 + f);
            (SampledSpectrum::splat(r_val), SampledSpectrum::splat(1.0 - r_val))
        };

        // Probability of reflection (average across spectrum to pick a path)
        let r_prob = (r_spectrum.values[0] + r_spectrum.values[1] + r_spectrum.values[2]) / 3.0;
        
        if u.x < r_prob {
            // Reflect (delta)
            let wi = Vector3 { x: -wo.x, y: -wo.y, z: wo.z };
            // Return weighted spectrum (importance sampling correction) and mark as delta
            Some((r_spectrum * (1.0 / r_prob), wi, r_prob, true))
        } else {
            // Transmit (delta)
            let wi = -wo;
            let t_prob = 1.0 - r_prob;
            Some((t_spectrum * (1.0 / t_prob), wi, t_prob, true))
        }
    }
}

// --- Helper ---
fn cosine_sample_hemisphere(u: Point2) -> Vector3 {
    let d = crate::core::math::sample_uniform_disk_polar(u);
    let z = (1.0 - d.x * d.x - d.y * d.y).max(0.0).sqrt();
    Vector3 { x: d.x, y: d.y, z }
}

// --- NEW: FresnelBlend (Layering) ---
pub struct FresnelBlend {
    diffuse: DiffuseBxDF,
    specular: MicrofacetReflection,
}

impl FresnelBlend {
    pub fn new(diffuse: DiffuseBxDF, specular: MicrofacetReflection) -> Self {
        Self { diffuse, specular }
    }

    pub fn f(&self, wo: Vector3, wi: Vector3) -> SampledSpectrum {
        // Specular term
        let specular_term = self.specular.f(wo, wi);

        // Fresnel weight using Schlick approximation with F0 = 0.04 (typical dielectric)
        let wh = (wo + wi).normalize();
        let cos_theta_d = wo.dot(wh).abs();
        let f0 = 0.04;
        let f_s = f0 + (1.0 - f0) * (1.0 - cos_theta_d).powi(5);

        // Diffuse scaled by (1 - F_s)
        let diffuse_term = self.diffuse.f(wo, wi) * (1.0 - f_s);

        specular_term + diffuse_term
    }

    // UPDATED: sample_f returns is_delta boolean (we return false for the blend)
    pub fn sample_f(&self, wo: Vector3, u: Point2) -> Option<(SampledSpectrum, Vector3, f32, bool)> {
        if u.x < 0.5 {
            let u_remap = Point2 { x: 2.0 * u.x, y: u.y };
            if let Some((_f_spec, wi, _pdf_spec, is_delta)) = self.specular.sample_f(wo, u_remap) {
                // Recalculate blended PDF/F
                let pdf_blend = self.pdf(wo, wi);
                let f_blend = self.f(wo, wi);
                // For the blend we treat the result as non-delta (composite)
                Some((f_blend, wi, pdf_blend, false))
            } else { None }
        } else {
            let u_remap = Point2 { x: 2.0 * (u.x - 0.5), y: u.y };
            if let Some((_f_diff, wi, _pdf_diff, _is_delta)) = self.diffuse.sample_f(wo, u_remap) {
                let pdf_blend = self.pdf(wo, wi);
                let f_blend = self.f(wo, wi);
                Some((f_blend, wi, pdf_blend, false))
            } else { None }
        }
    }

    pub fn pdf(&self, wo: Vector3, wi: Vector3) -> f32 {
        0.5 * self.specular.pdf(wo, wi) + 0.5 * self.diffuse.pdf(wo, wi)
    }
}

// --- 7. BSDF Container ---
pub struct BSDF {
    frame: Frame,
    bxdf: BxDF,
}

impl BSDF {
    pub fn new(normal: Vector3, bxdf: BxDF) -> Self {
        BSDF { frame: Frame::from_z(normal), bxdf }
    }
    pub fn f(&self, wo: Vector3, wi: Vector3) -> SampledSpectrum {
        self.bxdf.f(self.frame.to_local(wo), self.frame.to_local(wi))
    }

    // UPDATED: sample_f passes through the is_delta flag and converts to world
    pub fn sample_f(&self, wo_world: Vector3, u: Point2) -> Option<(SampledSpectrum, Vector3, f32, bool)> {
        let wo = self.frame.to_local(wo_world);
        if let Some((f, wi_local, pdf, is_delta)) = self.bxdf.sample_f(wo, u) {
            Some((f, self.frame.from_local(wi_local), pdf, is_delta))
        } else { None }
    }

    pub fn pdf(&self, wo_world: Vector3, wi_world: Vector3) -> f32 {
        let wo = self.frame.to_local(wo_world);
        let wi = self.frame.to_local(wi_world);
        self.bxdf.pdf(wo, wi)
    }
}

// --- 8. The BxDF Enum ---
pub enum BxDF {
    Diffuse(DiffuseBxDF),
    ThinDielectric(ThinDielectricBxDF),
    Microfacet(MicrofacetReflection),
    FresnelBlend(FresnelBlend), // <--- NEW
}

impl BxDF {
    pub fn f(&self, wo: Vector3, wi: Vector3) -> SampledSpectrum {
        match self {
            BxDF::Diffuse(b) => b.f(wo, wi),
            BxDF::ThinDielectric(b) => b.f(wo, wi),
            BxDF::Microfacet(b) => b.f(wo, wi),
            BxDF::FresnelBlend(b) => b.f(wo, wi),
        }
    }

    // UPDATED: sample_f returns the (f, wi, pdf, is_delta) tuple
    pub fn sample_f(&self, wo: Vector3, u: Point2) -> Option<(SampledSpectrum, Vector3, f32, bool)> {
        match self {
            BxDF::Diffuse(b) => b.sample_f(wo, u),
            BxDF::ThinDielectric(b) => b.sample_f(wo, u),
            BxDF::Microfacet(b) => b.sample_f(wo, u),
            BxDF::FresnelBlend(b) => b.sample_f(wo, u),
        }
    }

    pub fn pdf(&self, wo: Vector3, wi: Vector3) -> f32 {
        match self {
            BxDF::Diffuse(b) => b.pdf(wo, wi),
            BxDF::ThinDielectric(b) => b.pdf(wo, wi),
            BxDF::Microfacet(b) => b.pdf(wo, wi),
            BxDF::FresnelBlend(b) => b.pdf(wo, wi),
        }
    }
}
