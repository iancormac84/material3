use super::math_utils::{clamp_int, matrix_multiply};

/// Converts an L* value to a Y value.
///
/// L* in L*a*b* and Y in XYZ measure the same quantity, luminance.
/// L* measures perceptual luminance, a linear scale. Y in XYZ
/// measures relative luminance, a logarithmic scale.
///
/// `lstar` L* in L*a*b*
///
/// Returns Y in XYZ
pub fn y_from_lstar(lstar: f64) -> f64 {
    100.0 * lab_inv_f((lstar + 16.0) / 116.0)
}

/// Returns the alpha component of a color in ARGB format.
pub fn alpha_from_argb(argb: u32) -> u32 {
    argb >> 24 & 255
}

/// Returns the red component of a color in ARGB format.
pub fn red_from_argb(argb: u32) -> u32 {
    (argb >> 16) & 255
}

/// Returns the green component of a color in ARGB format.
pub fn green_from_argb(argb: u32) -> u32 {
    (argb >> 8) & 255
}

/// Returns the blue component of a color in ARGB format.
pub fn blue_from_argb(argb: u32) -> u32 {
    argb & 255
}

/// Converts a color from RGB components to ARGB format.
pub fn argb_from_rgb(red: u32, green: u32, blue: u32) -> u32 {
    255 << 24 | (red & 255) << 16 | (green & 255) << 8 | blue & 255
}

/// Converts a color from linear RGB components to ARGB format.
pub fn argb_from_linrgb(linrgb: [f64; 3]) -> u32 {
    let r = delinearized(linrgb[0]);
    let g = delinearized(linrgb[1]);
    let b = delinearized(linrgb[2]);
    argb_from_rgb(r, g, b)
}

/// Returns the XYZ to sRGB transformation matrix.
pub const XYZ_TO_SRGB: [[f64; 3]; 3] = [
    [
        3.2413774792388685,
        -1.5376652402851851,
        -0.49885366846268053,
    ],
    [-0.9691452513005321, 1.8758853451067872, 0.04156585616912061],
    [
        0.05562093689691305,
        -0.20395524564742123,
        1.0571799111220335,
    ],
];

/// Converts a color from ARGB to XYZ.
pub fn xyz_from_argb(argb: u32) -> [f64; 3] {
    let r = linearized(red_from_argb(argb));
    let g = linearized(green_from_argb(argb));
    let b = linearized(blue_from_argb(argb));

    matrix_multiply([r, g, b], SRGB_TO_XYZ)
}

/// Converts a color from XYZ to ARGB.
pub fn argb_from_xyz(x: f64, y: f64, z: f64) -> u32 {
    let linear_r = XYZ_TO_SRGB[0][0] * x + XYZ_TO_SRGB[0][1] * y + XYZ_TO_SRGB[0][2] * z;
    let linear_g = XYZ_TO_SRGB[1][0] * x + XYZ_TO_SRGB[1][1] * y + XYZ_TO_SRGB[1][2] * z;
    let linear_b = XYZ_TO_SRGB[2][0] * x + XYZ_TO_SRGB[2][1] * y + XYZ_TO_SRGB[2][2] * z;
    let r = delinearized(linear_r);
    let g = delinearized(linear_g);
    let b = delinearized(linear_b);
    argb_from_rgb(r, g, b)
}

/// Linearizes an RGB component.
///
///
/// `rgb_component` 0 <= rgb_component <= 255, represents R/G/B
/// channel
///
///
/// Returns 0.0 <= output <= 100.0, color channel converted to
/// linear RGB space
pub fn linearized(rgb_component: u32) -> f64 {
    let normalized = rgb_component as f64 / 255.0;
    if normalized <= 0.040449936 {
        normalized / 12.92 * 100.0
    } else {
        ((normalized + 0.055) / 1.055).powf(2.4) * 100.0
    }
}

/// Delinearizes an RGB component.
///
///
/// `rgb_component` 0.0 <= rgb_component <= 100.0, represents linear
/// R/G/B channel
/// Returns 0 <= output <= 255, color channel converted to regular
/// RGB space
pub fn delinearized(rgb_component: f64) -> u32 {
    let normalized = rgb_component / 100.0;
    let delinearized = if normalized <= 0.0031308 {
        normalized * 12.92
    } else {
        1.055 * normalized.powf(1.0 / 2.4) - 0.055
    };
    clamp_int(0, 255, (delinearized * 255.0).round() as u32)
}

/// Returns the sRGB to XYZ transformation matrix.
pub const SRGB_TO_XYZ: [[f64; 3]; 3] = [
    [0.41233895, 0.35762064, 0.18051042],
    [0.2126, 0.7152, 0.0722],
    [0.01932141, 0.11916382, 0.95034478],
];

/// Converts an L* value to an ARGB representation.
///
///
/// `lstar` L* in L*a*b*
/// Returns ARGB representation of grayscale color with lightness
/// matching L*
pub fn argb_from_lstar(lstar: f64) -> u32 {
    let y = y_from_lstar(lstar);
    let component = delinearized(y);
    argb_from_rgb(component, component, component)
}

/// Computes the L* value of a color in ARGB representation.
///
///
/// `argb` ARGB representation of a color
/// Returns L*, from L*a*b*, coordinate of the color
pub fn lstar_from_argb(argb: u32) -> f64 {
    let y = xyz_from_argb(argb)[1];
    116.0 * lab_f(y / 100.0) - 16.0
}

/// Returns the standard white point; white on a sunny day.
///
///
/// Returns The white point
pub const WHITE_POINT_D65: [f64; 3] = [95.047, 100.0, 108.883];

/// Converts a color represented in Lab color space into an ARGB
/// integer.
pub fn argb_from_lab(l: f64, a: f64, b: f64) -> u32 {
    let fy = (l + 16.0) / 116.0;
    let fx = a / 500.0 + fy;
    let fz = fy - b / 200.0;
    let x_normalized = lab_inv_f(fx);
    let y_normalized = lab_inv_f(fy);
    let z_normalized = lab_inv_f(fz);
    let x = x_normalized * WHITE_POINT_D65[0];
    let y = y_normalized * WHITE_POINT_D65[1];
    let z = z_normalized * WHITE_POINT_D65[2];
    argb_from_xyz(x, y, z)
}

/// Converts a color from ARGB representation to L*a*b*
/// representation.
///
///
/// `argb` the ARGB representation of a color
/// Returns a Lab object representing the color
pub fn lab_from_argb(argb: u32) -> [f64; 3] {
    let linear_r = linearized(red_from_argb(argb));
    let linear_g = linearized(green_from_argb(argb));
    let linear_b = linearized(blue_from_argb(argb));
    let x =
        SRGB_TO_XYZ[0][0] * linear_r + SRGB_TO_XYZ[0][1] * linear_g + SRGB_TO_XYZ[0][2] * linear_b;
    let y =
        SRGB_TO_XYZ[1][0] * linear_r + SRGB_TO_XYZ[1][1] * linear_g + SRGB_TO_XYZ[1][2] * linear_b;
    let z =
        SRGB_TO_XYZ[2][0] * linear_r + SRGB_TO_XYZ[2][1] * linear_g + SRGB_TO_XYZ[2][2] * linear_b;
    let x_normalized = x / WHITE_POINT_D65[0];
    let y_normalized = y / WHITE_POINT_D65[1];
    let z_normalized = z / WHITE_POINT_D65[2];
    let fx = lab_f(x_normalized);
    let fy = lab_f(y_normalized);
    let fz = lab_f(z_normalized);
    let l = 116.0 * fy - 16.0;
    let a = 500.0 * (fx - fy);
    let b = 200.0 * (fy - fz);
    [l, a, b]
}

fn lab_f(t: f64) -> f64 {
    let e = 216.0 / 24389.0;
    let kappa = 24389.0 / 27.0;
    if t > e {
        t.powf(1.0 / 3.0)
    } else {
        (kappa * t + 16.0) / 116.0
    }
}

fn lab_inv_f(ft: f64) -> f64 {
    let e = 216.0 / 24389.0;
    let kappa = 24389.0 / 27.0;
    let ft3 = ft * ft * ft;
    if ft3 > e {
        ft3
    } else {
        (116.0 * ft - 16.0) / kappa
    }
}

#[cfg(test)]
mod test {
    use approx_eq::assert_approx_eq;

    use crate::utils::color_utils::{
        argb_from_lab, argb_from_lstar, argb_from_rgb, argb_from_xyz, blue_from_argb, delinearized,
        green_from_argb, lab_from_argb, linearized, lstar_from_argb, red_from_argb, xyz_from_argb,
        y_from_lstar,
    };

    fn _lstar_from_y(y: f64) -> f64 {
        let scaled_y = y / 100.0;
        let e = 216.0 / 24389.0;
        if scaled_y <= e {
            24389.0 / 27.0 * scaled_y
        } else {
            let y_intermediate = scaled_y.powf(1.0 / 3.0);
            116.0 * y_intermediate - 16.0
        }
    }

    fn _range(start: f64, stop: f64, case_count: usize) -> Vec<f64> {
        let step_size = (stop - start) / (case_count - 1) as f64;
        (0..case_count)
            .map(|index| start + step_size * index as f64)
            .collect()
    }

    fn rgb_range() -> Vec<u32> {
        _range(0.0, 255.0, 8)
            .into_iter()
            .map(|element| element.round() as u32)
            .collect()
    }

    fn full_rgb_range() -> Vec<u32> {
        (0..256).collect()
    }

    #[test]
    fn range_integrity() {
        let range = _range(3.0, 9999.0, 1234);
        for i in 0..1234 {
            assert_approx_eq!(range[i], 3.0 + 8.1070559611 * i as f64, 1e-5);
        }
    }

    #[test]
    fn y_to_lstar_to_y() {
        let y_range = _range(0.0, 100.0, 1001);
        for y in y_range {
            assert_approx_eq!(y_from_lstar(_lstar_from_y(y)), y, 1e-5);
        }
    }

    #[test]
    fn lstar_to_y_to_lstar() {
        let lstar_range = _range(0.0, 100.0, 1001);
        for lstar in lstar_range {
            assert_approx_eq!(_lstar_from_y(y_from_lstar(lstar)), lstar, 1e-5);
        }
    }

    #[test]
    fn y_continuity() {
        let delta = 1e-8;
        let left = 8.0 - delta;
        let mid = 8.0;
        let right = 8.0 + delta;
        assert_approx_eq!(y_from_lstar(left), y_from_lstar(mid));
        assert_approx_eq!(y_from_lstar(right), y_from_lstar(mid));
    }

    #[test]
    fn rgb_to_xyz_to_rgb() {
        let r_range = rgb_range();
        let g_range = r_range.clone();
        let b_range = r_range.clone();
        for r in r_range {
            for g in &g_range {
                for b in &b_range {
                    let argb = argb_from_rgb(r, *g, *b);
                    let xyz = xyz_from_argb(argb);
                    let converted = argb_from_xyz(xyz[0], xyz[1], xyz[2]);
                    assert_approx_eq!(red_from_argb(converted) as f64, r as f64, 1.5);
                    assert_approx_eq!(green_from_argb(converted) as f64, *g as f64, 1.5);
                    assert_approx_eq!(blue_from_argb(converted) as f64, *b as f64, 1.5);
                }
            }
        }
    }

    #[test]
    fn rgb_to_lab_to_rgb() {
        let r_range = rgb_range();
        let g_range = r_range.clone();
        let b_range = r_range.clone();
        for r in r_range {
            for g in &g_range {
                for b in &b_range {
                    let argb = argb_from_rgb(r, *g, *b);
                    let lab = lab_from_argb(argb);
                    let converted = argb_from_lab(lab[0], lab[1], lab[2]);
                    assert_approx_eq!(red_from_argb(converted) as f64, r as f64, 1.5);
                    assert_approx_eq!(green_from_argb(converted) as f64, *g as f64, 1.5);
                    assert_approx_eq!(blue_from_argb(converted) as f64, *b as f64, 1.5);
                }
            }
        }
    }

    #[test]
    fn rgb_to_lstar_to_rgb() {
        let rgb_range = full_rgb_range();
        for component in rgb_range {
            let argb = argb_from_rgb(component, component, component);
            let lstar = lstar_from_argb(argb);
            let converted = argb_from_lstar(lstar);
            assert_eq!(converted, argb);
        }
    }

    #[test]
    fn rgb_to_lstar_to_y_commutes() {
        for r in rgb_range() {
            for g in rgb_range() {
                for b in rgb_range() {
                    let argb = argb_from_rgb(r, g, b);
                    let lstar = lstar_from_argb(argb);
                    let y = y_from_lstar(lstar);
                    let y2 = xyz_from_argb(argb)[1];
                    assert_approx_eq!(y, y2, 1e-5);
                }
            }
        }
    }

    #[test]
    fn lstar_to_rgb_to_y_commutes() {
        for lstar in _range(0.0, 100.0, 1001) {
            let argb = argb_from_lstar(lstar);
            let y = xyz_from_argb(argb)[1];
            let y2 = y_from_lstar(lstar);
            assert_approx_eq!(y, y2, 1.0);
        }
    }

    #[test]
    fn linearize_delinearize() {
        let rgb_range = full_rgb_range();
        for rgb_component in rgb_range {
            let converted = delinearized(linearized(rgb_component));
            assert_eq!(converted, rgb_component);
        }
    }
}
