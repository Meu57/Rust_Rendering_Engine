use crate::core::geometry::{Point2, Point2i, Vector3, Point3};
use crate::core::camera::PerspectiveCamera;
use crate::core::primitive::Primitive;
use crate::core::sampler::StratifiedSampler;
use crate::core::film::Film;
use crate::core::bsdf::{BSDF, BxDF, ThinDielectricBxDF}; 
use crate::core::bssrdf::{BSSRDF, SeparableBSSRDF};      
use crate::core::spectrum::{SampledSpectrum, SampledWavelengths};
use std::f32::consts::PI;

pub fn render(
    scene: &dyn Primitive,
    camera: &PerspectiveCamera,
    film: &mut Film,
) {
    let mut sampler = StratifiedSampler::new(2, 2); 
    let spp = sampler.samples_per_pixel() as f32;

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
                    
                    let bssrdf = SeparableBSSRDF::new_skin(1.4);
                    let wo = -ray.d;
                    let n_vec = Vector3::from(interaction.core.n);

                    // S_omega Exit
                    let cos_theta_o = n_vec.dot(wo).abs();
                    let s_omega_exit = bssrdf.eval_directional(cos_theta_o);

                    // Sample Probe Ray
                    let u_dist = sampler.get_2d().x; 
                    let r = u_dist * 0.05; 
                    
                    // Diffusion Term
                    let sp = bssrdf.eval_spatial(r);

                    // Lighting (S_omega Entry)
                    let light_dir = Vector3::new(0.0, 0.0, 1.0).normalize(); 
                    let cos_theta_i = n_vec.dot(light_dir).max(0.0);
                    let s_omega_entry = bssrdf.eval_directional(cos_theta_i);

                    // Combine
                    let throughput = sp * s_omega_exit * s_omega_entry;
                    
                    // FIX: Lower boost from 2000.0 to 50.0 to prevent white clipping
                    let display_spectrum = throughput * 50.0; 

                    let wavelengths = SampledWavelengths::sample_uniform(0.5);
                    let rgb = SampledSpectrum::xyz_to_rgb(display_spectrum.to_xyz(&wavelengths));
                    
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