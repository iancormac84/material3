pub mod cam16;
pub mod cam_solver;
pub mod viewing_conditions;

pub use self::{cam16::Cam16, cam_solver::solve_to_int, viewing_conditions::ViewingConditions};
use crate::utils::color_utils::lstar_from_argb;

/// HCT, hue, chroma, and tone. A color system that provides a perceptually
/// accurate color measurement system that can also accurately render what
/// colors will appear as in different lighting environments.
#[derive(Debug, PartialEq)]
pub struct Hct {
    pub hue: f64,
    pub chroma: f64,
    pub tone: f64,
    argb: u32,
}

impl Hct {
    /// 0 <= `hue` < 360; invalid values are corrected.
    /// 0 <= `chroma` <= ?; Informally, colorfulness. The color returned may be
    /// lower than the requested chroma. Chroma has a different maximum for any
    /// given hue and tone.
    /// 0 <= `tone` <= 100; informally, lightness. Invalid values are corrected.
    pub fn new(hue: f64, chroma: f64, tone: f64) -> Hct {
        Hct::_new(solve_to_int(hue, chroma, tone))
    }

    pub fn to_int(&self) -> u32 {
        self.argb
    }

    fn _new(argb: u32) -> Hct {
        let cam = Cam16::from_int(argb);
        Hct {
            hue: cam.hue,
            chroma: cam.chroma,
            tone: lstar_from_argb(argb),
            argb,
        }
    }

    /// HCT representation of `argb`.
    pub fn from_int(argb: u32) -> Hct {
        Hct::_new(argb)
    }
}

#[cfg(test)]
mod test {
    use crate::utils::color_utils::{lstar_from_argb, y_from_lstar};
    use approx_eq::assert_approx_eq;

    use super::{cam16::Cam16, viewing_conditions::ViewingConditions, Hct};

    const BLACK: u32 = 0xff000000;
    const WHITE: u32 = 0xffffffff;
    const RED: u32 = 0xffff0000;
    const GREEN: u32 = 0xff00ff00;
    const BLUE: u32 = 0xff0000ff;

    #[test]
    fn conversions_are_reflexive() {
        let cam = Cam16::from_int(RED);
        let color = cam.viewed(&ViewingConditions::default());
        assert_eq!(color, RED);
    }

    #[test]
    fn y_midgray() {
        assert_approx_eq!(18.418, y_from_lstar(50.0), 0.001);
    }

    #[test]
    fn y_black() {
        assert_approx_eq!(0.0, y_from_lstar(0.0), 0.001);
    }

    #[test]
    fn y_white() {
        assert_approx_eq!(100.0, y_from_lstar(100.0), 0.001);
    }

    #[test]
    fn cam_red() {
        let cam = Cam16::from_int(RED);
        assert_approx_eq!(46.445, cam.j, 3.0);
        assert_approx_eq!(113.357, cam.chroma, 3.0);
        assert_approx_eq!(27.408, cam.hue, 3.0);
        assert_approx_eq!(89.494, cam.m, 3.0);
        assert_approx_eq!(91.890, cam.s, 3.0);
        assert_approx_eq!(105.980, cam.q, 3.0);
    }

    #[test]
    fn cam_green() {
        let cam = Cam16::from_int(GREEN);
        assert_approx_eq!(79.332, cam.j, 3.0);
        assert_approx_eq!(108.410, cam.chroma, 3.0);
        assert_approx_eq!(142.140, cam.hue, 3.0);
        assert_approx_eq!(85.588, cam.m, 3.0);
        assert_approx_eq!(78.605, cam.s, 3.0);
        assert_approx_eq!(138.520, cam.q, 3.0);
    }

    #[test]
    fn cam_blue() {
        let cam = Cam16::from_int(BLUE);
        assert_approx_eq!(25.466, cam.j, 3.0);
        assert_approx_eq!(87.231, cam.chroma, 3.0);
        assert_approx_eq!(282.788, cam.hue, 3.0);
        assert_approx_eq!(68.867, cam.m, 3.0);
        assert_approx_eq!(93.675, cam.s, 3.0);
        assert_approx_eq!(78.481, cam.q, 3.0);
    }

    #[test]
    fn cam_black() {
        let cam = Cam16::from_int(BLACK);
        assert_approx_eq!(0.0, cam.j, 3.0);
        assert_approx_eq!(0.0, cam.chroma, 3.0);
        assert_approx_eq!(0.0, cam.hue, 3.0);
        assert_approx_eq!(0.0, cam.m, 3.0);
        assert_approx_eq!(0.0, cam.s, 3.0);
        assert_approx_eq!(0.0, cam.q, 3.0);
    }

    #[test]
    fn cam_white() {
        let cam = Cam16::from_int(WHITE);
        assert_approx_eq!(100.0, cam.j, 3.0);
        assert_approx_eq!(2.869, cam.chroma, 3.0);
        assert_approx_eq!(209.492, cam.hue, 3.0);
        assert_approx_eq!(2.265, cam.m, 3.0);
        assert_approx_eq!(12.068, cam.s, 3.0);
        assert_approx_eq!(155.521, cam.q, 3.0);
    }

    #[test]
    fn gamut_map_red() {
        let color_to_test = RED;
        let cam = Cam16::from_int(color_to_test);
        let color = Hct::new(cam.hue, cam.chroma, lstar_from_argb(color_to_test)).to_int();
        assert_eq!(color_to_test, color);
    }

    #[test]
    fn gamut_map_green() {
        let color_to_test = GREEN;
        let cam = Cam16::from_int(color_to_test);
        let color = Hct::new(cam.hue, cam.chroma, lstar_from_argb(color_to_test)).to_int();
        assert_eq!(color_to_test, color);
    }

    #[test]
    fn gamut_map_blue() {
        let color_to_test = BLUE;
        let cam = Cam16::from_int(color_to_test);
        let color = Hct::new(cam.hue, cam.chroma, lstar_from_argb(color_to_test)).to_int();
        assert_eq!(color_to_test, color);
    }

    #[test]
    fn gamut_map_white() {
        let color_to_test = WHITE;
        let cam = Cam16::from_int(color_to_test);
        let color = Hct::new(cam.hue, cam.chroma, lstar_from_argb(color_to_test)).to_int();
        assert_eq!(color_to_test, color);
    }

    #[test]
    fn gamut_map_midgray() {
        let color_to_test = GREEN;
        let cam = Cam16::from_int(color_to_test);
        let color = Hct::new(cam.hue, cam.chroma, lstar_from_argb(color_to_test)).to_int();
        assert_eq!(color_to_test, color);
    }

    #[test]
    fn hct_returns_a_sufficiently_close_color() {
        for hue in (15..360).step_by(30) {
            for chroma in (0..=100).step_by(10) {
                for tone in (20..=80).step_by(10) {
                    let hct_request_description = format!("H{} C{} T{}", hue, chroma, tone);
                    println!("{hct_request_description}");
                    let hct_color = Hct::new(hue as f64, chroma as f64, tone as f64);
                    if chroma as f64 > 0.0 {
                        assert_approx_eq!(hct_color.hue, hue as f64, 4.0);
                    }
                    assert!(
                        hct_color.chroma >= 0.0 && hct_color.chroma <= chroma as f64 + 2.5,
                        "Chroma should be close or less for {}",
                        hct_request_description
                    );
                    assert_approx_eq!(hct_color.tone, tone as f64, 0.5);
                }
            }
        }
    }

    #[test]
    fn hct_preserves_original_color() {
        for argb in 0xFF000000..=0xFFFFFFFF {
            let hct = Hct::from_int(argb);
            let reconstructed_argb = Hct::new(hct.hue, hct.chroma, hct.tone).to_int();

            assert_eq!(reconstructed_argb, argb);
        }
    }
}
