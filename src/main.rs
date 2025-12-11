mod core;
mod shapes;

use crate::core::spectrum::{SampledSpectrum, SampledWavelengths};

fn main() {
    println!("--- Week 3 Days 3-5: Spectral Rendering Test ---");

    // 1. Generate Wavelengths for this "Ray"
    // In a real renderer, 'u' would be a random number per ray.
    let lambdas = SampledWavelengths::sample_uniform(0.5); 
    println!("Sampled Wavelengths: {:?}", lambdas.lambda);

    // 2. Input: Create an RGB Color (e.g., Bright Red)
    let input_rgb = [1.0, 0.0, 0.0];
    println!("Input RGB: {:?}", input_rgb);

    // 3. Upsample: Convert RGB -> Spectrum
    // (Currently using our placeholder 'Average' method)
    let spectrum = SampledSpectrum::from_rgb(input_rgb, &lambdas);
    println!("Spectrum Values (Energy): {:?}", spectrum.values);

    // 4. Downsample: Convert Spectrum -> XYZ -> RGB
    // This uses the analytic CIE curves we implemented.
    let xyz = spectrum.to_xyz(&lambdas);
    println!("Converted XYZ: {:?}", xyz);
    
    let final_rgb = SampledSpectrum::xyz_to_rgb(xyz);
    println!("Final RGB: {:?}", final_rgb);

    println!("\n[NOTE] The 'Final RGB' will not match 'Input RGB' perfectly yet.");
    println!("       This is because we are using a placeholder 'Average' upsampler.");
    println!("       In Month 4, we will implement the Sigmoid Polynomial for exact matching.");
}