mod core;
mod shapes;

use crate::core::spectrum::{SampledSpectrum, SampledWavelengths, BlackbodySpectrum};

fn main() {
    println!("--- Week 3 Final: Blackbody Physics Test ---");

    // 1. Create a 5000K Blackbody (Warm Daylight / Horizon Sun)
    let temp = 5000.0;
    println!("Creating Light Source at {} Kelvin...", temp);
    let bb = BlackbodySpectrum::new(temp);

    // 2. Sample across the visible spectrum (Blue to Red)
    // We manually pick wavelengths to inspect the curve
    let test_lambdas = SampledWavelengths { 
        lambda: [450.0, 550.0, 650.0, 700.0], // Blue, Green, Red, Deep Red
        pdf: [1.0; 4] 
    };

    let spectrum = bb.sample(&test_lambdas);
    
    println!("Wavelengths (nm): {:?}", test_lambdas.lambda);
    println!("Normalized Intensity: {:?}", spectrum.values);

    // 3. Logic Check: 5000K should be "Warm" (Red > Blue)
    // At 5000K, the peak is around 580nm (Yellowish).
    // So 650nm (Red) should be fairly high, and 450nm (Blue) should be lower.
    let blue_intensity = spectrum.values[0]; // 450nm
    let red_intensity = spectrum.values[2];  // 650nm

    if red_intensity > blue_intensity {
        println!("[SUCCESS] Physics holds: 5000K is warm (Red > Blue).");
    } else {
        println!("[FAIL] Physics broken: 5000K should be warm.");
    }

    // 4. Test Extreme Heat: 10,000K (Blue Star)
    let hot_bb = BlackbodySpectrum::new(10_000.0);
    let hot_spec = hot_bb.sample(&test_lambdas);
    
    if hot_spec.values[0] > hot_spec.values[2] {
        println!("[SUCCESS] Physics holds: 10,000K is cool (Blue > Red).");
    }
}