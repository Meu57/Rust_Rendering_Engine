use crate::core::interaction::SurfaceInteraction;
use crate::core::geometry::{Point2, Vector3};
use std::f32::consts::PI;
use crate::core::transform::Transform;

/// Trait to map a 3D surface point to 2D Texture Coordinates (uv)
/// Attached to Textures, not Shapes, allowing one object to have multiple mappings.
pub trait TextureMapping2D {
    fn map(&self, si: &SurfaceInteraction) -> Point2;
}

// --- 1. UV Mapping (Default) ---
// Uses the mesh's own UV coordinates (from the triangle).
pub struct UVMapping2D {
    pub su: f32, pub sv: f32, // Scaling (Tiling)
    pub du: f32, pub dv: f32, // Offsets
}

impl Default for UVMapping2D {
    fn default() -> Self { 
        Self { su: 1.0, sv: 1.0, du: 0.0, dv: 0.0 } 
    }
}

impl TextureMapping2D for UVMapping2D {
    fn map(&self, si: &SurfaceInteraction) -> Point2 {
        // FIX: Access uv via 'core'
        Point2 {
            x: self.su * si.core.uv.x + self.du,
            y: self.sv * si.core.uv.y + self.dv,
        }
    }
}

// --- 2. Spherical Mapping (The Physics) ---
// Maps points to a sphere (Globe projection).
// Handles the "Singularity at the Poles" edge case.
pub struct SphericalMapping2D {
    pub world_to_texture: Transform,
}

impl SphericalMapping2D {
    pub fn new(world_to_texture: Transform) -> Self {
        Self { world_to_texture }
    }
}

impl TextureMapping2D for SphericalMapping2D {
    fn map(&self, si: &SurfaceInteraction) -> Point2 {
        // 1. Transform hit point to Local Texture Space (centered at 0,0,0)
        let p = self.world_to_texture.transform_point(si.core.p);
        
        // 2. Compute Theta (Latitude)
        // Safe vector normalization to handle potential zero length (rare but possible)
        let vec = Vector3::from(p); 
        let len = vec.length();
        if len == 0.0 { return Point2 { x: 0.0, y: 0.0 }; }
        
        // Clamp z/len to [-1, 1] to avoid NaN in acos if precision errors push it slightly over
        let theta = (p.z / len).clamp(-1.0, 1.0).acos();

        // 3. Compute Phi (Longitude) - THE SINGULARITY CHECK
        // At the poles (x=0, y=0), atan2 is undefined/unstable.
        // We detect if we are close to the pole (radius < epsilon).
        let phi = if (p.x * p.x + p.y * p.y) < 1e-5 {
            0.0 // Force smooth continuity at the pole
        } else {
            // Standard wrapping: atan2 returns [-PI, PI]
            let raw_phi = p.y.atan2(p.x);
            if raw_phi < 0.0 { raw_phi + 2.0 * PI } else { raw_phi }
        };

        // 4. Remap to [0, 1]
        // phi is [0, 2PI], theta is [0, PI]
        let u = phi * (1.0 / (2.0 * PI));
        let v = theta * (1.0 / PI);

        Point2 { x: u, y: v }
    }
}

// --- 3. Planar Mapping ---
// Like an X-Ray projector. Good for decals.
pub struct PlanarMapping2D {
    pub vs: Vector3, pub vt: Vector3, // Basis vectors for the plane
    pub ds: f32, pub dt: f32,         // Offsets
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