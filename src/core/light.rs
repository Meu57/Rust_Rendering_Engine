use std::sync::Arc;

use crate::core::geometry::{Point2, Point3, Vector3};
use crate::core::interaction::SurfaceInteraction;
use crate::core::primitive::Shape;
use crate::core::spectrum::SampledSpectrum;

/// Result of sampling a light source (incident radiance at a point)
pub struct LightLiSample {
    /// Incident radiance arriving at the shading point
    pub l: SampledSpectrum,

    /// Direction *towards* the light (world space, normalized)
    pub wi: Vector3,

    /// PDF with respect to solid angle
    pub pdf: f32,

    /// Sampled point on the light (used for shadow ray distance checks)
    pub p_light: Point3,
}

/// Light interface used by the integrator for Next Event Estimation
pub trait Light: Send + Sync {
    /// Sample incident radiance from this light at a surface point
    fn sample_li(
        &self,
        ctx: &SurfaceInteraction,
        u: Point2,
    ) -> Option<LightLiSample>;

    /// PDF of sampling direction `wi` from `ctx` (solid angle measure)
    fn pdf_li(&self, ctx: &SurfaceInteraction, wi: Vector3) -> f32;

    /// Is this a delta light? (point / directional)
    fn is_delta(&self) -> bool;
}

/// Diffuse area light backed by a geometric shape
pub struct DiffuseAreaLight {
    pub shape: Arc<dyn Shape>,
    pub l_emit: SampledSpectrum, // Emitted radiance (Le)
    pub area: f32,               // Cached surface area
}

impl DiffuseAreaLight {
    pub fn new(shape: Arc<dyn Shape>, l_emit: SampledSpectrum) -> Self {
        let area = shape.area();
        DiffuseAreaLight {
            shape,
            l_emit,
            area,
        }
    }
}

impl Light for DiffuseAreaLight {
    fn is_delta(&self) -> bool {
        false
    }

    fn sample_li(
        &self,
        ctx: &SurfaceInteraction,
        u: Point2,
    ) -> Option<LightLiSample> {
        // 1. Sample a point uniformly on the light (area measure)
        let (p_light, n_light) = self.shape.sample(u);

        // 2. Direction to light
        let wi_vec = p_light - ctx.core.p;
        let dist_sq = wi_vec.length_squared();
        if dist_sq == 0.0 {
            return None;
        }

        let dist = dist_sq.sqrt();
        // FIX: Vector3 does not implement Div<f32>, use multiplication by reciprocal
        let wi = wi_vec * (1.0 / dist);

        // 3. Backface culling (light must face the shading point)
        // FIX: Convert Normal3 to Vector3 for dot product
        let cos_theta_light = Vector3::from(n_light).dot(-wi);
        if cos_theta_light <= 0.0 {
            return None;
        }

        // 4. Convert area PDF to solid angle PDF
        //
        // pdf_omega = (dist^2) / (area * cos_theta_light)
        let pdf = dist_sq / (self.area * cos_theta_light);
        if !pdf.is_finite() || pdf <= 0.0 {
            return None;
        }

        Some(LightLiSample {
            l: self.l_emit,
            wi,
            pdf,
            p_light,
        })
    }

    fn pdf_li(&self, _ctx: &SurfaceInteraction, _wi: Vector3) -> f32 {
        // Proper implementation requires ray–light intersection testing.
        // This is intentionally left as 0 until MIS is wired correctly.
        0.0
    }
}