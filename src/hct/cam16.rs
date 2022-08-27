use super::viewing_conditions::ViewingConditions;
use crate::utils::{
    color_utils::{argb_from_xyz, xyz_from_argb},
    math_utils::signum,
};

#[derive(Debug, PartialEq, Default, Clone, Copy)]
pub struct Cam16 {
    /// Like red, orange, yellow, green, etc.
    pub hue: f64,
    /// Informally, colorfulness / color intensity. Like saturation in HSL,
    /// except perceptually accurate.
    pub chroma: f64,
    /// Lightness
    pub j: f64,
    /// Brightness; ratio of lightness to white point's lightness
    pub q: f64,
    /// Colorfulness
    pub m: f64,
    /// Saturation; ratio of chroma to white point's chroma
    pub s: f64,
    /// CAM16-UCS J coordinate
    pub jstar: f64,
    /// CAM16-UCS a coordinate
    pub astar: f64,
    /// CAM16-UCS b coordinate
    pub bstar: f64,
}

impl Cam16 {
    /// CAM16 instances also have coordinates in the CAM16-UCS space, called J*,
    /// a*, b*, or jstar, astar, bstar in code. CAM16-UCS is included in the CAM16
    /// specification, and should be used when measuring distances between colors.
    pub fn distance(&self, other: &Cam16) -> f64 {
        let dj = self.jstar - other.jstar;

        let da = self.astar - other.astar;

        let db = self.bstar - other.bstar;

        let de_prime = (dj * dj + da * da + db * db).sqrt();

        1.41 * de_prime.powf(0.63)
    }

    pub fn from_int(argb: u32) -> Cam16 {
        Cam16::from_int_in_viewing_conditions(argb, &ViewingConditions::default())
    }

    pub fn from_int_in_viewing_conditions(
        argb: u32,
        viewing_conditions: &ViewingConditions,
    ) -> Cam16 {
        // Transform ARGB int to XYZ
        let xyz = xyz_from_argb(argb);
        let x = xyz[0];
        let y = xyz[1];
        let z = xyz[2];

        // Transform XYZ to 'cone'/'rgb' responses

        let r_c = 0.401288 * x + 0.650173 * y - 0.051461 * z;
        let g_c = -0.250268 * x + 1.204414 * y + 0.045854 * z;
        let b_c = -0.002079 * x + 0.048952 * y + 0.953127 * z;

        // Discount illuminant
        let r_d = viewing_conditions.rgb_d[0] * r_c;
        let g_d = viewing_conditions.rgb_d[1] * g_c;
        let b_d = viewing_conditions.rgb_d[2] * b_c;

        // chromatic adaptation
        let r_a_f = (viewing_conditions.fl * r_d.abs() / 100.0).powf(0.42);
        let g_a_f = (viewing_conditions.fl * g_d.abs() / 100.0).powf(0.42);
        let b_a_f = (viewing_conditions.fl * b_d.abs() / 100.0).powf(0.42);
        let r_a = signum(r_d) as f64 * 400.0 * r_a_f / (r_a_f + 27.13);
        let g_a = signum(g_d) as f64 * 400.0 * g_a_f / (g_a_f + 27.13);
        let b_a = signum(b_d) as f64 * 400.0 * b_a_f / (b_a_f + 27.13);

        // redness-greenness
        let a = (11.0 * r_a + -12.0 * g_a + b_a) / 11.0;
        // yellowness-blueness
        let b = (r_a + g_a - 2.0 * b_a) / 9.0;

        // auxiliary components
        let u = (20.0 * r_a + 20.0 * g_a + 21.0 * b_a) / 20.0;
        let p2 = (40.0 * r_a + 20.0 * g_a + b_a) / 20.0;

        // hue
        let atan2 = b.atan2(a);
        let atan_degrees = atan2 * 180.0 / std::f64::consts::PI;
        let hue = if atan_degrees < 0.0 {
            atan_degrees + 360.0
        } else if atan_degrees >= 360.0 {
            atan_degrees - 360.0
        } else {
            atan_degrees
        };
        let hue_radians = hue * std::f64::consts::PI / 180.0;

        assert!((0.0..360.0).contains(&hue), "hue was really {}", hue);

        // achromatic response to color
        let ac = p2 * viewing_conditions.nbb;

        // CAM16 lightness and brightness
        let big_j =
            100.0 * (ac / viewing_conditions.aw).powf(viewing_conditions.c * viewing_conditions.z);

        let big_q = (4.0 / viewing_conditions.c)
            * (big_j / 100.0).sqrt()
            * (viewing_conditions.aw + 4.0)
            * (viewing_conditions.f_l_root);

        let hue_prime = if hue < 20.14 { hue + 360.0 } else { hue };
        let e_hue = 0.25 * ((hue_prime * std::f64::consts::PI / 180.0 + 2.0).cos() + 3.8);
        let p1 = 50000.0 / 13.0 * e_hue * viewing_conditions.nc * viewing_conditions.ncb;
        let t = p1 * (a * a + b * b).sqrt() / (u + 0.305);
        let alpha = {
            let tpow09 = t.powf(0.9);
            let rss = 0.29f64.powf(viewing_conditions.background_y_to_white_point_y);
            let rss = 1.64 - rss;
            let rss_pow = rss.powf(0.73);
            tpow09 * rss_pow
        };

        // CAM16 chroma, colorfulness, chroma
        let big_c = alpha * (big_j / 100.0).sqrt();

        let big_m = big_c * viewing_conditions.f_l_root;
        let s = 50.0 * ((alpha * viewing_conditions.c) / (viewing_conditions.aw + 4.0)).sqrt();

        // CAM16-UCS components
        let jstar = (1.0 + 100.0 * 0.007) * big_j / (1.0 + 0.007 * big_j);
        let mstar = 1.0 / 0.0228 * (1.0 + 0.0228 * big_m).log(std::f64::consts::E);

        let astar = mstar * hue_radians.cos();
        let bstar = mstar * hue_radians.sin();

        Cam16 {
            hue,
            chroma: big_c,
            j: big_j,
            q: big_q,
            m: big_m,
            s,
            jstar,
            astar,
            bstar,
        }
    }

    /// Create a CAM16 color from lightness `j`, chroma `c`, and hue `h`,
    /// assuming the color was viewed in default viewing conditions.
    pub fn from_jch(j: f64, c: f64, h: f64) -> Cam16 {
        Cam16::from_jch_in_viewing_conditions(j, c, h, &ViewingConditions::default())
    }

    pub fn from_jch_in_viewing_conditions(
        big_j: f64,
        big_c: f64,
        h: f64,
        viewing_conditions: &ViewingConditions,
    ) -> Cam16 {
        let big_q = (4.0 / viewing_conditions.c)
            * (big_j / 100.0).sqrt()
            * (viewing_conditions.aw + 4.0)
            * (viewing_conditions.f_l_root);

        let big_m = big_c * viewing_conditions.f_l_root;
        let alpha = big_c / (big_j / 100.0).sqrt();
        let s = 50.0 * ((alpha * viewing_conditions.c) / (viewing_conditions.aw + 4.0)).sqrt();

        let hue_radians = h * std::f64::consts::PI / 180.0;

        let jstar = (1.0 + 100.0 * 0.007) * big_j / (1.0 + 0.007 * big_j);
        let mstar = 1.0 / 0.0228 * (1.0 + 0.0228 * big_m).log(std::f64::consts::E);

        let astar = mstar * hue_radians.cos();
        let bstar = mstar * hue_radians.sin();

        Cam16 {
            hue: h,
            chroma: big_c,
            j: big_j,
            q: big_q,
            m: big_m,
            s,
            jstar,
            astar,
            bstar,
        }
    }

    /// Create a CAM16 color from CAM16-UCS coordinates `jstar`, `astar`, `bstar`
    /// assuming the color was viewed in default viewing conditions.
    pub fn from_ucs(jstar: f64, astar: f64, bstar: f64) -> Cam16 {
        Cam16::from_ucs_in_viewing_conditions(jstar, astar, bstar, &ViewingConditions::default())
    }

    /// Create a CAM16 color from CAM16-UCS coordinates `jstar`, `astar`, `bstar`
    /// in [`ViewingConditions`].
    pub fn from_ucs_in_viewing_conditions(
        jstar: f64,
        astar: f64,
        bstar: f64,
        viewing_conditions: &ViewingConditions,
    ) -> Cam16 {
        let a = astar;
        let b = bstar;
        let m = (a * a + b * b).sqrt();
        let big_m = ((m * 0.0228).exp() - 1.0) / 0.0228;
        let c = big_m / viewing_conditions.f_l_root;
        let mut h = b.atan2(a) * (180.0 / std::f64::consts::PI);
        if h < 0.0 {
            h += 360.0;
        }
        let j = jstar / (1.0 - (jstar - 100.0) * 0.007);

        Cam16::from_jch_in_viewing_conditions(j, c, h, viewing_conditions)
    }

    pub fn viewed_in_srgb(&self) -> u32 {
        self.viewed(&ViewingConditions::default())
    }

    /// ARGB representation of a color, given the color was viewed in
    /// [`ViewingConditions`]
    pub fn viewed(&self, viewing_conditions: &ViewingConditions) -> u32 {
        let alpha = if self.chroma == 0.0 || self.j == 0.0 {
            0.0
        } else {
            self.chroma / (self.j / 100.0).sqrt()
        };

        let t = {
            let bkpow = 0.29f64.powf(viewing_conditions.background_y_to_white_point_y);
            let bkpow = 1.64 - bkpow;
            let bkpow_pow = bkpow.powf(0.73);
            let bkpow_pow = alpha / bkpow_pow;
            bkpow_pow.powf(1.0 / 0.9)
        };

        let h_rad = self.hue * std::f64::consts::PI / 180.0;

        let e_hue = 0.25 * ((h_rad + 2.0).cos() + 3.8);
        let ac = viewing_conditions.aw
            * (self.j / 100.0).powf(1.0 / viewing_conditions.c / viewing_conditions.z);
        let p1 = e_hue * (50000.0 / 13.0) * viewing_conditions.nc * viewing_conditions.ncb;

        let p2 = ac / viewing_conditions.nbb;

        let h_sin = h_rad.sin();
        let h_cos = h_rad.cos();

        let gamma = 23.0 * (p2 + 0.305) * t / (23.0 * p1 + 11.0 * t * h_cos + 108.0 * t * h_sin);
        let a = gamma * h_cos;
        let b = gamma * h_sin;
        let r_a = (460.0 * p2 + 451.0 * a + 288.0 * b) / 1403.0;
        let g_a = (460.0 * p2 - 891.0 * a - 261.0 * b) / 1403.0;
        let b_a = (460.0 * p2 - 220.0 * a - 6300.0 * b) / 1403.0;

        let r_c_base = 0.0f64.max((27.13 * r_a.abs()) / (400.0 - r_a.abs()));
        let r_c = signum(r_a) as f64 * (100.0 / viewing_conditions.fl) * r_c_base.powf(1.0 / 0.42);
        let g_c_base = 0f64.max((27.13 * g_a.abs()) / (400.0 - g_a.abs()));
        let g_c = signum(g_a) as f64 * (100.0 / viewing_conditions.fl) * g_c_base.powf(1.0 / 0.42);
        let b_c_base = 0f64.max((27.13 * b_a.abs()) / (400.0 - b_a.abs()));
        let b_c = signum(b_a) as f64 * (100.0 / viewing_conditions.fl) * b_c_base.powf(1.0 / 0.42);
        let r_f = r_c / viewing_conditions.rgb_d[0];
        let g_f = g_c / viewing_conditions.rgb_d[1];
        let b_f = b_c / viewing_conditions.rgb_d[2];

        let x = 1.86206786 * r_f - 1.01125463 * g_f + 0.14918677 * b_f;
        let y = 0.38752654 * r_f + 0.62144744 * g_f - 0.00897398 * b_f;
        let z = -0.01584150 * r_f - 0.03412294 * g_f + 1.04996444 * b_f;

        argb_from_xyz(x, y, z)
    }
}
