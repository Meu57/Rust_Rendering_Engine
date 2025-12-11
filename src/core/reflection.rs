use crate::core::spectrum::SampledSpectrum;

// --- Helper: Clamp ---
fn clamp(val: f32, min: f32, max: f32) -> f32 {
    if val < min { min } else if val > max { max } else { val }
}

// --- 1. Dielectric (Glass/Water) Fresnel ---
// Returns the fraction of light that is REFLECTED.
// The rest (1.0 - Fr) is TRANSMITTED (refracted).
pub fn fr_dielectric(cos_theta_i: f32, eta_i: f32, eta_t: f32) -> f32 {
    let mut cos_theta_i = clamp(cos_theta_i, -1.0, 1.0);
    
    // Adjust indices based on whether we are entering or exiting
    let mut eta_i_eff = eta_i;
    let mut eta_t_eff = eta_t;
    
    if cos_theta_i < 0.0 {
        // Exiting the medium (Glass -> Air)
        std::mem::swap(&mut eta_i_eff, &mut eta_t_eff);
        cos_theta_i = cos_theta_i.abs();
    }

    // Snell's Law: n1 * sin1 = n2 * sin2
    let sin_theta_i = (1.0 - cos_theta_i * cos_theta_i).max(0.0).sqrt();
    let sin_theta_t = (eta_i_eff / eta_t_eff) * sin_theta_i;

    // --- CRITICAL EDGE CASE: Total Internal Reflection (TIR) ---
    // If sin(theta_t) > 1.0, light cannot escape. 100% Reflection.
    if sin_theta_t >= 1.0 {
        return 1.0; 
    }

    let cos_theta_t = (1.0 - sin_theta_t * sin_theta_t).max(0.0).sqrt();

    // Fresnel Equations for Parallel and Perpendicular polarization
    let r_parl = ((eta_t_eff * cos_theta_i) - (eta_i_eff * cos_theta_t)) /
                 ((eta_t_eff * cos_theta_i) + (eta_i_eff * cos_theta_t));
                 
    let r_perp = ((eta_i_eff * cos_theta_i) - (eta_t_eff * cos_theta_t)) /
                 ((eta_i_eff * cos_theta_i) + (eta_t_eff * cos_theta_t));

    // Unpolarized light is the average of the squares
    (r_parl * r_parl + r_perp * r_perp) / 2.0
}

// --- 2. Conductor (Metal) Fresnel ---
// Metals effectively do not transmit light. They reflect or absorb.
// Because absorption varies by wavelength (Gold reflects Red, absorbs Blue),
// we must compute this for the whole Spectrum.
pub fn fr_conductor(cos_theta_i: f32, eta: SampledSpectrum, k: SampledSpectrum) -> SampledSpectrum {
    let cos_theta_i = clamp(cos_theta_i.abs(), 0.0, 1.0);
    let cos_theta_i2 = cos_theta_i * cos_theta_i;
    let sin_theta_i2 = 1.0 - cos_theta_i2;

    let eta2 = eta * eta;
    let k2 = k * k;

    // Complex number expansion |a+bi|^2 logic
    let t0 = eta2 + k2; // (n^2 + k^2)
    let t1 = eta * (2.0 * cos_theta_i); 

    let rs = (t0 - t1 + SampledSpectrum::splat(cos_theta_i2)) / 
             (t0 + t1 + SampledSpectrum::splat(cos_theta_i2));

    let t2 = t0 * cos_theta_i2 + SampledSpectrum::splat(sin_theta_i2);
    let t3 = eta * (2.0 * cos_theta_i * sin_theta_i2); 
    
    // Approximation commonly used in graphics (PBRT v3 style simplified)
    // For exact complex fresnel, we can use the PBRT v4 full expansion, 
    // but this captures the color shift correctly.
    let rp = rs * (t2 - t3 + SampledSpectrum::splat(sin_theta_i2)) / 
                  (t2 + t3 + SampledSpectrum::splat(sin_theta_i2));

    (rs + rp) * 0.5
}