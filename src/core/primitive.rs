use std::sync::Arc;
use crate::core::geometry::{Bounds3, Point3};
use crate::core::ray::Ray;
use crate::core::interaction::SurfaceInteraction;
use crate::core::transform::Transform;
use crate::core::math::hash_float; 

// --- 1. The Shape Trait (Geometry Only) ---
pub trait Shape: Send + Sync {
    fn bounds(&self) -> Bounds3;
    fn intersect(&self, ray: &Ray, t_max: f32) -> Option<(f32, SurfaceInteraction)>;
}

// --- 2. The Material Trait (Appearance) ---
pub trait Material: Send + Sync {}

// --- 3. The Primitive Trait (Scene Objects) ---
pub trait Primitive: Send + Sync {
    fn bounds(&self) -> Bounds3;
    fn intersect(&self, ray: &Ray) -> Option<(f32, SurfaceInteraction)>;
}

// --- Implementation A: GeometricPrimitive ---
pub struct GeometricPrimitive {
    pub shape: Arc<dyn Shape>,
    pub material: Option<Arc<dyn Material>>,
    pub alpha: f32, 
}

impl GeometricPrimitive {
    pub fn new(shape: Arc<dyn Shape>, material: Option<Arc<dyn Material>>, alpha: f32) -> Self {
        GeometricPrimitive { shape, material, alpha }
    }
}

impl Primitive for GeometricPrimitive {
    fn bounds(&self) -> Bounds3 {
        self.shape.bounds()
    }

    fn intersect(&self, ray: &Ray) -> Option<(f32, SurfaceInteraction)> {
        let hit = self.shape.intersect(ray, f32::INFINITY);
        
        if let Some((t_hit, interaction)) = hit {
            if self.alpha < 1.0 {
                let u = hash_float(interaction.core.p.x, interaction.core.p.y, interaction.core.p.z);
                if u > self.alpha {
                    let next_ray = interaction.core.spawn_ray(ray.d);
                    if let Some((t_next, next_interaction)) = self.intersect(&next_ray) {
                        return Some((t_hit + t_next, next_interaction));
                    } else {
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

// --- Implementation B: TransformedPrimitive ---
pub struct TransformedPrimitive {
    pub primitive: Arc<dyn Primitive>,
    pub world_to_primitive: Transform, 
}

impl TransformedPrimitive {
    pub fn new(primitive: Arc<dyn Primitive>, object_to_world: Transform) -> Self {
        TransformedPrimitive {
            primitive,
            world_to_primitive: object_to_world.inverse(), 
        }
    }
}

impl Primitive for TransformedPrimitive {
    fn bounds(&self) -> Bounds3 {
        self.primitive.bounds() 
    }

    fn intersect(&self, ray: &Ray) -> Option<(f32, SurfaceInteraction)> {
        // --- FIX: Use helper instead of manual construction ---
        let transformed_ray = self.world_to_primitive.transform_ray(ray);

        if let Some((t, mut interaction)) = self.primitive.intersect(&transformed_ray) {
            let primitive_to_world = self.world_to_primitive.inverse();
            interaction.core.p = primitive_to_world.transform_point(interaction.core.p);
            interaction.core.n = primitive_to_world.transform_normal(interaction.core.n);
            interaction.core.wo = primitive_to_world.transform_vector(interaction.core.wo);
            interaction.shading.n = primitive_to_world.transform_normal(interaction.shading.n);
            Some((t, interaction))
        } else {
            None
        }
    }
}