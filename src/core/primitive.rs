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
        // 1. Geometric Hit?
        let hit = self.shape.intersect(ray, f32::INFINITY);
        
        if let Some((t_hit, interaction)) = hit {
            // 2. Alpha Test
            if self.alpha < 1.0 { // <<-----ERROR HERE>>
                let u = hash_float(interaction.core.p.x, interaction.core.p.y, interaction.core.p.z); // <<-----ERROR HERE>>
                
                // If we hit a "hole" (u > alpha)
                if u > self.alpha { // <<-----ERROR HERE>>
                    //
                    // Spawn a new ray starting at the hole, continuing forward
                    let next_ray = interaction.core.spawn_ray(ray.d);
                    
                    // RECURSE: "Is there more of me behind this hole?"
                    // For a triangle, this usually returns None immediately.
                    // For a Sphere, this might hit the back face.
                    if let Some((t_next, next_interaction)) = self.intersect(&next_ray) {
                        // CRITICAL: Adjust distance!
                        // The new hit is 't_next' away from the HOLE.
                        // We need the distance from the ORIGINAL ray origin.
                        return Some((t_hit + t_next, next_interaction));
                    } else {
                        // We passed through the object and hit nothing *inside* it.
                        // Return None so the Scene/BVH knows to keep looking for OTHER objects.
                        return None; 
                    }
                }
            }
            
            Some((t_hit, interaction))
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