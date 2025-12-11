// We use 'dyn' traits because a primitive might hold a Triangle, Sphere, etc.
// For now, we assume these traits will exist in the future.

/* NOTE: You will need to define these Traits in your mod.rs or a traits.rs file soon:
   pub trait Shape: Send + Sync { ... }
   pub trait Material: Send + Sync { ... }
*/

use std::sync::Arc;

// Placeholder traits to make this compile for now
pub trait Shape {} 
pub trait Material {}

pub struct GeometricPrimitive {
    pub shape: Arc<dyn Shape + Send + Sync>,
    pub material: Option<Arc<dyn Material + Send + Sync>>,
    // pub area_light: Option<Arc<dyn AreaLight>>, 
}

impl GeometricPrimitive {
    pub fn new(shape: Arc<dyn Shape + Send + Sync>, material: Option<Arc<dyn Material + Send + Sync>>) -> Self {
        GeometricPrimitive {
            shape,
            material,
        }
    }
}