use crate::core::geometry::{Point3, Vector3};
use crate::core::math::RNG;

pub struct Perlin {
    perm_x: Vec<usize>,
    perm_y: Vec<usize>,
    perm_z: Vec<usize>,
    ranfloat: Vec<f32>, // Cache random floats for value noise/gradients
}

impl Perlin {
    pub fn new() -> Self {
        let mut rng = RNG::new(1234, 5678); // Fixed seed for reproducibility

        let ranfloat: Vec<f32> = (0..256).map(|_| rng.next_f32()).collect();
        let perm_x = Perlin::perlin_generate_perm(&mut rng);
        let perm_y = Perlin::perlin_generate_perm(&mut rng);
        let perm_z = Perlin::perlin_generate_perm(&mut rng);

        Perlin {
            perm_x,
            perm_y,
            perm_z,
            ranfloat,
        }
    }

    // Generate a shuffled array of 0..256
    fn perlin_generate_perm(rng: &mut RNG) -> Vec<usize> {
        let mut p: Vec<usize> = (0..256).collect();
        for i in (1..256).rev() {
            let target = (rng.next_u32() as usize) % (i + 1);
            p.swap(i, target);
        }
        p
    }

    // The Quintic Fade Curve: 6t^5 - 15t^4 + 10t^3
    fn trilinear_interp(c: [[[Vector3; 2]; 2]; 2], u: f32, v: f32, w: f32) -> f32 {
        let uu = u * u * u * (u * (u * 6.0 - 15.0) + 10.0);
        let vv = v * v * v * (v * (v * 6.0 - 15.0) + 10.0);
        let ww = w * w * w * (w * (w * 6.0 - 15.0) + 10.0);

        let mut accum = 0.0;
        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    let weight_v = Vector3::new(u - i as f32, v - j as f32, w - k as f32);
                    let idx_i = i as f32; let idx_j = j as f32; let idx_k = k as f32;
                    
                    accum += (idx_i * uu + (1.0 - idx_i) * (1.0 - uu)) *
                             (idx_j * vv + (1.0 - idx_j) * (1.0 - vv)) *
                             (idx_k * ww + (1.0 - idx_k) * (1.0 - ww)) *
                             c[i][j][k].dot(weight_v);
                }
            }
        }
        accum
    }

    pub fn noise(&self, p: Point3) -> f32 {
        let u = p.x - p.x.floor();
        let v = p.y - p.y.floor();
        let w = p.z - p.z.floor();

        let i = p.x.floor() as i32;
        let j = p.y.floor() as i32;
        let k = p.z.floor() as i32;

        let mut c = [[[Vector3::new(0.0,0.0,0.0); 2]; 2]; 2];

        for di in 0..2 {
            for dj in 0..2 {
                for dk in 0..2 {
                    // Hash lookup
                    let idx = self.perm_x[((i + di as i32) & 255) as usize] ^
                              self.perm_y[((j + dj as i32) & 255) as usize] ^
                              self.perm_z[((k + dk as i32) & 255) as usize];
                    
                    // Pick a random gradient vector from the hash
                    // (Simplified: generating random unit vector on the fly from seed)
                    // In production, we'd look up a precomputed table of 12 vectors.
                    // Here we use the hash to deterministically create a vector.
                    let hash = idx as u32; // basic mix
                    // This is a simple hash-to-vector trick (Standard Perlin)
                    // Ideally use precomputed gradients array.
                    // For now, let's use a crude approximation for Day 1:
                    let x = (self.ranfloat[idx as usize] * 2.0 - 1.0); 
                    // To keep it simple for this step, we will implement full gradient lookup in Day 2
                    // For today: Let's use simple Value Noise interpolation to verify plumbing, 
                    // OR stick to the Perlin Gradient formulation if we add the gradient array.
                    
                    // Let's implement the gradient lookup properly:
                    c[di][dj][dk] = self.get_gradient(idx); 
                }
            }
        }

        Self::trilinear_interp(c, u, v, w)
    }

    fn get_gradient(&self, hash: usize) -> Vector3 {
        // The 12 standard gradients for Perlin Noise
        let h = hash & 15;
        let u = if h < 8 { 1.0 } else { 0.0 }; // x or y ? (Simplified logic)
        // Actually, let's use the standard set:
        match h {
            0 => Vector3::new(1.0, 1.0, 0.0),  1 => Vector3::new(-1.0, 1.0, 0.0),
            2 => Vector3::new(1.0, -1.0, 0.0), 3 => Vector3::new(-1.0, -1.0, 0.0),
            4 => Vector3::new(1.0, 0.0, 1.0),  5 => Vector3::new(-1.0, 0.0, 1.0),
            6 => Vector3::new(1.0, 0.0, -1.0), 7 => Vector3::new(-1.0, 0.0, -1.0),
            8 => Vector3::new(0.0, 1.0, 1.0),  9 => Vector3::new(0.0, -1.0, 1.0),
            10 => Vector3::new(0.0, 1.0, -1.0), 11 => Vector3::new(0.0, -1.0, -1.0),
            12 => Vector3::new(1.0, 1.0, 0.0), 13 => Vector3::new(-1.0, 1.0, 0.0),
            14 => Vector3::new(0.0, 1.0, 1.0), _ => Vector3::new(0.0, -1.0, 1.0),
        }
    }

    // Fractal Brownian Motion (fBm)
    // Accumulates 'depth' layers of noise
    pub fn turb(&self, p: Point3, depth: usize) -> f32 {
        let mut accum = 0.0;
        let mut temp_p = p;
        let mut weight = 1.0;

        for _ in 0..depth {
            accum += weight * self.noise(temp_p);
            weight *= 0.5;
            // Scale point by 2.0 (lacunarity)
            temp_p = Point3::new(temp_p.x * 2.0, temp_p.y * 2.0, temp_p.z * 2.0);
        }
        
        accum.abs()
    }
}