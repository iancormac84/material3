use crate::utils::{color_utils::y_from_lstar, math_utils::lerp};

/// In traditional color spaces, a color can be identified solely by the
/// observer's measurement of the color. Color appearance models such as CAM16
/// also use information about the environment where the color was
/// observed, known as the viewing conditions.
///
/// For example, white under the traditional assumption of a midday sun white
/// point is accurately measured as a slightly chromatic blue by CAM16.
/// (roughly, hue 203, chroma 3, lightness 100)
///
/// This class caches intermediate values of the CAM16 conversion process that
/// depend only on viewing conditions, enabling speed ups.
#[derive(Debug)]
pub struct ViewingConditions {
    pub white_point: [f64; 3],
    pub adapting_luminance: f64,
    pub background_lstar: f64,
    pub surround: f64,
    pub discounting_illuminant: bool,

    pub background_y_to_white_point_y: f64,
    pub aw: f64,
    pub nbb: f64,
    pub ncb: f64,
    pub c: f64,
    pub nc: f64,
    pub drgb_inverse: [f64; 3],
    pub rgb_d: [f64; 3],
    pub fl: f64,
    pub f_l_root: f64,
    pub z: f64,
}

impl Default for ViewingConditions {
    fn default() -> ViewingConditions {
        ViewingConditions::new(
            [95.047, 100.0, 108.883],
            200.0 / std::f64::consts::PI * y_from_lstar(50.0) / 100.0,
            50.0,
            2.0,
            false,
        )
    }
}
impl ViewingConditions {
    pub fn new(
        white_point: [f64; 3],
        adapting_luminance: f64,
        background_lstar: f64,
        surround: f64,
        discounting_illuminant: bool,
    ) -> ViewingConditions {
        let adapting_luminance = if adapting_luminance > 0.0 {
            adapting_luminance
        } else {
            200.0 / std::f64::consts::PI * y_from_lstar(50.0) / 100.0
        };

        let background_lstar = 30f64.max(background_lstar);

        let r_w =
            white_point[0] * 0.401288 + white_point[1] * 0.650173 + white_point[2] * -0.051461;
        let g_w =
            white_point[0] * -0.250268 + white_point[1] * 1.204414 + white_point[2] * 0.045854;
        let b_w =
            white_point[0] * -0.002079 + white_point[1] * 0.048952 + white_point[2] * 0.953127;

        // Scale input surround, domain (0, 2), to CAM16 surround, domain (0.8, 1.0)
        assert!((0.0..=2.0).contains(&surround));

        let f = 0.8 + (surround / 10.0);

        let c = if f >= 0.9 {
            lerp(0.59, 0.69, (f - 0.9) * 10.0)
        } else {
            lerp(0.525, 0.59, (f - 0.8) * 10.0)
        };

        let mut d = if discounting_illuminant {
            1.0
        } else {
            f * (1.0 - ((1.0 / 3.6) * ((-adapting_luminance - 42.0) / 92.0).exp()))
        };
        d = if d > 1.0 {
            1.0
        } else if d < 0.0 {
            0.0
        } else {
            d
        };
        let nc = f;

        let rgb_d = [
            d * (100.0 / r_w) + 1.0 - d,
            d * (100.0 / g_w) + 1.0 - d,
            d * (100.0 / b_w) + 1.0 - d,
        ];

        // Factor used in calculating meaningful factors
        let k = 1.0 / (5.0 * adapting_luminance + 1.0);
        let k4 = k * k * k * k;
        let k4_f = 1.0 - k4;

        // Luminance-level adaptation factor
        let fl = (k4 * adapting_luminance)
            + (0.1 * k4_f * k4_f * (5.0 * adapting_luminance).powf(1.0 / 3.0));
        // Intermediate factor, ratio of background relative luminance to white relative luminance
        let n = y_from_lstar(background_lstar) / white_point[1];

        // Base exponential nonlinearity
        // note Schlomer 2018 has a typo and uses 1.58, the correct factor is 1.48
        let z = 1.48 + n.sqrt();

        // Luminance-level induction factors
        let nbb = 0.725 / n.powf(0.2);
        let ncb = nbb;

        // Discounted cone responses to the white point, adjusted for post-saturationtic
        // adaptation perceptual nonlinearities.
        let rgb_a_factors = [
            (fl * rgb_d[0] * r_w / 100.0).powf(0.42),
            (fl * rgb_d[1] * g_w / 100.0).powf(0.42),
            (fl * rgb_d[2] * b_w / 100.0).powf(0.42),
        ];

        let rgb_a = [
            (400.0 * rgb_a_factors[0]) / (rgb_a_factors[0] + 27.13),
            (400.0 * rgb_a_factors[1]) / (rgb_a_factors[1] + 27.13),
            (400.0 * rgb_a_factors[2]) / (rgb_a_factors[2] + 27.13),
        ];

        let aw = (40.0 * rgb_a[0] + 20.0 * rgb_a[1] + rgb_a[2]) / 20.0 * nbb;

        ViewingConditions {
            white_point,
            adapting_luminance,
            background_lstar,
            surround,
            discounting_illuminant,
            background_y_to_white_point_y: n,
            aw,
            nbb,
            ncb,
            c,
            nc,
            drgb_inverse: [0.0, 0.0, 0.0],
            rgb_d,
            fl,
            f_l_root: fl.powf(0.25),
            z,
        }
    }
}
