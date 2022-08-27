use std::collections::HashMap;

pub mod celebi;
pub mod map;
pub mod point_provider;
pub mod point_provider_lab;
pub mod wsmeans;
pub mod wu;

pub use self::{
    celebi::QuantizerCelebi, map::QuantizerMap, wsmeans::QuantizerWsmeans, wu::QuantizerWu,
};

pub trait Quantizer {
    fn quantize(&mut self, pixels: &[u32], max_colors: u32) -> QuantizerResult;
}

pub struct QuantizerResult {
    pub color_to_count: HashMap<u32, u32>,
    pub input_pixel_to_cluster_pixel: HashMap<u32, u32>,
}
