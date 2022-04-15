use material3::utils::color_utils::{
    argb_from_rgb, argb_from_xyz, blue_from_argb, green_from_argb, red_from_argb, xyz_from_argb,
};

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

fn main() {
    let r_range = rgb_range();
    let g_range = r_range.clone();
    let b_range = r_range.clone();
    for r in r_range {
        for g in &g_range {
            for b in &b_range {
                let argb = argb_from_rgb(r, *g, *b);
                let xyz = xyz_from_argb(argb);
                let converted = argb_from_xyz(xyz[0], xyz[1], xyz[2]);
                println!("converted is {converted}");
                let converted_red_from_argb = red_from_argb(converted);
                let converted_green_from_argb = green_from_argb(converted);
                let converted_blue_from_argb = blue_from_argb(converted);
                println!("r is {r} and converted_red_from_argb is {converted_red_from_argb}");
                println!("g is {g} and converted_green_from_argb is {converted_green_from_argb}");
                println!("b is {b} and converted_blue_from_argb is {converted_blue_from_argb}");
            }
        }
    }
}
