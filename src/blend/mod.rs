use crate::{
    hct::{Cam16, Hct},
    utils::{color_utils, math_utils},
};

/// Functions for blending in HCT and CAM16.

/// Shifts `design_color`'s hue towards `source_color`'s, creating a slightly
/// warmer/coolor variant of `design_color`. Hue will shift up to 15 degrees.
pub fn harmonize(design_color: u32, source_color: u32) -> u32 {
    let from_hct = Hct::from_int(design_color);
    let to_hct = Hct::from_int(source_color);
    let difference_degrees = math_utils::calculate_difference_degrees(from_hct.hue, to_hct.hue);
    let rotation_degrees = (difference_degrees * 0.5).min(15.0);
    let output_hue = math_utils::sanitize_degrees_double(
        from_hct.hue + rotation_degrees * math_utils::rotation_direction(from_hct.hue, to_hct.hue),
    );
    Hct::new(output_hue, from_hct.chroma, from_hct.tone).to_int()
}

/// Blends `from`'s hue in HCT towards `to`'s hue.
pub fn hct_hue(from: u32, to: u32, amount: f64) -> u32 {
    let ucs = cam16_ucs(from, to, amount);
    let ucs_cam = Cam16::from_int(ucs);
    let from_cam = Cam16::from_int(from);
    Hct::new(
        ucs_cam.hue,
        from_cam.chroma,
        color_utils::lstar_from_argb(from),
    )
    .to_int()
}

/// Blend `from` and `to` in the CAM16-UCS color space.
pub fn cam16_ucs(from: u32, to: u32, amount: f64) -> u32 {
    let from_cam = Cam16::from_int(from);
    let to_cam = Cam16::from_int(to);

    let from_j_star = from_cam.jstar;
    let from_a_star = from_cam.astar;
    let from_b_star = from_cam.bstar;

    let to_j_star = to_cam.jstar;
    let to_a_star = to_cam.astar;
    let to_b_star = to_cam.bstar;

    let jstar = from_j_star + (to_j_star - from_j_star) * amount;
    let astar = from_a_star + (to_a_star - from_a_star) * amount;
    let bstar = from_b_star + (to_b_star - from_b_star) * amount;

    Cam16::from_ucs(jstar, astar, bstar).viewed_in_srgb()
}

#[cfg(test)]
mod test {
    use crate::blend::harmonize;

    const RED: u32 = 0xffff0000;
    const BLUE: u32 = 0xff0000ff;
    const GREEN: u32 = 0xff00ff00;
    const YELLOW: u32 = 0xffffff00;

    #[test]
    fn red_to_blue() {
        let answer = harmonize(RED, BLUE);
        assert_eq!(answer, 0xffFB0057);
    }

    #[test]
    fn red_to_green() {
        let answer = harmonize(RED, GREEN);
        assert_eq!(answer, 0xffD85600);
    }

    #[test]
    fn red_to_yellow() {
        let answer = harmonize(RED, YELLOW);
        assert_eq!(answer, 0xffD85600);
    }

    #[test]
    fn blue_to_green() {
        let answer = harmonize(BLUE, GREEN);
        assert_eq!(answer, 0xff0047A3);
    }

    #[test]
    fn blue_to_red() {
        let answer = harmonize(BLUE, RED);
        assert_eq!(answer, 0xff5700DC);
    }

    #[test]
    fn blue_to_yellow() {
        let answer = harmonize(BLUE, YELLOW);
        assert_eq!(answer, 0xff0047A3);
    }

    #[test]
    fn green_to_blue() {
        let answer = harmonize(GREEN, BLUE);
        assert_eq!(answer, 0xff00FC94);
    }

    #[test]
    fn green_to_red() {
        let answer = harmonize(GREEN, RED);
        assert_eq!(answer, 0xffB1F000);
    }

    #[test]
    fn green_to_yellow() {
        let answer = harmonize(GREEN, YELLOW);
        assert_eq!(answer, 0xffB1F000);
    }

    #[test]
    fn yellow_to_blue() {
        let answer = harmonize(YELLOW, BLUE);
        assert_eq!(answer, 0xffEBFFBA);
    }

    #[test]
    fn yellow_to_green() {
        let answer = harmonize(YELLOW, GREEN);
        assert_eq!(answer, 0xffEBFFBA);
    }

    #[test]
    fn yellow_to_red() {
        let answer = harmonize(YELLOW, RED);
        assert_eq!(answer, 0xffFFF6E3);
    }
}
