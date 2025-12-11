mod core;
mod shapes;

use crate::core::geometry::{Vector3, Point3, Point2};
use crate::core::microfacet::TrowbridgeReitzDistribution;

fn main() {
    println!("--- Month 2 Week 5: Microfacet Theory Test ---");

    // 1. Create two distributions: Smooth (Shiny) vs Rough (Matte)
    let shiny = TrowbridgeReitzDistribution::new(0.1, 0.1);
    let matte = TrowbridgeReitzDistribution::new(0.8, 0.8);

    // 2. Test Direction: The "Half Vector" is perfectly aligned with Normal (Z-up)
    // This represents the peak of the specular highlight.
    let wh = Vector3::new(0.0, 0.0, 1.0);

    // 3. Evaluate NDF (D)
    let d_shiny = shiny.d(wh);
    let d_matte = matte.d(wh);

    println!("NDF Peak (Shiny, r=0.1): {:.2}", d_shiny);
    println!("NDF Peak (Matte, r=0.8): {:.2}", d_matte);

    // Expectation: Shiny should have a much higher peak than matte.
    if d_shiny > d_matte * 10.0 {
        println!("[SUCCESS] Shiny surface concentrates light much more than matte.");
    } else {
        println!("[FAIL] NDF values imply matte is sharper than shiny?");
    }

    // 4. Test Geometry Term (G)
    // Light is coming in at 45 degrees, View is at 45 degrees
    let w_45 = Vector3::new(0.707, 0.0, 0.707); // 45 deg
    let g_shiny = shiny.g(w_45, w_45);
    let g_matte = matte.g(w_45, w_45);

    println!("G (Shadowing) at 45deg (Shiny): {:.4}", g_shiny);
    println!("G (Shadowing) at 45deg (Matte): {:.4}", g_matte);

    // Expectation: Rougher surfaces self-shadow MORE, so G should be LOWER.
    if g_shiny > g_matte {
        println!("[SUCCESS] Matte surface has more self-shadowing (Lower G).");
    }
}