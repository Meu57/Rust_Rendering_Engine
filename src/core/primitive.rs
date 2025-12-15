use std::sync::Arc;

use crate::core::geometry::{Bounds3, Point2, Point3, Normal3};
use crate::core::ray::Ray;
use crate::core::interaction::SurfaceInteraction;
use crate::core::transform::Transform;
use crate::core::math::hash_float; 
use crate::core::material::Material; 

// --- 1. The Shape Trait (Geometry + Sampling Support) ---
//
// UPDATED:
// - Added area()
// - Added sample()
//
// These are REQUIRED for area lights and NEE.
// No existing behavior is changed.
pub trait Shape: Send + Sync {
    fn bounds(&self) -> Bounds3;
    fn intersect(&self, ray: &Ray, t_max: f32)
        -> Option<(f32, SurfaceInteraction)>;

    // --- NEW: Area Light Support ---
    fn area(&self) -> f32;
    fn sample(&self, u: Point2) -> (Point3, Normal3);
}

// --- 2. The Primitive Trait ---
// Carries Material through intersection (unchanged)
pub trait Primitive: Send + Sync {
    fn bounds(&self) -> Bounds3;
    fn intersect(&self, ray: &Ray)
        -> Option<(f32, SurfaceInteraction, Option<Arc<dyn Material>>)>;
}

// --- Implementation A: GeometricPrimitive ---
pub struct GeometricPrimitive {
    pub shape: Arc<dyn Shape>,
    pub material: Option<Arc<dyn Material>>,
    pub alpha: f32, 
}

impl GeometricPrimitive {
    pub fn new(
        shape: Arc<dyn Shape>,
        material: Option<Arc<dyn Material>>,
        alpha: f32,
    ) -> Self {
        GeometricPrimitive { shape, material, alpha }
    }
}

impl Primitive for GeometricPrimitive {
    fn bounds(&self) -> Bounds3 {
        self.shape.bounds()
    }

    fn intersect(
        &self,
        ray: &Ray,
    ) -> Option<(f32, SurfaceInteraction, Option<Arc<dyn Material>>)> {
        let hit = self.shape.intersect(ray, f32::INFINITY);

        if let Some((t_hit, interaction)) = hit {
            // --- Stochastic Alpha Test ---
            if self.alpha < 1.0 {
                let u = hash_float(
                    interaction.core.p.x,
                    interaction.core.p.y,
                    interaction.core.p.z,
                );
                if u > self.alpha {
                    return None;
                }
            }

            Some((t_hit, interaction, self.material.clone()))
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
    pub fn new(
        primitive: Arc<dyn Primitive>,
        object_to_world: Transform,
    ) -> Self {
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

    fn intersect(
        &self,
        ray: &Ray,
    ) -> Option<(f32, SurfaceInteraction, Option<Arc<dyn Material>>)> {
        let transformed_ray = self.world_to_primitive.transform_ray(ray);

        if let Some((t, mut interaction, mat)) =
            self.primitive.intersect(&transformed_ray)
        {
            let primitive_to_world = self.world_to_primitive.inverse();

            interaction.core.p =
                primitive_to_world.transform_point(interaction.core.p);
            interaction.core.n =
                primitive_to_world.transform_normal(interaction.core.n);
            interaction.core.wo =
                primitive_to_world.transform_vector(interaction.core.wo);
            interaction.shading.n =
                primitive_to_world.transform_normal(interaction.shading.n);

            Some((t, interaction, mat))
        } else {
            None
        }
    }
}

// --- Implementation C: PrimitiveList (The Scene) ---
pub struct PrimitiveList {
    pub primitives: Vec<Arc<dyn Primitive>>,
}

impl PrimitiveList {
    pub fn new(primitives: Vec<Arc<dyn Primitive>>) -> Self {
        PrimitiveList { primitives }
    }
}

impl Primitive for PrimitiveList {
    fn bounds(&self) -> Bounds3 {
        if self.primitives.is_empty() {
            return Bounds3::new(
                Point3::new(0.0, 0.0, 0.0),
                Point3::new(0.0, 0.0, 0.0),
            );
        }

        let mut b = self.primitives[0].bounds();
        for p in &self.primitives[1..] {
            let pb = p.bounds();
            b = Bounds3::new(
                Point3::new(
                    b.min.x.min(pb.min.x),
                    b.min.y.min(pb.min.y),
                    b.min.z.min(pb.min.z),
                ),
                Point3::new(
                    b.max.x.max(pb.max.x),
                    b.max.y.max(pb.max.y),
                    b.max.z.max(pb.max.z),
                ),
            );
        }
        b
    }

    fn intersect(
        &self,
        ray: &Ray,
    ) -> Option<(f32, SurfaceInteraction, Option<Arc<dyn Material>>)> {
        let mut closest_t = f32::INFINITY;
        let mut closest_hit = None;

        for p in &self.primitives {
            if let Some((t, interaction, mat)) = p.intersect(ray) {
                if t < closest_t {
                    closest_t = t;
                    closest_hit = Some((t, interaction, mat));
                }
            }
        }
        closest_hit
    }
}
