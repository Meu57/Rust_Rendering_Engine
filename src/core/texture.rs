use crate::core::interaction::SurfaceInteraction;
use crate::core::geometry::{Point2, Vector3, Point3};
use crate::core::spectrum::SampledSpectrum;
use crate::core::transform::Transform;
use crate::core::noise::Perlin; 
use std::f32::consts::PI;

// --- 1. The Texture Trait (Moved from imagemap.rs) ---
pub trait Texture: Send + Sync {
    fn evaluate(&self, si: &SurfaceInteraction) -> SampledSpectrum;
}

// --- 2. Noise Texture (Procedural) ---
pub struct NoiseTexture {
    noise: Perlin,
    scale: f32, // Controls frequency (how "zoomed out" the noise is)
}

impl NoiseTexture {
    pub fn new(scale: f32) -> Self {
        NoiseTexture {
            noise: Perlin::new(),
            scale,
        }
    }
}

impl Texture for NoiseTexture {
    fn evaluate(&self, si: &SurfaceInteraction) -> SampledSpectrum {
        // FIX: Point3 * f32 is not implemented. Scale manually.
        let p = Point3::new(
            si.core.p.x * self.scale,
            si.core.p.y * self.scale,
            si.core.p.z * self.scale
        );
        
        // Evaluate Perlin Noise at this point
        // noise() returns -1.0 to 1.0 (approx). 
        // We map it to 0.0 to 1.0 for color.
        let n = self.noise.noise(p);
        let val = 0.5 * (1.0 + n); 
        
        // Return as Greyscale
        SampledSpectrum::splat(val)
    }
}

// --- 3. Constant Texture (Helper) ---
pub struct ConstantTexture {
    value: SampledSpectrum,
}

impl ConstantTexture {
    pub fn new(value: SampledSpectrum) -> Self { Self { value } }
}

impl Texture for ConstantTexture {
    fn evaluate(&self, _si: &SurfaceInteraction) -> SampledSpectrum {
        self.value
    }
}

// --- 4. Texture Mappings ---

// FIX: Added 'Send + Sync' requirement so it can be used in Texture
pub trait TextureMapping2D: Send + Sync {
    fn map(&self, si: &SurfaceInteraction) -> Point2;
}

pub struct UVMapping2D {
    pub su: f32, pub sv: f32,
    pub du: f32, pub dv: f32,
}
impl Default for UVMapping2D {
    fn default() -> Self { Self { su: 1.0, sv: 1.0, du: 0.0, dv: 0.0 } }
}
impl TextureMapping2D for UVMapping2D {
    fn map(&self, si: &SurfaceInteraction) -> Point2 {
        Point2 {
            x: self.su * si.core.uv.x + self.du,
            y: self.sv * si.core.uv.y + self.dv,
        }
    }
}

pub struct SphericalMapping2D {
    pub world_to_texture: Transform,
}
impl SphericalMapping2D {
    pub fn new(world_to_texture: Transform) -> Self { Self { world_to_texture } }
}
impl TextureMapping2D for SphericalMapping2D {
    fn map(&self, si: &SurfaceInteraction) -> Point2 {
        let p = self.world_to_texture.transform_point(si.core.p);
        let vec = Vector3::from(p); 
        let len = vec.length();
        if len == 0.0 { return Point2 { x: 0.0, y: 0.0 }; }
        let theta = (p.z / len).clamp(-1.0, 1.0).acos();
        let phi = if (p.x * p.x + p.y * p.y) < 1e-5 { 0.0 } else {
            let raw = p.y.atan2(p.x);
            if raw < 0.0 { raw + 2.0 * PI } else { raw }
        };
        Point2 { x: phi / (2.0 * PI), y: theta / PI }
    }
}

pub struct PlanarMapping2D {
    pub vs: Vector3, pub vt: Vector3, pub ds: f32, pub dt: f32,
}
impl TextureMapping2D for PlanarMapping2D {
    fn map(&self, si: &SurfaceInteraction) -> Point2 {
        let vec = Vector3::from(si.core.p);
        Point2 {
            x: self.ds + vec.dot(self.vs),
            y: self.dt + vec.dot(self.vt),
        }
    }
}