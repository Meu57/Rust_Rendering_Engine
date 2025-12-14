use crate::core::geometry::{Point2, Point2i, Vector3};
use crate::core::camera::PerspectiveCamera;
use crate::core::primitive::Primitive;
use crate::core::sampler::StratifiedSampler;
use crate::core::film::Film;
use crate::core::spectrum::{SampledSpectrum, SampledWavelengths};
use crate::core::texture::{Texture, NoiseTexture}; // <--- Import Noise

pub fn render(
    scene: &dyn Primitive,
    camera: &PerspectiveCamera,
    film: &mut Film,
) {
    let mut sampler = StratifiedSampler::new(2, 2); 
    let spp = sampler.samples_per_pixel() as f32;

    // Create the Procedural Texture
    // Scale = 4.0 means the noise pattern repeats ~4 times across the unit space
    let noise_tex = NoiseTexture::new(4.0);

    println!("Rendering {}x{} image...", film.resolution.x, film.resolution.y);

    for y in 0..film.resolution.y {
        for x in 0..film.resolution.x {
            let pixel = Point2i { x, y };
            sampler.start_pixel(pixel);
            
            let mut pixel_color = Vector3 { x: 0.0, y: 0.0, z: 0.0 };

            for _ in 0..sampler.samples_per_pixel() {
                let offset = sampler.get_2d();
                let raster_sample = Point2 { x: x as f32 + offset.x, y: y as f32 + offset.y };

                let ray = camera.generate_ray(
                    raster_sample, 
                    crate::core::geometry::Point2 { x: film.resolution.x as f32, y: film.resolution.y as f32 },
                    90.0
                );

                let color = if let Some((_, interaction)) = scene.intersect(&ray) {
                    
                    // --- Week 8: Noise Test ---
                    // Evaluate the procedural texture at the hit point
                    let spectrum = noise_tex.evaluate(&interaction);
                    
                    let wavelengths = SampledWavelengths::sample_uniform(0.5);
                    let rgb = SampledSpectrum::xyz_to_rgb(spectrum.to_xyz(&wavelengths));
                    
                    Vector3 { x: rgb[0], y: rgb[1], z: rgb[2] }

                } else {
                    Vector3 { x: 0.0, y: 0.0, z: 0.0 } 
                };

                pixel_color = pixel_color + color;
            }

            film.set_pixel(pixel, pixel_color * (1.0 / spp));
        }
        if y % 10 == 0 { print!("."); use std::io::Write; std::io::stdout().flush().unwrap(); }
    }
    println!("\nDone!");
}