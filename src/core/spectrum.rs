use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub};
use crate::core::math::Interval; // Assuming we might need math helpers

// --- CONSTANTS ---
pub const N_SPECTRUM_SAMPLES: usize = 4; // The "Sweet Spot"
const LAMBDA_MIN: f32 = 400.0;
const LAMBDA_MAX: f32 = 700.0;

// --- 1. The Wavelength Context ---
// Carries the "Domain" of the spectrum (Which wavelengths are we tracking?)
#[derive(Debug, Clone, Copy)]
pub struct SampledWavelengths {
    pub lambda: [f32; N_SPECTRUM_SAMPLES],
    pub pdf: [f32; N_SPECTRUM_SAMPLES],
}

impl SampledWavelengths {
    // Generate wavelengths. Ideally, this uses Stratified Sampling.
    // For now, we pick a random start and space them evenly.
    pub fn sample_uniform(u: f32) -> Self {
        let mut lambda = [0.0; N_SPECTRUM_SAMPLES];
        let mut pdf = [0.0; N_SPECTRUM_SAMPLES];
        
        // Map u [0,1] to the first bucket's offset
        let bucket_width = (LAMBDA_MAX - LAMBDA_MIN) / (N_SPECTRUM_SAMPLES as f32);
        
        for i in 0..N_SPECTRUM_SAMPLES {
            let offset = u * bucket_width; // Stratified jitter
            let val = LAMBDA_MIN + (i as f32 * bucket_width) + offset;
            
            // Handle wrap-around (Periodic sampling) if needed, 
            // but for simple uniform sampling, we keep it linear here.
            lambda[i] = val;
            pdf[i] = 1.0 / (LAMBDA_MAX - LAMBDA_MIN); // Uniform PDF
        }

        SampledWavelengths { lambda, pdf }
    }
}

// --- 2. The Energy Container (SampledSpectrum) ---
// Replaces the old RGB Spectrum struct.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SampledSpectrum {
    pub values: [f32; N_SPECTRUM_SAMPLES],
}

impl SampledSpectrum {
    pub fn new(val: f32) -> Self {
        SampledSpectrum { values: [val; N_SPECTRUM_SAMPLES] }
    }

    pub fn splat(val: f32) -> Self {
        SampledSpectrum { values: [val; N_SPECTRUM_SAMPLES] }
    }
    
    // Convert RGB to Spectrum (Upsampling)
    // NOTE: This is a placeholder for the "Sigmoid Polynomial" table.
    // We use a constant reflection model for now to allow compilation.
    pub fn from_rgb(rgb: [f32; 3], _lambdas: &SampledWavelengths) -> Self {
        // Naive approximation: Average the RGB energy
        // In Month 4, this gets replaced by the Sigmoid Table Lookup.
        let avg = (rgb[0] + rgb[1] + rgb[2]) / 3.0;
        SampledSpectrum::splat(avg)
    }

    // Convert Spectrum back to XYZ (Integration)
    pub fn to_xyz(&self, lambdas: &SampledWavelengths) -> [f32; 3] {
        let mut x = 0.0;
        let mut y = 0.0;
        let mut z = 0.0;

        for i in 0..N_SPECTRUM_SAMPLES {
            let wav = lambdas.lambda[i];
            let val = self.values[i];
            let pdf = lambdas.pdf[i];

            // Integrate against CIE matching functions
            if pdf != 0.0 {
                x += val * cie_x(wav) / pdf;
                y += val * cie_y(wav) / pdf;
                z += val * cie_z(wav) / pdf;
            }
        }
        
        // Monte Carlo Average
        let n = N_SPECTRUM_SAMPLES as f32;
        [x / n, y / n, z / n]
    }
    
    // Helper to convert XYZ to RGB (Standard sRGB matrix)
    pub fn xyz_to_rgb(xyz: [f32; 3]) -> [f32; 3] {
        let x = xyz[0]; let y = xyz[1]; let z = xyz[2];
        [
            3.240479 * x - 1.537150 * y - 0.498535 * z,
            -0.969256 * x + 1.875991 * y + 0.041556 * z,
            0.055648 * x - 0.204043 * y + 1.057311 * z
        ]
    }
}

// --- Analytic CIE Matching Functions (Gaussian Fits) ---
// [Source: Wyman et al. "Simple Analytic Approximations to the CIE XYZ Color Matching Functions"]
fn g(x: f32, alpha: f32, mu: f32, sigma1: f32, sigma2: f32) -> f32 {
    let t = (x - mu) / (if x < mu { sigma1 } else { sigma2 });
    alpha * (-0.5 * t * t).exp()
}

fn cie_x(lambda: f32) -> f32 {
    g(lambda, 1.056, 599.8, 37.9, 31.0) + 
    g(lambda, 0.362, 442.0, 16.0, 26.7) + 
    g(lambda, -0.065, 501.1, 20.4, 26.2)
}

fn cie_y(lambda: f32) -> f32 {
    g(lambda, 0.821, 568.8, 46.9, 40.5) + 
    g(lambda, 0.286, 530.9, 16.3, 31.1)
}

fn cie_z(lambda: f32) -> f32 {
    g(lambda, 1.217, 437.0, 11.8, 36.0) + 
    g(lambda, 0.681, 459.0, 26.0, 13.8)
}

// --- Operator Overloads (Component-wise) ---

impl Add for SampledSpectrum {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        let mut v = [0.0; N_SPECTRUM_SAMPLES];
        for i in 0..N_SPECTRUM_SAMPLES { v[i] = self.values[i] + rhs.values[i]; }
        SampledSpectrum { values: v }
    }
}

impl Mul for SampledSpectrum {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        let mut v = [0.0; N_SPECTRUM_SAMPLES];
        for i in 0..N_SPECTRUM_SAMPLES { v[i] = self.values[i] * rhs.values[i]; }
        SampledSpectrum { values: v }
    }
}

impl Mul<f32> for SampledSpectrum {
    type Output = Self;
    fn mul(self, scalar: f32) -> Self {
        let mut v = [0.0; N_SPECTRUM_SAMPLES];
        for i in 0..N_SPECTRUM_SAMPLES { v[i] = self.values[i] * scalar; }
        SampledSpectrum { values: v }
    }
}

// --- 3. The Radiometric Wrappers (Updated) ---
// These now wrap SampledSpectrum instead of the old RGB Spectrum

#[derive(Debug, Clone, Copy)]
pub struct Flux(pub SampledSpectrum);

#[derive(Debug, Clone, Copy)]
pub struct Irradiance(pub SampledSpectrum);

#[derive(Debug, Clone, Copy)]
pub struct Radiance(pub SampledSpectrum);

// Wrapper Math
impl Add for Radiance {
    type Output = Radiance;
    fn add(self, rhs: Self) -> Self { Radiance(self.0 + rhs.0) }
}
impl Mul<f32> for Radiance {
    type Output = Radiance;
    fn mul(self, scalar: f32) -> Self { Radiance(self.0 * scalar) }
}
impl Mul<SampledSpectrum> for Radiance {
    type Output = Radiance;
    fn mul(self, color: SampledSpectrum) -> Self { Radiance(self.0 * color) }
}
impl Div<f32> for Flux {
    type Output = Irradiance;
    fn div(self, area: f32) -> Irradiance { Irradiance(self.0 * (1.0 / area)) }
}

// --- Week 3 Days 6-7: Blackbody Radiation ---

// Physical Constants
const C: f32 = 299792458.0;       // Speed of Light [m/s]
const H: f32 = 6.62606957e-34;    // Planck's Constant [J s]
const KB: f32 = 1.3806488e-23;    // Boltzmann Constant [J/K]

/// Calculates Blackbody Radiance for a given wavelength (nm) and temperature (K)
/// Uses Planck's Law: L(lambda, T)
pub fn blackbody(lambda_nm: f32, temp_k: f32) -> f32 {
    if temp_k <= 0.0 || lambda_nm <= 0.0 { return 0.0; }

    // Convert nanometers to meters for the physics formula
    let l = lambda_nm * 1.0e-9;

    // Exponent: (h * c) / (lambda * kB * T)
    let exponent = (H * C) / (l * KB * temp_k);
    
    // Denominator: lambda^5 * (e^exponent - 1)
    let denominator = l.powi(5) * (exponent.exp() - 1.0);

    // Result: (2 * h * c^2) / denominator
    (2.0 * H * C * C) / denominator
}

/// A normalized Blackbody Spectrum (Peak = 1.0)
/// We separate Color (Temperature) from Intensity (Brightness scale)
#[derive(Debug, Clone, Copy)]
pub struct BlackbodySpectrum {
    pub temp_k: f32,
    pub normalization_factor: f32,
}

impl BlackbodySpectrum {
    pub fn new(temp_k: f32) -> Self {
        // Wien's Displacement Law: Find the peak wavelength (in meters)
        // b approx 2.8977721e-3 m K
        let lambda_max_meters = 2.8977721e-3 / temp_k;
        
        // Convert to nm to query our blackbody function
        let peak_val = blackbody(lambda_max_meters * 1.0e9, temp_k);
        
        Self {
            temp_k,
            normalization_factor: if peak_val > 0.0 { 1.0 / peak_val } else { 0.0 },
        }
    }

    // Evaluate the spectrum at a specific wavelength
    pub fn eval(&self, lambda: f32) -> f32 {
        blackbody(lambda, self.temp_k) * self.normalization_factor
    }

    // Helper to generate a SampledSpectrum from this Blackbody
    pub fn sample(&self, lambdas: &SampledWavelengths) -> SampledSpectrum {
        let mut values = [0.0; N_SPECTRUM_SAMPLES];
        for i in 0..N_SPECTRUM_SAMPLES {
            values[i] = self.eval(lambdas.lambda[i]);
        }
        SampledSpectrum { values }
    }
}

// --- Missing Math Implementations for Week 5 ---

impl Sub for SampledSpectrum {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        let mut v = [0.0; N_SPECTRUM_SAMPLES];
        for i in 0..N_SPECTRUM_SAMPLES { 
            v[i] = self.values[i] - rhs.values[i]; 
        }
        SampledSpectrum { values: v }
    }
}

impl Div for SampledSpectrum {
    type Output = Self;
    fn div(self, rhs: Self) -> Self {
        let mut v = [0.0; N_SPECTRUM_SAMPLES];
        for i in 0..N_SPECTRUM_SAMPLES { 
            // Avoid NaN propagation if dividing by zero (though rare in Fresnel)
            v[i] = if rhs.values[i] != 0.0 { self.values[i] / rhs.values[i] } else { 0.0 }; 
        }
        SampledSpectrum { values: v }
    }
}