use crate::core::geometry::{Point2, Point2i, Vector3};
use crate::core::camera::PerspectiveCamera;
use crate::core::primitive::Primitive;
use crate::core::sampler::StratifiedSampler;
use crate::core::film::Film;
use crate::core::spectrum::{SampledSpectrum, SampledWavelengths};
use crate::core::light::Light;

pub fn render(
    scene: &dyn Primitive,
    lights: &Vec<Box<dyn Light>>,
    camera: &PerspectiveCamera,
    film: &mut Film,
) {
    let mut sampler = StratifiedSampler::new(8, 8);
    let spp = sampler.samples_per_pixel() as f32;
    let max_depth = 5;

    println!(
        "Rendering {}x{} image (Direct Lighting + NEE)...",
        film.resolution.x, film.resolution.y
    );

    for y in 0..film.resolution.y {
        for x in 0..film.resolution.x {
            let pixel = Point2i { x, y };
            sampler.start_pixel(pixel);

            let mut pixel_color = Vector3 { x: 0.0, y: 0.0, z: 0.0 };

            for _ in 0..sampler.samples_per_pixel() {
                let offset = sampler.get_2d();
                let raster_sample = Point2 {
                    x: x as f32 + offset.x,
                    y: y as f32 + offset.y,
                };

                let mut ray = camera.generate_ray(
                    raster_sample,
                    Point2 {
                        x: film.resolution.x as f32,
                        y: film.resolution.y as f32,
                    },
                    90.0,
                );

                let wavelengths =
                    SampledWavelengths::sample_uniform(sampler.get_2d().x);

                let mut l = SampledSpectrum::new(0.0);
                let mut beta = SampledSpectrum::new(1.0);

                for _bounce in 0..max_depth {
                    let hit = scene.intersect(&ray);

                    let Some((_, interaction, material_opt)) = hit else {
                        break;
                    };

                    let Some(mat) = material_opt else {
                        break;
                    };

                    let Some(bsdf) = mat.compute_scattering(&interaction) else {
                        break;
                    };

                    // --------------------------------------------------
                    // NEXT EVENT ESTIMATION (Explicit Light Sampling)
                    // --------------------------------------------------
                    for light in lights {
                        let u_light = sampler.get_2d();

                        if let Some(ls) = light.sample_li(&interaction, u_light) {
                            let shadow_ray =
                                interaction.core.spawn_ray(ls.wi);

                            let light_dist =
                                (ls.p_light - interaction.core.p).length();

                            let occluded =
                                if let Some((t_occ, _, _)) =
                                    scene.intersect(&shadow_ray)
                                {
                                    t_occ < light_dist - 1e-3
                                } else {
                                    false
                                };

                            if !occluded && ls.pdf > 0.0 {
                                let n =
                                    Vector3::from(interaction.shading.n);
                                let cos_theta =
                                    ls.wi.dot(n).max(0.0);

                                let f = bsdf.f(-ray.d, ls.wi);

                                l = l + beta * f * ls.l * (cos_theta / ls.pdf);
                            }
                        }
                    }

                    // --------------------------------------------------
                    // Indirect Bounce (no emission accumulation here)
                    // --------------------------------------------------
                    let u_bsdf = sampler.get_2d();
                    let wo = -ray.d;

                    if let Some((f, wi, pdf)) =
                        bsdf.sample_f(wo, u_bsdf)
                    {
                        if pdf == 0.0 {
                            break;
                        }

                        let n =
                            Vector3::from(interaction.shading.n);
                        let cos_theta = wi.dot(n).abs();

                        beta = beta * f * (cos_theta / pdf);
                        ray = interaction.core.spawn_ray(wi);
                    } else {
                        break;
                    }
                }

                let rgb =
                    SampledSpectrum::xyz_to_rgb(l.to_xyz(&wavelengths));

                pixel_color = pixel_color
                    + Vector3 {
                        x: rgb[0],
                        y: rgb[1],
                        z: rgb[2],
                    };
            }

            film.set_pixel(pixel, pixel_color * (1.0 / spp));
        }

        if y % 10 == 0 {
            print!(".");
            use std::io::Write;
            std::io::stdout().flush().unwrap();
        }
    }

    println!("\nDone!");
}
