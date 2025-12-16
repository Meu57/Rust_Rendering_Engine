use crate::core::geometry::{Point2, Point2i, Vector3};
use crate::core::camera::PerspectiveCamera;
use crate::core::primitive::Primitive;
use crate::core::sampler::StratifiedSampler;
use crate::core::film::Film;
use crate::core::spectrum::{SampledSpectrum, SampledWavelengths};
use crate::core::light::Light;

// Power heuristic for MIS weighting (p^2 / (p^2 + q^2))
fn power_heuristic(nf: i32, f_pdf: f32, ng: i32, g_pdf: f32) -> f32 {
    let f = (nf as f32) * f_pdf;
    let g = (ng as f32) * g_pdf;
    let ff = f * f;
    let gg = g * g;
    if ff + gg == 0.0 { 0.0 } else { ff / (ff + gg) }
}

/// Full path tracer with NEE + MIS + robust delta light handling.
/// Assumptions:
/// - Light::sample_li returns a direction wi, radiance Li and pdf in *solid angle*.
///   If some lights return area pdfs, uncomment the area→solid-angle conversion below.
/// - bsdf.sample_f returns (f, wi, pdf, is_delta), pdf in solid angle.
/// - Emission (Le) is added only for camera ray or after specular bounce.
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
        "Rendering {}x{} image (Full Path Tracing with MIS)...",
        film.resolution.x, film.resolution.y
    );

    for y in 0..film.resolution.y {
        for x in 0..film.resolution.x {
            let pixel = Point2i { x, y };
            sampler.start_pixel(pixel);

            let mut pixel_color = Vector3 { x: 0.0, y: 0.0, z: 0.0 };

            for _s in 0..sampler.samples_per_pixel() {
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
                    90.0, // Adapt if your camera stores FOV
                );

                let wavelengths = SampledWavelengths::sample_uniform(sampler.get_2d().x);
                let mut l = SampledSpectrum::new(0.0);
                let mut beta = SampledSpectrum::new(1.0);
                let mut specular_bounce = false;

                for bounces in 0..max_depth {
                    let hit = scene.intersect(&ray);

                    // Escaped scene -> environment contribution would go here if you have one
                    let Some((_, interaction, material_opt)) = hit else {
                        // e.g. l += beta * env_le(ray.d, wavelengths);
                        break;
                    };

                    // Surface emission (Le)
                    if let Some(mat) = &material_opt {
                        let le = mat.emitted(&interaction);
                        if le.values.iter().any(|&v| v > 0.0) {
                            // Only for primary rays or specular paths (avoid double counting with NEE)
                            if bounces == 0 || specular_bounce {
                                l = l + beta * le;
                            }
                        }
                    }

                    // No material: terminate
                    let Some(mat) = material_opt else { break; };

                    // Build BSDF
                    let Some(bsdf) = mat.compute_scattering(&interaction) else {
                        break; // absorbed / invalid
                    };

                    // === Next Event Estimation: sample one light with MIS (robust) ===
                    if !lights.is_empty() {
                        let n_lights = lights.len();
                        let light_choice_f = sampler.get_2d().x * n_lights as f32;
                        let light_idx = light_choice_f.floor() as usize;
                        let light_idx = light_idx.min(n_lights - 1);
                        let light = &lights[light_idx];
                        let pdf_light_choice = 1.0 / (n_lights as f32);

                        let u_light = sampler.get_2d();
                        if let Some(ls) = light.sample_li(&interaction, u_light) {
                            // If Light::sample_li returns area pdf, convert here.
                            // For now we assume ls.pdf is already in solid angle:
                            let mut ls_pdf_solid = ls.pdf;

                            // Uncomment and adapt if some lights use area measure:
                            // let light_dist = (ls.p_light - interaction.core.p).length();
                            // let cos_at_light = ls.n_light.dot(-ls.wi).max(0.0);
                            // if cos_at_light > 1e-7 {
                            //     ls_pdf_solid = ls.pdf * (light_dist * light_dist) / cos_at_light;
                            // } else {
                            //     ls_pdf_solid = 0.0;
                            // }

                            let li_nonzero =
                                !ls.l.values.iter().all(|&v| v == 0.0);

                            // Delta lights: pdf may be zero but they must still contribute
                            if light.is_delta() {
                                if li_nonzero {
                                    let shadow_ray = interaction.core.spawn_ray(ls.wi);
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

                                    if !occluded {
                                        let wo = -ray.d;
                                        let f = bsdf.f(wo, ls.wi);
                                        if !f.values.iter().all(|&v| v == 0.0) {
                                            let n_vec =
                                                Vector3::from(interaction.shading.n);
                                            let cos_theta =
                                                n_vec.dot(ls.wi).max(0.0);
                                            if cos_theta > 0.0 {
                                                // No MIS competition for delta lights
                                                l = l + beta * f * ls.l * cos_theta;
                                            }
                                        }
                                    }
                                }
                            } else {
                                // Non-delta lights: standard MIS
                                if ls_pdf_solid > 0.0 && li_nonzero {
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

                                    if !occluded {
                                        let wo = -ray.d;
                                        let f = bsdf.f(wo, ls.wi);
                                        if !f.values.iter().all(|&v| v == 0.0) {
                                            let n_vec =
                                                Vector3::from(interaction.shading.n);
                                            let cos_theta =
                                                n_vec.dot(ls.wi).max(0.0);
                                            if cos_theta > 0.0 {
                                                let pdf_light =
                                                    ls_pdf_solid * pdf_light_choice;
                                                let pdf_bsdf = bsdf.pdf(wo, ls.wi);

                                                let weight_light =
                                                    power_heuristic(
                                                        1,
                                                        pdf_light,
                                                        1,
                                                        pdf_bsdf,
                                                    );

                                                if pdf_light > 0.0 {
                                                    l = l
                                                        + beta
                                                            * f
                                                            * ls.l
                                                            * (cos_theta / pdf_light)
                                                            * weight_light;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // === BSDF sampling for indirect lighting ===
                    let u_bsdf = sampler.get_2d();
                    let wo = -ray.d;

                    // bsdf.sample_f: (f, wi, pdf, is_delta)
                    if let Some((f, wi, pdf, is_delta)) = bsdf.sample_f(wo, u_bsdf) {
                        if pdf == 0.0
                            || f.values.iter().all(|&v| v == 0.0)
                        {
                            break;
                        }

                        let n_vec = Vector3::from(interaction.shading.n);
                        let cos_theta = wi.dot(n_vec).max(0.0);
                        if cos_theta == 0.0 {
                            break;
                        }

                        // Throughput update
                        beta = beta * f * (cos_theta / pdf);

                        // Russian roulette
                        if bounces > 3 {
                            let max_component =
                                beta.values.iter().fold(0.0f32, |a, &b| a.max(b));
                            let q = (1.0 - max_component)
                                .max(0.05)
                                .min(0.95);
                            if sampler.get_2d().x < q {
                                break;
                            }
                            beta = beta * (1.0 / (1.0 - q));
                        }

                        // Next ray
                        ray = interaction.core.spawn_ray(wi);
                        specular_bounce = is_delta;
                    } else {
                        break;
                    }
                }

                let rgb = SampledSpectrum::xyz_to_rgb(l.to_xyz(&wavelengths));
                pixel_color = pixel_color + Vector3 {
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
