use crate::core::geometry::{Point3, Vector3, Normal3, Point2};
use crate::core::ray::Ray; // Add this import at top

/// Base struct for any interaction (Surface, Volume, Light)
#[derive(Debug, Clone)]
pub struct Interaction {
    pub p: Point3,       // The point of intersection
    pub time: f32,       // The time it happened
    pub p_error: Vector3,// Robustness: The error bounds (epsilon box)
    pub wo: Vector3,     // Outgoing direction (negative ray direction)
    pub n: Normal3,      // Geometric Normal
    pub uv: Point2,      // Texture Coordinates
}

/// Specialized struct for Surface Hits (Triangles, Spheres)
#[derive(Debug, Clone)]
pub struct SurfaceInteraction {
    pub core: Interaction,

    // Differential Geometry (for texture mapping)
    pub dpdu: Vector3,
    pub dpdv: Vector3,
    pub dndu: Normal3,
    pub dndv: Normal3,

    // Shading Geometry (Bump mapping, normal mapping results)
    pub shading: ShadingData,
}

#[derive(Debug, Clone)]
pub struct ShadingData {
    pub n: Normal3,
    pub dpdu: Vector3,
    pub dpdv: Vector3,
    pub dndu: Normal3,
    pub dndv: Normal3,
}

impl SurfaceInteraction {
    // Helper to create a basic interaction
    pub fn new(
        p: Point3, p_error: Vector3, uv: Point2, wo: Vector3,
        n: Normal3, time: f32
    ) -> Self {
        let core = Interaction { p, time, p_error, wo, n, uv };
        let shading = ShadingData { n, dpdu: Vector3{x:0.0,y:0.0,z:0.0}, dpdv: Vector3{x:0.0,y:0.0,z:0.0}, dndu: n, dndv: n }; // Defaults
        
        SurfaceInteraction {
            core,
            dpdu: Vector3{x:0.0,y:0.0,z:0.0}, 
            dpdv: Vector3{x:0.0,y:0.0,z:0.0},
            dndu: Normal3{x:0.0,y:0.0,z:0.0}, 
            dndv: Normal3{x:0.0,y:0.0,z:0.0},
            shading,
        }
    }
}

impl Interaction {
    // Spawns a new ray starting from this interaction point
    // directed along 'd'. Handles offset to prevent self-intersection.
    pub fn spawn_ray(&self, d: Vector3) -> Ray {
        // Robustness Note: In a full engine, we use self.p_error to 
        // strictly bound the offset. For Week 2, we use a shadow epsilon.
        let offset = d * 0.001;  // <<-----ERROR HERE>>
        let origin = self.p + offset;
        
        Ray::new(origin, d, self.time)
    }
}