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
    let ke = 8.0;
    if lstar > ke {
        ((lstar + 16.0) / 116.0).powf(3.0) * 100.0
    } else {
        lstar / 24389.0 / 27.0 * 100.0
    }
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

/// Returns the XYZ to sRGB transformation matrix.
pub fn xyz_to_srgb() -> [[f64; 3]; 3] {
    [
        [3.2406, -1.5372, -0.4986],
        [-0.9689, 1.8758, 0.0415],
        [0.0557, -0.204, 1.057],
    ]
}

/// Converts a color from ARGB to XYZ.
pub fn xyz_from_argb(argb: u32) -> [f64; 3] {
    let r = linearized(red_from_argb(argb));
    let g = linearized(green_from_argb(argb));
    let b = linearized(blue_from_argb(argb));

    matrix_multiply([r, g, b], srgb_to_xyz())
}

/// Converts a color from XYZ to ARGB.
pub fn argb_from_xyz(x: f64, y: f64, z: f64) -> u32 {
    let linear_rgb = matrix_multiply([x, y, z], xyz_to_srgb());
    let r = delinearized(linear_rgb[0]);
    let g = delinearized(linear_rgb[1]);
    let b = delinearized(linear_rgb[2]);
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
pub fn srgb_to_xyz() -> [[f64; 3]; 3] {
    [
        [0.41233895, 0.35762064, 0.18051042],
        [0.2126, 0.7152, 0.0722],
        [0.01932141, 0.11916382, 0.95034478],
    ]
}

/// Converts an L* value to an ARGB representation.
///
///
/// `lstar` L* in L*a*b*
/// Returns ARGB representation of grayscale color with lightness
/// matching L*
pub fn argb_from_lstar(lstar: f64) -> u32 {
    let fy = (lstar + 16.0) / 116.0;
    let fz = fy;
    let fx = fy;
    let kappa = 24389.0 / 27.0;
    let epsilon = 216.0 / 24389.0;
    let l_exceeds_epsilon_kappa = lstar > 8.0;
    let y = if l_exceeds_epsilon_kappa {
        fy * fy * fy
    } else {
        lstar / kappa
    };
    let cube_exceed_epsilon = fy * fy * fy > epsilon;
    let x = if cube_exceed_epsilon {
        fx * fx * fx
    } else {
        lstar / kappa
    };
    let z = if cube_exceed_epsilon {
        fz * fz * fz
    } else {
        lstar / kappa
    };
    let white_point = white_point_d65();
    argb_from_xyz(x * white_point[0], y * white_point[1], z * white_point[2])
}

/// Computes the L* value of a color in ARGB representation.
///
///
/// `argb` ARGB representation of a color
/// Returns L*, from L*a*b*, coordinate of the color
pub fn lstar_from_argb(argb: u32) -> f64 {
    let y = xyz_from_argb(argb)[1] / 100.0;

    let e = 216.0 / 24389.0;
    if y <= e {
        24389.0 / 27.0 * y
    } else {
        let y_intermediate = y.powf(1.0 / 3.0);

        116.0 * y_intermediate - 16.0
    }
}

/// Returns the standard white point; white on a sunny day.
///
///
/// Returns The white point
pub fn white_point_d65() -> [f64; 3] {
    [95.047, 100.0, 108.883]
}

/// Converts a color represented in Lab color space into an ARGB
/// integer.
pub fn argb_from_lab(l: f64, a: f64, b: f64) -> u32 {
    let white_point = white_point_d65();
    let fy = (l + 16.0) / 116.0;
    let fx = a / 500.0 + fy;
    let fz = fy - b / 200.0;
    let x_normalized = lab_inv_f(fx);
    let y_normalized = lab_inv_f(fy);
    let z_normalized = lab_inv_f(fz);
    let x = x_normalized * white_point[0];
    let y = y_normalized * white_point[1];
    let z = z_normalized * white_point[2];
    argb_from_xyz(x, y, z)
}

/// Converts a color from ARGB representation to L*a*b*
/// representation.
///
///
/// `argb` the ARGB representation of a color
/// Returns a Lab object representing the color
pub fn lab_from_argb(argb: u32) -> [f64; 3] {
    let white_point = white_point_d65();
    let xyz = xyz_from_argb(argb);
    let x_normalized = xyz[0] / white_point[0];
    let y_normalized = xyz[1] / white_point[1];
    let z_normalized = xyz[2] / white_point[2];
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
