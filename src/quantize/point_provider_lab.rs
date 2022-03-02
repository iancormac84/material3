use crate::utils::color_utils;

use super::point_provider::PointProvider;

pub struct PointProviderLab;

impl PointProvider for PointProviderLab {
    fn from_int(&self, argb: u32) -> [f64; 3] {
        color_utils::lab_from_argb(argb)
    }
    fn to_int(&self, lab: &[f64]) -> u32 {
        color_utils::argb_from_lab(lab[0], lab[1], lab[2])
    }
    fn distance(&self, one: &[f64], two: &[f64]) -> f64 {
        let d_l = one[0] - two[0];
        let d_a = one[1] - two[1];
        let d_b = one[2] - two[2];
        // Standard CIE 1976 delta E formula also takes the square root, unneeded
        // here. This method is used by quantization algorithms to compare distance,
        // and the relative ordering is the same, with or without a square root.

        // This relatively minor optimization is helpful because this method is
        // called at least once for each pixel in an image.
        d_l * d_l + d_a * d_a + d_b * d_b
    }
}
