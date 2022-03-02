pub mod cam16;
pub mod viewing_conditions;

pub use self::{cam16::Cam16, viewing_conditions::ViewingConditions};
use crate::utils::{
    color_utils::{argb_from_lstar, lstar_from_argb},
    math_utils::{clamp_double, sanitize_degrees_double},
};

/// When the delta between the floor & ceiling of a binary search for maximum chroma at a hue and
/// tone is less than this, the binary search terminates.
const CHROMA_SEARCH_ENDPOINT: f64 = 0.4;

/// The maximum color distance, in CAM16-UCS, between a requested color and the color returned.
const DE_MAX: f64 = 1.0;

/// The maximum difference between the requested L* and the L* returned.
const DL_MAX: f64 = 0.2;

/// The minimum color distance, in CAM16-UCS, between a requested color and an 'exact' match. This
/// allows the binary search during gamut mapping to terminate much earlier when the error is
/// infinitesimal.
const DE_MAX_ERROR: f64 = 0.000000001;

/// When the delta between the floor & ceiling of a binary search for J, lightness in CAM16, is
/// less than this, the binary search terminates.
const LIGHTNESS_SEARCH_ENDPOINT: f64 = 0.01;

/// HCT, hue, chroma, and tone. A color system that provides a perceptually
/// accurate color measurement system that can also accurately render what
/// colors will appear as in different lighting environments.
#[derive(Debug, PartialEq)]
pub struct Hct {
    pub hue: f64,
    pub chroma: f64,
    pub tone: f64,
}

impl Hct {
    /// 0 <= `hue` < 360; invalid values are corrected.
    /// 0 <= `chroma` <= ?; Informally, colorfulness. The color returned may be
    /// lower than the requested chroma. Chroma has a different maximum for any
    /// given hue and tone.
    /// 0 <= `tone` <= 100; informally, lightness. Invalid values are corrected.
    pub fn new(hue: f64, chroma: f64, tone: f64) -> Hct {
        let mut hct = Self {
            hue: sanitize_degrees_double(hue),
            chroma,
            tone: clamp_double(0.0, 100.0, tone),
        };
        hct.set_internal_state(hct.to_int());
        hct
    }

    pub fn to_int(&self) -> u32 {
        self.get_int(self.hue, self.chroma, self.tone)
    }

    fn set_internal_state(&mut self, argb: u32) {
        let cam = Cam16::from_int(argb);
        let tone = lstar_from_argb(argb);

        self.hue = cam.hue;
        self.chroma = cam.chroma;
        self.tone = tone;
    }

    /// HCT representation of `argb`.
    pub fn from_int(argb: u32) -> Hct {
        let cam = Cam16::from_int(argb);
        Hct::new(cam.hue, cam.chroma, lstar_from_argb(argb))
    }

    pub fn get_cam16(&self, hue: f64, chroma: f64, lstar: f64) -> Cam16 {
        self.get_cam16_in_viewing_conditions(hue, chroma, lstar, &ViewingConditions::default())
    }

    pub fn get_int(&self, hue: f64, chroma: f64, lstar: f64) -> u32 {
        self.get_int_in_viewing_conditions(hue, chroma, lstar, &ViewingConditions::default())
    }

    fn get_int_in_viewing_conditions(
        &self,
        hue: f64,
        chroma: f64,
        lstar: f64,
        viewing_conditions: &ViewingConditions,
    ) -> u32 {
        if chroma < 1.0 || lstar.round() <= 0.0 || lstar.round() >= 100.0 {
            return argb_from_lstar(lstar) as u32;
        }

        let hue = if hue < 0.0 {
            0.0
        } else if hue > 360.0 {
            360.0
        } else {
            hue
        };

        // Perform a binary search to find a chroma low enough that lstar is
        // possible. For example, a high chroma, high L* red isn't available.

        // The highest chroma possible. Updated as binary search proceeds.
        let mut high = chroma;

        // The guess for the current binary search iteration. Starts off at the highest chroma, thus,
        // if a color is possible at the requested chroma, the search can stop early.
        let mut mid = chroma;
        let mut low = 0.0;
        let mut is_first_loop = true;

        let mut answer: Option<Cam16> = None;

        while (low - high).abs() >= CHROMA_SEARCH_ENDPOINT {
            // Given the current chroma guess, mid, and the desired hue, find J, lightness in CAM16 color
            // space, that creates a color with L* = `lstar` in L*a*b*

            let possible_answer = self.find_cam_by_j(hue, mid, lstar, viewing_conditions);

            if is_first_loop {
                if let Some(possible_answer) = possible_answer {
                    return possible_answer.viewed(viewing_conditions);
                } else {
                    // If this binary search iteration was the first iteration, and this point has been reached,
                    // it means the requested chroma was not available at the requested hue and L*. Proceed to a
                    // traditional binary search, starting at the midpoint between the requested chroma and 0.

                    is_first_loop = false;
                    mid = low + (high - low) / 2.0;
                    continue;
                }
            }

            if possible_answer.is_none() {
                // There isn't a CAM16 J that creates a color with L*a*b* L*. Try a lower chroma.
                high = mid;
            } else {
                answer = possible_answer;
                // It is possible to create a color with L* `lstar` and `mid` chroma. Try a higher chroma.
                low = mid;
            }

            mid = low + (high - low) / 2.0;
        }

        // There was no answer: for the desired hue, there was no chroma low enough to generate a color
        // with the desired L*. All values of L* are possible when there is 0 chroma. Return a color
        // with 0 chroma, i.e. a shade of gray, with the desired L*.
        if let Some(answer) = answer {
            answer.viewed(viewing_conditions)
        } else {
            argb_from_lstar(lstar)
        }
    }

    fn get_cam16_in_viewing_conditions(
        &self,
        hue: f64,
        chroma: f64,
        lstar: f64,
        viewing_conditions: &ViewingConditions,
    ) -> Cam16 {
        if chroma < 1.0 || lstar.round() <= 0.0 || lstar.round() >= 100.0 {
            return Cam16::from_int(lstar_from_argb(lstar as u32) as u32);
        }

        let hue = if hue < 0.0 {
            0.0
        } else if hue > 360.0 {
            360.0
        } else {
            hue
        };

        // Perform a binary search to find a chroma low enough that lstar is
        // possible. For example, a high chroma, high L* red isn't available.

        // The highest chroma possible. Updated as binary search proceeds.
        let mut high = chroma;

        // The guess for the current binary search iteration. Starts off at the highest chroma, thus,
        // if a color is possible at the requested chroma, the search can stop early.
        let mut mid = chroma;
        let mut low = 0.0;
        let mut is_first_loop = true;

        let mut answer: Option<Cam16> = None;

        while (low - high).abs() >= CHROMA_SEARCH_ENDPOINT {
            // Given the current chroma guess, mid, and the desired hue, find J, lightness in CAM16 color
            // space, that creates a color with L* = `lstar` in L*a*b*
            let possible_answer = self.find_cam_by_j(hue, mid, lstar, viewing_conditions);

            if is_first_loop {
                if let Some(possible_answer) = possible_answer {
                    return possible_answer;
                } else {
                    // If this binary search iteration was the first iteration, and this point has been reached,
                    // it means the requested chroma was not available at the requested hue and L*. Proceed to a
                    // traditional binary search, starting at the midpoint between the requested chroma and 0.

                    is_first_loop = false;
                    mid = low + (high - low) / 2.0;
                    continue;
                }
            }

            if possible_answer.is_none() {
                // There isn't a CAM16 J that creates a color with L*a*b* L*. Try a lower chroma.
                high = mid;
            } else {
                answer = possible_answer;
                // It is possible to create a color with L* `lstar` and `mid` chroma. Try a higher chroma.
                low = mid;
            }

            mid = low + (high - low) / 2.0;
        }

        // There was no answer: for the desired hue, there was no chroma low enough to generate a color
        // with the desired L*. All values of L* are possible when there is 0 chroma. Return a color
        // with 0 chroma, i.e. a shade of gray, with the desired L*.
        if answer.is_none() {
            return Cam16::from_int(argb_from_lstar(lstar));
        }

        answer.unwrap()
    }

    fn find_cam_by_j(
        &self,
        hue: f64,
        chroma: f64,
        lstar: f64,
        frame: &ViewingConditions,
    ) -> Option<Cam16> {
        let mut low: f64 = 0.0;
        let mut high: f64 = 100.0;
        let mut best_d_l = std::f64::MAX;
        let mut best_d_e = std::f64::MAX;
        let mut best_cam = None;
        while (low - high).abs() > LIGHTNESS_SEARCH_ENDPOINT {
            let mid = low + (high - low) / 2.0;

            let cam_before_clip = Cam16::from_jch_in_viewing_conditions(mid, chroma, hue, frame);
            let clipped = cam_before_clip.viewed(frame);

            let clipped_lstar = lstar_from_argb(clipped);

            let d_l = (lstar - clipped_lstar).abs();

            if d_l < DL_MAX {
                let cam_clipped = Cam16::from_int_in_viewing_conditions(clipped, frame);

                let d_e = cam_clipped.distance(&Cam16::from_jch_in_viewing_conditions(
                    cam_clipped.j,
                    cam_clipped.chroma,
                    hue,
                    frame,
                ));

                if (d_e <= DE_MAX && d_e < best_d_e) && d_l < DL_MAX {
                    best_d_l = d_l;
                    best_d_e = d_e;
                    best_cam = Some(cam_clipped);
                }
            }

            if best_d_l == 0.0 && best_d_e < DE_MAX_ERROR {
                break;
            }

            if clipped_lstar < lstar {
                low = mid;
            } else {
                high = mid;
            }
        }

        best_cam
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
        assert_approx_eq!(46.445, cam.j, 0.001);
        assert_approx_eq!(113.357, cam.chroma, 0.001);
        assert_approx_eq!(27.408, cam.hue, 0.001);
        assert_approx_eq!(89.494, cam.m, 0.001);
        assert_approx_eq!(91.889, cam.s, 0.001);
        assert_approx_eq!(105.988, cam.q, 0.001);
    }

    #[test]
    fn cam_green() {
        let cam = Cam16::from_int(GREEN);
        assert_approx_eq!(79.331, cam.j, 0.001);
        assert_approx_eq!(108.410, cam.chroma, 0.001);
        assert_approx_eq!(142.139, cam.hue, 0.001);
        assert_approx_eq!(85.587, cam.m, 0.001);
        assert_approx_eq!(78.604, cam.s, 0.001);
        assert_approx_eq!(138.520, cam.q, 0.001);
    }

    #[test]
    fn cam_blue() {
        let cam = Cam16::from_int(BLUE);
        assert_approx_eq!(25.465, cam.j, 0.001);
        assert_approx_eq!(87.230, cam.chroma, 0.001);
        assert_approx_eq!(282.788, cam.hue, 0.001);
        assert_approx_eq!(68.867, cam.m, 0.001);
        assert_approx_eq!(93.674, cam.s, 0.001);
        assert_approx_eq!(78.481, cam.q, 0.001);
    }

    #[test]
    fn cam_black() {
        let cam = Cam16::from_int(BLACK);
        assert_approx_eq!(0.0, cam.j, 0.001);
        assert_approx_eq!(0.0, cam.chroma, 0.001);
        assert_approx_eq!(0.0, cam.hue, 0.001);
        assert_approx_eq!(0.0, cam.m, 0.001);
        assert_approx_eq!(0.0, cam.s, 0.001);
        assert_approx_eq!(0.0, cam.q, 0.001);
    }

    #[test]
    fn cam_white() {
        let cam = Cam16::from_int(WHITE);
        assert_approx_eq!(100.0, cam.j, 0.001);
        assert_approx_eq!(2.869, cam.chroma, 0.001);
        assert_approx_eq!(209.492, cam.hue, 0.001);
        assert_approx_eq!(2.265, cam.m, 0.001);
        assert_approx_eq!(12.068, cam.s, 0.001);
        assert_approx_eq!(155.521, cam.q, 0.001);
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
}
