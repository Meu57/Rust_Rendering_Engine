use crate::core::texture::TextureMapping2D;
use crate::core::mipmap::MIPMap;
use crate::core::spectrum::SampledSpectrum;
use crate::core::interaction::SurfaceInteraction;
use std::sync::Arc;

pub trait Texture {
    fn evaluate(&self, si: &SurfaceInteraction) -> SampledSpectrum;
}

pub struct ImageTexture {
    mapping: Box<dyn TextureMapping2D>,
    mipmap: Arc<MIPMap>,
}

impl ImageTexture {
    pub fn new(mapping: Box<dyn TextureMapping2D>, filename: &str) -> Self {
        // Load Image using 'image' crate
        let img = image::open(filename).expect("Failed to load texture").to_rgb32f();
        let (width, height) = img.dimensions();

        // Convert pixels to Spectrum
        let mut texels = Vec::with_capacity((width * height) as usize);
        for y in 0..height {
            for x in 0..width {
                let pixel = img.get_pixel(x, y);
                // Note: We ignore wavelength upsampling for now (using RGB average)
                let rgb = [pixel[0], pixel[1], pixel[2]];
                texels.push(SampledSpectrum::from_rgb(rgb, &crate::core::spectrum::SampledWavelengths::sample_uniform(0.5)));
            }
        }

        let resolution = crate::core::geometry::Point2 { x: width as f32, y: height as f32 };
        let mipmap = Arc::new(MIPMap::new(resolution, texels));

        ImageTexture { mapping, mipmap }
    }
}

impl Texture for ImageTexture {
    fn evaluate(&self, si: &SurfaceInteraction) -> SampledSpectrum {
        // 1. Get (u, v) from the Mapping Strategy (Planar/Spherical/UV)
        let st = self.mapping.map(si);
        
        // 2. Lookup color in the MIP Map
        self.mipmap.lookup(st)
    }
}