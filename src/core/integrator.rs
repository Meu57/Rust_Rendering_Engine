use crate::core::geometry::{Point2, Point2i, Vector3};
use crate::core::camera::PerspectiveCamera;
use crate::core::primitive::Primitive;
use crate::core::sampler::StratifiedSampler;
use crate::core::film::Film;
use crate::core::bsdf::{BSDF, BxDF, ThinDielectricBxDF};
use crate::core::spectrum::{SampledSpectrum, SampledWavelengths};

pub fn render(
    scene: &dyn Primitive,
    camera: &PerspectiveCamera,
    film: &mut Film,
) {
    let mut sampler = StratifiedSampler::new(2, 2); // 4 samples per pixel
    let spp = sampler.samples_per_pixel() as f32;

    println!("Rendering {}x{} image...", film.resolution.x, film.resolution.y);

    for y in 0..film.resolution.y {
        for x in 0..film.resolution.x {
            let pixel = Point2i { x, y };
            sampler.start_pixel(pixel);
            
            let mut pixel_color = Vector3 { x: 0.0, y: 0.0, z: 0.0 };

            for _ in 0..sampler.samples_per_pixel() {
                // 1. Get sub-pixel offset
                let offset = sampler.get_2d();
                
                // 2. Map to Raster Space (Pixel + Offset)
                let raster_sample = Point2 { 
                    x: x as f32 + offset.x, 
                    y: y as f32 + offset.y 
                };

                // 3. Generate Ray
                let ray = camera.generate_ray(
                    raster_sample, 
                    crate::core::geometry::Point2 { 
                        x: film.resolution.x as f32, 
                        y: film.resolution.y as f32 
                    },
                    90.0
                );

                // 4. Intersect (Li - Radiance)
                let color = if let Some((_, interaction)) = scene.intersect(&ray) {

                    // ---------------------------------------------------------
                    // ✅ WEEK 7: Thin-Film Soap Bubble BSDF
                    // ---------------------------------------------------------

                    // Hardcoded thin-film parameters
                    let bxdf = BxDF::ThinDielectric(ThinDielectricBxDF::new(
                        1.33,   // IOR of soap film
                        400.0,  // thickness in nm
                    ));

                    // FIX: Convert Normal3 to Vector3 explicitly
                    let bsdf = BSDF::new(Vector3::from(interaction.core.n), bxdf);

                    // Light coming from the camera direction
                    let wo = -ray.d;

                    // Dummy sample for now
                    let u_sample = Point2 { x: 0.0, y: 0.5 };

                    if let Some((f, _wi, _pdf)) = bsdf.sample_f(wo, u_sample) {
                        // Convert spectral reflectance to RGB
                        let wavelengths = SampledWavelengths::sample_uniform(0.5);
                        let rgb = SampledSpectrum::xyz_to_rgb(f.to_xyz(&wavelengths));

                        Vector3 { x: rgb[0], y: rgb[1], z: rgb[2] }
                    } else {
                        Vector3 { x: 0.0, y: 0.0, z: 0.0 }
                    }

                    // ---------------------------------------------------------

                } else {
                    Vector3 { x: 0.0, y: 0.0, z: 0.0 } // Black background
                };

                pixel_color = pixel_color + color;
            }

            // Average samples
            film.set_pixel(pixel, pixel_color * (1.0 / spp));
        }

        // Simple progress bar
        if y % 10 == 0 { 
            print!("."); 
            use std::io::Write; 
            std::io::stdout().flush().unwrap(); 
        }
    }

    println!("\nDone!");
}