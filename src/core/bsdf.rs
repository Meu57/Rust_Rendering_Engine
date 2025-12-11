use crate::core::geometry::{Vector3, Point2};
use crate::core::spectrum::SampledSpectrum;
use std::f32::consts::PI;

// --- 1. The Local Coordinate Frame ---
// Transforms vectors from World Space to "Shading Space" (where Z is normal)
#[derive(Debug, Clone, Copy)]
pub struct Frame {
    x: Vector3, // Tangent
    y: Vector3, // Bitangent
    z: Vector3, // Normal
}

impl Frame {
    // Construct orthonormal basis from a normal
    pub fn from_z(z: Vector3) -> Self {
        let (x, y) = coordinate_system(z);
        Frame { x, y, z }
    }

    // World -> Local
    pub fn to_local(&self, v: Vector3) -> Vector3 {
        Vector3 {
            x: v.dot(self.x),
            y: v.dot(self.y),
            z: v.dot(self.z),
        }
    }

    // Local -> World
    pub fn from_local(&self, v: Vector3) -> Vector3 {
        self.x * v.x + self.y * v.y + self.z * v.z
    }
}

// Helper to build a coordinate system (Duff et al.)
fn coordinate_system(v1: Vector3) -> (Vector3, Vector3) {
    let sign = if v1.z >= 0.0 { 1.0 } else { -1.0 };
    let a = -1.0 / (sign + v1.z);
    let b = v1.x * v1.y * a;
    
    let v2 = Vector3 {
        x: 1.0 + sign * v1.x * v1.x * a,
        y: sign * b,
        z: -sign * v1.x,
    };
    
    let v3 = Vector3 {
        x: b,
        y: sign + v1.y * v1.y * a,
        z: -v1.y,
    };
    
    (v2, v3)
}

// --- 2. The BxDF Enum (The Monolithic Shader) ---
// We use an Enum instead of dynamic Traits for CPU cache performance.
pub enum BxDF {
    Diffuse(DiffuseBxDF),
    // Future: Conductor(ConductorBxDF),
    // Future: Dielectric(DielectricBxDF),
}

impl BxDF {
    pub fn f(&self, wo: Vector3, wi: Vector3) -> SampledSpectrum {
        match self {
            BxDF::Diffuse(b) => b.f(wo, wi),
        }
    }

    pub fn sample_f(&self, wo: Vector3, u: Point2) -> Option<(SampledSpectrum, Vector3, f32)> {
        match self {
            BxDF::Diffuse(b) => b.sample_f(wo, u),
        }
    }

    pub fn pdf(&self, wo: Vector3, wi: Vector3) -> f32 {
        match self {
            BxDF::Diffuse(b) => b.pdf(wo, wi),
        }
    }
}

// --- 3. Diffuse BxDF (Lambertian) ---
// Simple matte surface. Reflects light equally in all directions.
pub struct DiffuseBxDF {
    pub r: SampledSpectrum, // Reflectance (Albedo)
}

impl DiffuseBxDF {
    pub fn new(r: SampledSpectrum) -> Self { Self { r } }

    // Lambert: f = R / PI
    pub fn f(&self, _wo: Vector3, _wi: Vector3) -> SampledSpectrum {
        self.r * (1.0 / PI)
    }

    // Cosine-weighted hemisphere sampling
    pub fn sample_f(&self, wo: Vector3, u: Point2) -> Option<(SampledSpectrum, Vector3, f32)> {
        // Importance sample the cosine term
        let wi = cosine_sample_hemisphere(u);
        
        // Ensure we are in the same hemisphere
        if wo.z * wi.z < 0.0 { return None; }

        let pdf = self.pdf(wo, wi);
        let f = self.f(wo, wi);
        
        Some((f, wi, pdf))
    }

    pub fn pdf(&self, _wo: Vector3, wi: Vector3) -> f32 {
        // PDF = cos(theta) / PI
        if wi.z <= 0.0 { 0.0 } else { wi.z * (1.0 / PI) }
    }
}

fn cosine_sample_hemisphere(u: Point2) -> Vector3 {
    let r = u.x.sqrt();
    let phi = 2.0 * PI * u.y;
    let x = r * phi.cos();
    let y = r * phi.sin();
    let z = (1.0 - x*x - y*y).max(0.0).sqrt();
    Vector3 { x, y, z }
}

// --- 4. The BSDF Container (Wrapper) ---
// This is what the Integrator interacts with.
pub struct BSDF {
    frame: Frame,
    bxdf: BxDF,
}

impl BSDF {
    pub fn new(normal: Vector3, bxdf: BxDF) -> Self {
        BSDF {
            frame: Frame::from_z(normal),
            bxdf,
        }
    }

    // Evaluates f() in World Space
    pub fn f(&self, wo_world: Vector3, wi_world: Vector3) -> SampledSpectrum {
        let wo = self.frame.to_local(wo_world);
        let wi = self.frame.to_local(wi_world);
        self.bxdf.f(wo, wi)
    }

    // Samples a new direction in World Space
    pub fn sample_f(&self, wo_world: Vector3, u: Point2) -> Option<(SampledSpectrum, Vector3, f32)> {
        let wo = self.frame.to_local(wo_world);
        
        if let Some((f, wi_local, pdf)) = self.bxdf.sample_f(wo, u) {
            let wi_world = self.frame.from_local(wi_local);
            Some((f, wi_world, pdf))
        } else {
            None
        }
    }
}