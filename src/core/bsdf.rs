use crate::core::geometry::{Vector3, Point2};
use crate::core::spectrum::SampledSpectrum;
use std::f32::consts::PI;

// --- Helper Functions ---
fn cos_theta(w: Vector3) -> f32 { w.z }

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

// --- 2. The BxDF Enum ---
pub enum BxDF {
    Diffuse(DiffuseBxDF),
    ThinDielectric(ThinDielectricBxDF),
}

impl BxDF {
    pub fn f(&self, wo: Vector3, wi: Vector3) -> SampledSpectrum {
        match self {
            BxDF::Diffuse(b) => b.f(wo, wi),
            BxDF::ThinDielectric(b) => b.f(wo, wi),
        }
    }
    pub fn sample_f(&self, wo: Vector3, u: Point2) -> Option<(SampledSpectrum, Vector3, f32)> {
        match self {
            BxDF::Diffuse(b) => b.sample_f(wo, u),
            BxDF::ThinDielectric(b) => b.sample_f(wo, u),
        }
    }
    pub fn pdf(&self, wo: Vector3, wi: Vector3) -> f32 {
        match self {
            BxDF::Diffuse(b) => b.pdf(wo, wi),
            BxDF::ThinDielectric(b) => b.pdf(wo, wi),
        }
    }
}

// --- 3. Diffuse BxDF ---
pub struct DiffuseBxDF {
    pub r: SampledSpectrum,
}
impl DiffuseBxDF {
    pub fn new(r: SampledSpectrum) -> Self { Self { r } }
    pub fn f(&self, _wo: Vector3, _wi: Vector3) -> SampledSpectrum {
        self.r * (1.0 / PI)
    }
    pub fn sample_f(&self, wo: Vector3, u: Point2) -> Option<(SampledSpectrum, Vector3, f32)> {
        let wi = cosine_sample_hemisphere(u);
        if wo.z * wi.z < 0.0 { return None; }
        Some((self.f(wo, wi), wi, self.pdf(wo, wi)))
    }
    pub fn pdf(&self, _wo: Vector3, wi: Vector3) -> f32 {
        if wi.z <= 0.0 { 0.0 } else { wi.z * (1.0 / PI) }
    }
}

// --- 4. Thin Dielectric BxDF (Window / Bubble) ---
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

    pub fn sample_f(&self, wo: Vector3, u: Point2) -> Option<(SampledSpectrum, Vector3, f32)> {
        // Fresnel for the single interface
        let f = crate::core::reflection::fr_dielectric(cos_theta(wo), 1.0, self.eta);

        // --- INTERFERENCE LOGIC ---
        // If thickness > 0, we calculate Wave Interference (Soap Bubble).
        // If thickness == 0, we assume it's a thick window (Incoherent sum).
        
        let (r_spectrum, t_spectrum) = if self.thickness > 0.0 {
            // -- COHERENT (Bubble) --
            // We need wavelengths to calculate phase shift.
            // Using standard visible approximations for the 4 buckets [400-700nm]
            let lambdas = [437.5, 512.5, 587.5, 662.5]; 
            let mut r_vals = [0.0; 4];
            let mut t_vals = [0.0; 4];

            // Optical Path Difference: 2 * eta * d * cos(theta_t)
            // Snell's law for cos(theta_t) inside film
            let sin_theta_i2 = 1.0 - wo.z * wo.z;
            let sin_theta_t2 = sin_theta_i2 / (self.eta * self.eta);
            let cos_theta_t = (1.0 - sin_theta_t2).max(0.0).sqrt();
            
            let path_diff = 2.0 * self.eta * self.thickness * cos_theta_t;

            for i in 0..4 {
                let lambda = lambdas[i];
                // Phase shift = 2*PI * path / lambda + PI (for external reflection flip)
                // Note: The first reflection (Air->Film) has PI shift. Second (Film->Air) does not.
                // The relative phase shift is 2*PI*path/lambda.
                let phase = (2.0 * PI * path_diff) / lambda;
                
                // Interference Intensity: I = 2F(1 - cos(phase)) approx for weak reflections
                // Or exact geometric series for coherent light:
                // R = (2r^2 (1 - cos phi)) / (1 + r^4 - 2r^2 cos phi) where r = sqrt(F)
                // Let's use the simple thin film approximation: 
                // R ~ 4 * F * sin^2(phase/2) (assuming F is small)
                
                let s = (phase / 2.0).sin();
                let r_coherent = 4.0 * f * s * s;
                
                r_vals[i] = r_coherent.clamp(0.0, 1.0);
                t_vals[i] = 1.0 - r_vals[i];
            }
            
            (SampledSpectrum { values: r_vals }, SampledSpectrum { values: t_vals })

        } else {
            // -- INCOHERENT (Window) --
            // R = 2F / (1 + F)
            let r_val = (2.0 * f) / (1.0 + f);
            (SampledSpectrum::splat(r_val), SampledSpectrum::splat(1.0 - r_val))
        };

        // Probability of reflection (average across spectrum to pick a path)
        // We just pick one channel or average them for the PDF
        let r_prob = (r_spectrum.values[0] + r_spectrum.values[1] + r_spectrum.values[2]) / 3.0;
        
        if u.x < r_prob {
            // Reflect
            let wi = Vector3 { x: -wo.x, y: -wo.y, z: wo.z };
            Some((r_spectrum * (1.0 / r_prob), wi, r_prob)) // Weight = R / PDF
        } else {
            // Transmit
            let wi = -wo;
            let t_prob = 1.0 - r_prob;
            Some((t_spectrum * (1.0 / t_prob), wi, t_prob))
        }
    }
}

// --- Helper ---
fn cosine_sample_hemisphere(u: Point2) -> Vector3 {
    let d = crate::core::math::sample_uniform_disk_polar(u);
    let z = (1.0 - d.x * d.x - d.y * d.y).max(0.0).sqrt();
    Vector3 { x: d.x, y: d.y, z }
}

// --- 5. BSDF Container ---
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
    pub fn sample_f(&self, wo_world: Vector3, u: Point2) -> Option<(SampledSpectrum, Vector3, f32)> {
        let wo = self.frame.to_local(wo_world);
        if let Some((f, wi_local, pdf)) = self.bxdf.sample_f(wo, u) {
            Some((f, self.frame.from_local(wi_local), pdf))
        } else { None }
    }
}