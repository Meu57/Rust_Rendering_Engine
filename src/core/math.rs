use std::f32;

/// A mathematical interval [low, high] that guarantees the true value is inside.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Interval {
    pub min: f32,
    pub max: f32,
}

impl Interval {
    // Exact value (thin interval)
    pub fn new(v: f32) -> Self {
        Interval { min: v, max: v }
    }

    // Interval with explicit error margin
    pub fn with_error(v: f32, error: f32) -> Self {
        Interval {
            min: next_float_down(v - error),
            max: next_float_up(v + error),
        }
    }

    // Merge two intervals (union)
    pub fn union(self, other: Interval) -> Self {
        Interval {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }
}

// --- Arithmetic Overloads for Interval Propagation ---

use std::ops::{Add, Mul, Sub};

impl Add for Interval {
    type Output = Interval;
    fn add(self, rhs: Interval) -> Interval {
        Interval {
            // Round down lower bound, Round up upper bound
            min: next_float_down(self.min + rhs.min),
            max: next_float_up(self.max + rhs.max),
        }
    }
}

impl Sub for Interval {
    type Output = Interval;
    fn sub(self, rhs: Interval) -> Interval {
        Interval {
            min: next_float_down(self.min - rhs.max), // Min - Max = Smallest
            max: next_float_up(self.max - rhs.min),   // Max - Min = Largest
        }
    }
}

impl Mul for Interval {
    type Output = Interval;
    fn mul(self, rhs: Interval) -> Interval {
        let p = [
            self.min * rhs.min, self.min * rhs.max,
            self.max * rhs.min, self.max * rhs.max
        ];
        
        // We can't rely on standard min/max because of rounding, 
        // so we manually check all 4 combinations with robust rounding.
        let min_val = p.iter().fold(f32::INFINITY, |a, &b| a.min(b));
        let max_val = p.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));

        Interval {
            min: next_float_down(min_val),
            max: next_float_up(max_val),
        }
    }
}

// --- The "Dark Arts": Bitwise Float Manipulation ---

// Nudge float to the next higher representable number (towards +infinity)
pub fn next_float_up(v: f32) -> f32 {
    if v.is_infinite() && v > 0.0 { return v; }
    if v == -0.0 { return 0.0; } // -0.0 -> +0.0

    let bits = v.to_bits();
    if v >= 0.0 {
        f32::from_bits(bits + 1)
    } else {
        f32::from_bits(bits - 1)
    }
}

// Nudge float to the next lower representable number (towards -infinity)
pub fn next_float_down(v: f32) -> f32 {
    if v.is_infinite() && v < 0.0 { return v; }
    if v == 0.0 { return -0.0; } // +0.0 -> -0.0

    let bits = v.to_bits();
    if v > 0.0 {
        f32::from_bits(bits - 1)
    } else {
        f32::from_bits(bits + 1)
    }
}

// --- Robust Quadratic Solver ---
// Solves At^2 + Bt + C = 0
// Returns Option<(t0, t1)> sorted by distance
pub fn solve_quadratic(a: f32, b: f32, c: f32) -> Option<(f32, f32)> {
    // 1. Precise Discriminant using f64 to avoid some cancellation
    // (In a full engine, we'd use the EFT method mentioned by the teacher)
    let discrim = (b as f64 * b as f64) - (4.0 * a as f64 * c as f64);
    
    if discrim < 0.0 { return None; }
    
    let root_discrim = discrim.sqrt() as f32;

    // 2. Stable Solution for q
    // Avoid subtraction: -0.5 * (b + sign(b)*sqrt(d))
    let q = if b < 0.0 {
        -0.5 * (b - root_discrim)
    } else {
        -0.5 * (b + root_discrim)
    };

    let t0 = q / a;
    let t1 = c / q;

    if t0 > t1 { Some((t1, t0)) } else { Some((t0, t1)) }
}

// --- Error-Free Arithmetic ---

/// Computes (a * b) - (c * d) with error correction using FMA.
/// This prevents catastrophic cancellation when the terms are nearly equal.
pub fn difference_of_products(a: f32, b: f32, c: f32, d: f32) -> f32 {
    let cd = c * d;
    // Rust's mul_add(a, b, c) computes (a * b) + c
    let diff = a.mul_add(b, -cd); 
    let error = (-c).mul_add(d, cd);
    diff + error
}

// --- Stable Hashing for Stochastic Alpha ---
pub fn hash_float(x: f32, y: f32, z: f32) -> f32 {
    let ix = x.to_bits();
    let iy = y.to_bits();
    let iz = z.to_bits();
    
    // MurmurHash3 mix constants
    let mut h = (ix ^ iy ^ iz).wrapping_mul(0xcc9e2d51);
    h = (h << 13) | (h >> 19);
    h = h.wrapping_mul(0x1b873593);
    h ^= h >> 16;
    h = h.wrapping_mul(0x85ebca6b);
    h ^= h >> 13;
    h = h.wrapping_mul(0xc2b2ae35);
    h ^= h >> 16;

    // Convert to float [0, 1]
    (h as f32) / (u32::MAX as f32)
}

use crate::core::geometry::Point2;
use std::f32::consts::PI;

// --- Sampling Helpers ---

/// Samples a point on a unit disk with uniform probability.
/// Used for generating random directions for lights and surfaces.
pub fn sample_uniform_disk_polar(u: Point2) -> Point2 {
    let r = u.x.sqrt();
    let theta = 2.0 * PI * u.y;
    Point2 {
        x: r * theta.cos(),
        y: r * theta.sin(),
    }
}

// --- PCG32 Random Number Generator ---
// Minimal implementation of the PCG32 algorithm.
// Fast, statistically good, and deterministic.
pub struct RNG {
    state: u64,
    inc: u64,
}

impl RNG {
    pub fn new(seed: u64, seq_index: u64) -> Self {
        let mut rng = RNG { state: 0, inc: (seq_index << 1) | 1 };
        rng.next_u32();
        rng.state = rng.state.wrapping_add(seed);
        rng.next_u32();
        rng
    }

    pub fn next_u32(&mut self) -> u32 {
        let oldstate = self.state;
        self.state = oldstate.wrapping_mul(6364136223846793005).wrapping_add(self.inc);
        let xorshifted = ((oldstate >> 18) ^ oldstate) >> 27;
        let rot = (oldstate >> 59) as u32;
        (xorshifted as u32).rotate_right(rot)
    }

    pub fn next_f32(&mut self) -> f32 {
        (self.next_u32() as f32) * 2.3283064365386963e-10 // x * 2^-32
    }
}