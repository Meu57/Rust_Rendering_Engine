use crate::core::geometry::{Point2, Point2i, Vector3};
use crate::core::camera::PerspectiveCamera;
use crate::core::primitive::Primitive;
use crate::core::sampler::StratifiedSampler;
use crate::core::film::Film;
use crate::core::spectrum::{SampledSpectrum, SampledWavelengths};

pub fn render(
    scene: &dyn Primitive,
    camera: &PerspectiveCamera,
    film: &mut Film,
) {
    let mut sampler = StratifiedSampler::new(8, 8); 
    let spp = sampler.samples_per_pixel() as f32;
    let max_depth = 5; 

    println!("Rendering {}x{} image (Path Tracing)...", film.resolution.x, film.resolution.y);

    for y in 0..film.resolution.y {
        for x in 0..film.resolution.x {
            let pixel = Point2i { x, y };
            sampler.start_pixel(pixel);
            
            let mut pixel_color = Vector3 { x: 0.0, y: 0.0, z: 0.0 };
            
            for _ in 0..sampler.samples_per_pixel() {
                let offset = sampler.get_2d();
                let raster_sample = Point2 { x: x as f32 + offset.x, y: y as f32 + offset.y };

                let mut ray = camera.generate_ray(
                    raster_sample, 
                    crate::core::geometry::Point2 { x: film.resolution.x as f32, y: film.resolution.y as f32 },
                    90.0
                );

                #[allow(non_snake_case)]
                let mut L = SampledSpectrum::new(0.0); 
                let mut beta = SampledSpectrum::new(1.0); 
                let wavelengths = SampledWavelengths::sample_uniform(sampler.get_2d().x);

                for _bounces in 0..max_depth {
                    // FIX: intersect now returns the material too
                    let hit = scene.intersect(&ray);
                    
                    if let Some((_, interaction, material_opt)) = hit {
                        
                        if let Some(mat) = material_opt {
                            // 1. Add Emission (Le)
                            #[allow(non_snake_case)]
                            let Le = mat.emitted(&interaction);
                            L = L + beta * Le;

                            // 2. Compute Scattering
                            if let Some(bsdf) = mat.compute_scattering(&interaction) {
                                let u_sample = sampler.get_2d();
                                let wo = -ray.d;
                                
                                if let Some((f, wi, pdf)) = bsdf.sample_f(wo, u_sample) {
                                    if pdf == 0.0 { break; } 

                                    let n_vec = Vector3::from(interaction.shading.n);
                                    let cos_theta = wi.dot(n_vec).abs();
                                    
                                    beta = beta * f * (cos_theta / pdf);
                                    ray = interaction.core.spawn_ray(wi);
                                } else {
                                    break; // Absorbed
                                }
                            } else {
                                break; // Pure light source (no scatter)
                            }
                        } else {
                            break; // No material
                        }
                    } else {
                        break; // Miss
                    }
                }

                let rgb = SampledSpectrum::xyz_to_rgb(L.to_xyz(&wavelengths));
                pixel_color = pixel_color + Vector3 { x: rgb[0], y: rgb[1], z: rgb[2] };
            }

            film.set_pixel(pixel, pixel_color * (1.0 / spp));
        }
        if y % 10 == 0 { print!("."); use std::io::Write; std::io::stdout().flush().unwrap(); }
    }
    println!("\nDone!");
}