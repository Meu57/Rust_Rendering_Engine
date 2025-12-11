use std::sync::Arc;
use crate::core::geometry::{Bounds3, Point3};
use crate::core::ray::Ray;
use crate::core::interaction::SurfaceInteraction;
use crate::core::transform::Transform;

// --- 1. The Shape Trait (Geometry Only) ---
// Knows WHERE it is, but not WHAT it is.
pub trait Shape: Send + Sync {
    fn bounds(&self) -> Bounds3;
    fn intersect(&self, ray: &Ray, t_max: f32) -> Option<(f32, SurfaceInteraction)>;
}

// --- 2. The Material Trait (Appearance) ---
// Placeholder for now (Week 3 topic)
pub trait Material: Send + Sync {
    // fn compute_scattering(&self, si: &mut SurfaceInteraction);
}

// --- 3. The Primitive Trait (Scene Objects) ---
// The main interface for the scene graph.
pub trait Primitive: Send + Sync {
    fn bounds(&self) -> Bounds3;
    fn intersect(&self, ray: &Ray) -> Option<(f32, SurfaceInteraction)>;
}

// --- Implementation A: GeometricPrimitive (A Shape + Material) ---
pub struct GeometricPrimitive {
    pub shape: Arc<dyn Shape>,
    pub material: Option<Arc<dyn Material>>,
}

impl GeometricPrimitive {
    pub fn new(shape: Arc<dyn Shape>, material: Option<Arc<dyn Material>>) -> Self {
        GeometricPrimitive { shape, material }
    }
}

impl Primitive for GeometricPrimitive {
    fn bounds(&self) -> Bounds3 {
        self.shape.bounds()
    }

    fn intersect(&self, ray: &Ray) -> Option<(f32, SurfaceInteraction)> {
        // 1. Ask Shape for geometric hit
        let hit = self.shape.intersect(ray, f32::INFINITY);
        
        if let Some((t, mut interaction)) = hit {
            // 2. Bind Material (Decoration)
            // interaction.primitive = Some(self); // In full engine, we link back
            Some((t, interaction))
        } else {
            None
        }
    }
}

// --- Implementation B: TransformedPrimitive (Instancing) ---
//
pub struct TransformedPrimitive {
    pub primitive: Arc<dyn Primitive>,
    pub world_to_primitive: Transform, // World -> Object matrix
}

impl TransformedPrimitive {
    pub fn new(primitive: Arc<dyn Primitive>, object_to_world: Transform) -> Self {
        TransformedPrimitive {
            primitive,
            world_to_primitive: object_to_world.inverse(), // Store the inverse for rays
        }
    }
}

impl Primitive for TransformedPrimitive {
    fn bounds(&self) -> Bounds3 {
        // Transform the inner bounds to world space (Implementation omitted for brevity)
        // In real code: transform_bounds(self.primitive.bounds(), inverse(world_to_primitive))
        self.primitive.bounds() 
    }

    fn intersect(&self, ray: &Ray) -> Option<(f32, SurfaceInteraction)> {
        // 1. Transform Ray to Object Space
        let transformed_ray = Ray {
            o: self.world_to_primitive.transform_point(ray.o),
            d: self.world_to_primitive.transform_vector(ray.d),
            time: ray.time,
        };

        // 2. Intersect inner primitive
        if let Some((t, mut interaction)) = self.primitive.intersect(&transformed_ray) {
            // 3. Transform Interaction back to World Space
            // We must undo the transform for the hit point and normal
            let primitive_to_world = self.world_to_primitive.inverse();
            
            // FIX: Access fields via '.core'
            interaction.core.p = primitive_to_world.transform_point(interaction.core.p);
            interaction.core.n = primitive_to_world.transform_normal(interaction.core.n);
            interaction.core.wo = primitive_to_world.transform_vector(interaction.core.wo);

            // Also transform the shading normal (essential for lighting later)
            interaction.shading.n = primitive_to_world.transform_normal(interaction.shading.n);

            Some((t, interaction))
        } else {
            None
        }
    }
}