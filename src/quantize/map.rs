use indexmap::IndexMap;

use crate::utils::color_utils;

use super::QuantizerResult;

pub struct QuantizerMap;

impl super::Quantizer for QuantizerMap {
    fn quantize(&mut self, pixels: &[u32], _max_colors: u32) -> QuantizerResult {
        let mut count_by_color = IndexMap::new();
        for pixel in pixels {
            let alpha = color_utils::alpha_from_argb(*pixel);

            if alpha < 255 {
                continue;
            }

            *count_by_color.entry(*pixel).or_insert(0) += 1;
        }
        QuantizerResult {
            color_to_count: count_by_color,
            input_pixel_to_cluster_pixel: IndexMap::new(),
        }
    }
}
