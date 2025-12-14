use std::sync::Arc;
use crate::core::geometry::{Point3, Vector3, Normal3, Point2, Bounds3};
use crate::core::ray::Ray;
use crate::core::interaction::{SurfaceInteraction, ShadingData};
use crate::core::math::{difference_of_products};
use crate::core::primitive::Shape;

// --- The Mesh Data ---
pub struct TriangleMesh {
    pub n_triangles: usize,
    pub vertex_indices: Vec<usize>,
    pub p: Vec<Point3>,
    pub n: Option<Vec<Normal3>>,
    pub uv: Option<Vec<Point2>>,
}

impl TriangleMesh {
    pub fn new(indices: Vec<usize>, p: Vec<Point3>, n: Option<Vec<Normal3>>, uv: Option<Vec<Point2>>) -> Self {
        TriangleMesh {
            n_triangles: indices.len() / 3,
            vertex_indices: indices,
            p,
            n,
            uv,
        }
    }
}

// --- The Triangle Shape ---
#[derive(Clone)]
pub struct Triangle {
    pub mesh: Arc<TriangleMesh>,
    pub v_index: usize,
}

impl Triangle {
    pub fn new(mesh: Arc<TriangleMesh>, tri_number: usize) -> Self {
        Triangle {
            mesh,
            v_index: tri_number * 3,
        }
    }
}

impl Shape for Triangle {
    fn bounds(&self) -> Bounds3 {
        let idx = &self.mesh.vertex_indices;
        let p0 = self.mesh.p[idx[self.v_index]];
        let p1 = self.mesh.p[idx[self.v_index + 1]];
        let p2 = self.mesh.p[idx[self.v_index + 2]];
        Bounds3::new(p0, p1).union_point(p2)
    }

    fn intersect(&self, ray: &Ray, t_max: f32) -> Option<(f32, SurfaceInteraction)> {
        let idx = &self.mesh.vertex_indices;
        let p0 = self.mesh.p[idx[self.v_index]];
        let p1 = self.mesh.p[idx[self.v_index + 1]];
        let p2 = self.mesh.p[idx[self.v_index + 2]];

        // 1. Permutation
        let abs_d = Vector3 { x: ray.d.x.abs(), y: ray.d.y.abs(), z: ray.d.z.abs() };
        let kz = if abs_d.x > abs_d.y {
            if abs_d.x > abs_d.z { 0 } else { 2 }
        } else {
            if abs_d.y > abs_d.z { 1 } else { 2 }
        };
        let kx = (kz + 1) % 3;
        let ky = (kx + 1) % 3;

        let permute = |v: Vector3| -> Vector3 {
            let c = [v.x, v.y, v.z];
            Vector3 { x: c[kx], y: c[ky], z: c[kz] }
        };

        let d = permute(ray.d);
        let p0t_vec = permute(p0 - ray.o);
        let p1t_vec = permute(p1 - ray.o);
        let p2t_vec = permute(p2 - ray.o);

        // 2. Shear
        let sx = -d.x / d.z;
        let sy = -d.y / d.z;
        let sz = 1.0 / d.z;

        let mut p0t = Vector3 { x: p0t_vec.x + sx * p0t_vec.z, y: p0t_vec.y + sy * p0t_vec.z, z: p0t_vec.z };
        let mut p1t = Vector3 { x: p1t_vec.x + sx * p1t_vec.z, y: p1t_vec.y + sy * p1t_vec.z, z: p1t_vec.z };
        let mut p2t = Vector3 { x: p2t_vec.x + sx * p2t_vec.z, y: p2t_vec.y + sy * p2t_vec.z, z: p2t_vec.z };

        // 3. Edge Functions
        let mut e0 = difference_of_products(p1t.x, p2t.y, p1t.y, p2t.x); 
        let mut e1 = difference_of_products(p2t.x, p0t.y, p2t.y, p0t.x);
        let mut e2 = difference_of_products(p0t.x, p1t.y, p0t.y, p1t.x);

        // 4. Fallback for precision
        if e0 == 0.0 || e1 == 0.0 || e2 == 0.0 {
            let p2txp1ty = (p2t.x as f64) * (p1t.y as f64);
            let p2typ1tx = (p2t.y as f64) * (p1t.x as f64);
            e0 = (p2typ1tx - p2txp1ty) as f32;

            let p0txp2ty = (p0t.x as f64) * (p2t.y as f64);
            let p0typ2tx = (p0t.y as f64) * (p2t.x as f64);
            e1 = (p0typ2tx - p0txp2ty) as f32;

            let p1txp0ty = (p1t.x as f64) * (p0t.y as f64);
            let p1typ0tx = (p1t.y as f64) * (p0t.x as f64);
            e2 = (p1typ0tx - p1txp0ty) as f32;
        }

        if (e0 < 0.0 || e1 < 0.0 || e2 < 0.0) && (e0 > 0.0 || e1 > 0.0 || e2 > 0.0) {
            return None;
        }

        let det = e0 + e1 + e2;
        if det == 0.0 { return None; }

        p0t.z *= sz;
        p1t.z *= sz;
        p2t.z *= sz;
        
        let t_scaled = e0 * p0t.z + e1 * p1t.z + e2 * p2t.z;

        if (det < 0.0 && (t_scaled >= 0.0 || t_scaled < t_max * det)) || 
           (det > 0.0 && (t_scaled <= 0.0 || t_scaled > t_max * det)) {
            return None;
        }

        let inv_det = 1.0 / det;
        let t = t_scaled * inv_det;
        
        // --- BARYCENTRIC INTERPOLATION ---
        let b0 = e0 * inv_det;
        let b1 = e1 * inv_det;
        let b2 = e2 * inv_det;

        // Interpolate UVs
        let uv = if let Some(uvs) = &self.mesh.uv {
            let uv0 = uvs[idx[self.v_index]];
            let uv1 = uvs[idx[self.v_index + 1]];
            let uv2 = uvs[idx[self.v_index + 2]];
            
            Point2 {
                x: b0 * uv0.x + b1 * uv1.x + b2 * uv2.x,
                y: b0 * uv0.y + b1 * uv1.y + b2 * uv2.y,
            }
        } else {
            Point2 { x: 0.0, y: 0.0 }
        };

        // --- FIX: Calculate Real Geometric Normal ---
        // Cross product of two edges gives the true normal of the plane
        let edge1 = p1 - p0;
        let edge2 = p2 - p0;
        let n_geom = Normal3::from(edge1.cross(edge2).normalize());

        // Ensure normal faces the ray (if single sided) or just pass it through
        // For now, we trust the winding order.
        
        let p_hit = ray.at(t);
        let p_error = Vector3{x:0.0, y:0.0, z:0.0}; 
        
        let interaction = SurfaceInteraction::new(
            p_hit, 
            p_error, 
            uv,
            -ray.d, 
            n_geom, 
            ray.time
        );

        Some((t, interaction))
    }
}