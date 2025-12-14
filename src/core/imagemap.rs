use crate::core::texture::{Texture, TextureMapping2D}; // Import Trait
use crate::core::mipmap::MIPMap;
use crate::core::spectrum::SampledSpectrum;
use crate::core::interaction::SurfaceInteraction;
use std::sync::Arc;

pub struct ImageTexture {
    mapping: Box<dyn TextureMapping2D>,
    mipmap: Arc<MIPMap>,
}

impl ImageTexture {
    pub fn new(mapping: Box<dyn TextureMapping2D>, filename: &str) -> Self {
        let img = image::open(filename).expect("Failed to load texture").to_rgb32f();
        let (width, height) = img.dimensions();

        let mut texels = Vec::with_capacity((width * height) as usize);
        for y in 0..height {
            for x in 0..width {
                let pixel = img.get_pixel(x, y);
                let rgb = [pixel[0], pixel[1], pixel[2]];
                texels.push(SampledSpectrum::from_rgb(rgb, &crate::core::spectrum::SampledWavelengths::sample_uniform(0.5)));
            }
        }

        let resolution = crate::core::geometry::Point2 { x: width as f32, y: height as f32 };
        let mipmap = Arc::new(MIPMap::new(resolution, texels));

        ImageTexture { mapping, mipmap }
    }
}

// Implement the Trait (defined in texture.rs)
impl Texture for ImageTexture {
    fn evaluate(&self, si: &SurfaceInteraction) -> SampledSpectrum {
        let st = self.mapping.map(si);
        self.mipmap.lookup(st)
    }
}