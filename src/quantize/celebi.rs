use super::{
    point_provider_lab::PointProviderLab, wsmeans::QuantizerWsmeans, wu::QuantizerWu, Quantizer,
    QuantizerResult,
};

pub struct QuantizerCelebi;
impl Quantizer for QuantizerCelebi {
    fn quantize(&mut self, pixels: &[u32], max_colors: u32) -> QuantizerResult {
        let mut wu = QuantizerWu::new();
        let wu_result = wu.quantize(pixels, max_colors);
        let mut wsmeans = QuantizerWsmeans {
            debug: true,
            starting_clusters: wu_result.color_to_count.keys().copied().collect(),
            point_provider: PointProviderLab,
            max_iterations: 5,
            return_input_pixel_to_cluster_pixel: false,
        };
        wsmeans.quantize(pixels, max_colors)
    }
}

#[cfg(test)]
mod test {
    use indexmap::IndexSet;

    use crate::quantize::{celebi::QuantizerCelebi, Quantizer};

    const RED: u32 = 0xffff0000;
    const GREEN: u32 = 0xff00ff00;
    const BLUE: u32 = 0xff0000ff;
    const MAX_COLORS: u32 = 256;

    #[test]
    fn one_red() {
        let mut celebi = QuantizerCelebi;
        let result = celebi.quantize(&vec![RED], MAX_COLORS);
        let colors: Vec<u32> = result.color_to_count.keys().copied().collect();
        assert_eq!(colors.len(), 1);
        assert_eq!(colors[0], RED);
    }

    #[test]
    fn one_green() {
        let mut celebi = QuantizerCelebi;
        let result = celebi.quantize(&vec![GREEN], MAX_COLORS);
        let colors: Vec<u32> = result.color_to_count.keys().copied().collect();
        assert_eq!(colors.len(), 1);
        assert_eq!(colors[0], GREEN);
    }

    #[test]
    fn one_blue() {
        let mut celebi = QuantizerCelebi;
        let result = celebi.quantize(&vec![BLUE], MAX_COLORS);
        let colors: Vec<u32> = result.color_to_count.keys().copied().collect();
        assert_eq!(colors.len(), 1);
        assert_eq!(colors[0], BLUE);
    }

    #[test]
    fn five_blue() {
        let mut celebi = QuantizerCelebi;
        let result = celebi.quantize(&vec![BLUE, BLUE, BLUE, BLUE, BLUE], MAX_COLORS);
        let colors: Vec<u32> = result.color_to_count.keys().copied().collect();
        assert_eq!(colors.len(), 1);
        assert_eq!(colors[0], BLUE);
    }

    #[test]
    fn one_red_one_green_one_blue() {
        let mut celebi = QuantizerCelebi;
        let result = celebi.quantize(&vec![RED, GREEN, BLUE], MAX_COLORS);
        let colors: Vec<u32> = result.color_to_count.keys().copied().collect();

        let mut set = IndexSet::new();
        for color in &colors {
            set.insert(*color);
        }

        assert_eq!(set.len(), 3);
        assert_eq!(colors[0], BLUE);
        assert_eq!(colors[1], RED);
        assert_eq!(colors[2], GREEN);
    }

    #[test]
    fn two_red_three_green() {
        let mut celebi = QuantizerCelebi;
        let result = celebi.quantize(&vec![RED, RED, GREEN, GREEN, GREEN], MAX_COLORS);
        let colors: Vec<u32> = result.color_to_count.keys().copied().collect();

        let mut set = IndexSet::new();
        for color in &colors {
            set.insert(*color);
        }

        assert_eq!(colors[0], GREEN);
        assert_eq!(colors[1], RED);
    }
}
