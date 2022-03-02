use indexmap::IndexMap;

use crate::utils::color_utils;

use super::{map::QuantizerMap, Quantizer, QuantizerResult};

#[derive(Debug)]
pub struct QuantizerWu {
    weights: [u32; Self::TOTAL_SIZE],
    moments_r: [u32; Self::TOTAL_SIZE],
    moments_g: [u32; Self::TOTAL_SIZE],
    moments_b: [u32; Self::TOTAL_SIZE],
    moments: [f64; Self::TOTAL_SIZE],
    cubes: Vec<Cube>,
}

impl Default for QuantizerWu {
    fn default() -> Self {
        Self::new()
    }
}

impl Quantizer for QuantizerWu {
    fn quantize(&mut self, pixels: &[u32], max_colors: u32) -> QuantizerResult {
        let result = QuantizerMap.quantize(pixels, max_colors);
        self.construct_histogram(result.color_to_count);
        self.compute_moments();
        let create_cubes_result = self.create_cubes(max_colors as usize);
        let results = self.create_result(create_cubes_result.result_count as usize);
        QuantizerResult {
            color_to_count: results.iter().map(|e| (*e, 0)).collect(),
            input_pixel_to_cluster_pixel: IndexMap::new(),
        }
    }
}

impl QuantizerWu {
    // A histogram of all the input colors is constructed. It has the shape of a
    // cube. The cube would be too large if it contained all 16 million colors:
    // historical best practice is to use 5 bits  of the 8 in each channel,
    // reducing the histogram to a volume of ~32,000.
    pub const INDEX_BITS: usize = 5;
    pub const MAX_INDEX: usize = 32;
    pub const SIDE_LENGTH: usize = 33;
    pub const TOTAL_SIZE: usize = 35937;

    pub fn new() -> QuantizerWu {
        QuantizerWu {
            weights: [0; Self::TOTAL_SIZE],
            moments_r: [0; Self::TOTAL_SIZE],
            moments_g: [0; Self::TOTAL_SIZE],
            moments_b: [0; Self::TOTAL_SIZE],
            moments: [0.0; Self::TOTAL_SIZE],
            cubes: vec![],
        }
    }

    fn get_index(&self, r: usize, g: usize, b: usize) -> usize {
        (r << (Self::INDEX_BITS * 2))
            + (r << (Self::INDEX_BITS + 1))
            + (g << Self::INDEX_BITS)
            + r
            + g
            + b
    }
    fn construct_histogram(&mut self, pixels: IndexMap<u32, u32>) {
        self.weights = [0; Self::TOTAL_SIZE];
        self.moments_r = [0; Self::TOTAL_SIZE];
        self.moments_g = [0; Self::TOTAL_SIZE];
        self.moments_b = [0; Self::TOTAL_SIZE];
        self.moments = [0.0; Self::TOTAL_SIZE];
        for (pixel, count) in pixels {
            let red = color_utils::red_from_argb(pixel);
            let green = color_utils::green_from_argb(pixel);
            let blue = color_utils::blue_from_argb(pixel);
            let bits_to_remove = 8 - Self::INDEX_BITS;
            let i_r = (red >> bits_to_remove) + 1;
            let i_g = (green >> bits_to_remove) + 1;
            let i_b = (blue >> bits_to_remove) + 1;
            let index = self.get_index(i_r as usize, i_g as usize, i_b as usize);
            self.weights[index] += count;
            self.moments_r[index] += red * count;
            self.moments_g[index] += green * count;
            self.moments_b[index] += blue * count;
            self.moments[index] += (count * ((red * red) + (green * green) + (blue * blue))) as f64;
        }
    }
    fn compute_moments(&mut self) {
        for r in 1..Self::SIDE_LENGTH {
            let mut area = vec![0; Self::SIDE_LENGTH];
            let mut area_r = vec![0; Self::SIDE_LENGTH];
            let mut area_g = vec![0; Self::SIDE_LENGTH];
            let mut area_b = vec![0; Self::SIDE_LENGTH];
            let mut area2 = vec![0.0; Self::SIDE_LENGTH];
            for g in 1..Self::SIDE_LENGTH {
                let mut line = 0;
                let mut line_r = 0;
                let mut line_g = 0;
                let mut line_b = 0;
                let mut line2 = 0.0;
                for b in 1..Self::SIDE_LENGTH {
                    let index = self.get_index(r, g, b);
                    line += self.weights[index];
                    line_r += self.moments_r[index];
                    line_g += self.moments_g[index];
                    line_b += self.moments_b[index];
                    line2 += self.moments[index];

                    area[b] += line;
                    area_r[b] += line_r;
                    area_g[b] += line_g;
                    area_b[b] += line_b;
                    area2[b] += line2;

                    let previous_index = self.get_index(r - 1, g, b);
                    self.weights[index] = self.weights[previous_index] + area[b];
                    self.moments_r[index] = self.moments_r[previous_index] + area_r[b];
                    self.moments_g[index] = self.moments_g[previous_index] + area_g[b];
                    self.moments_b[index] = self.moments_b[previous_index] + area_b[b];
                    self.moments[index] = self.moments[previous_index] + area2[b];
                }
            }
        }
    }
    fn create_cubes(&mut self, max_color_count: usize) -> CreateCubesResult {
        self.cubes = vec![Cube::default(); max_color_count];
        let mut generated_color_count = max_color_count;
        {
            let cube_ref = &mut self.cubes[0];
            cube_ref.set_r1(Self::MAX_INDEX);
            cube_ref.set_g1(Self::MAX_INDEX);
            cube_ref.set_b1(Self::MAX_INDEX);
        }

        let mut volume_variance = vec![0.0; max_color_count];
        let mut next = 0;
        for mut i in 1..max_color_count {
            let mut cube_next = self.cubes[next].clone();
            let mut cube_i = self.cubes[i].clone();
            if self.cut(&mut cube_next, &mut cube_i) {
                let cube_next_vol = cube_next.vol;
                let cube_i_vol = cube_i.vol;
                self.cubes[next] = cube_next;
                self.cubes[i] = cube_i;
                let imm_cube_next = &self.cubes[next];
                let imm_cube_i = &self.cubes[i];
                volume_variance[next] = if cube_next_vol > 1 {
                    self.variance(imm_cube_next)
                } else {
                    0.0
                };
                volume_variance[i] = if cube_i_vol > 1 {
                    self.variance(imm_cube_i)
                } else {
                    0.0
                };
            } else {
                volume_variance[next] = 0.0;
                i -= 1;
            }

            next = 0;
            let mut temp = volume_variance[0];
            for j in 1..=i {
                if volume_variance[j] > temp {
                    temp = volume_variance[j];
                    next = j;
                }
            }
            if temp <= 0.0 {
                generated_color_count = i + 1;
                break;
            }
        }

        CreateCubesResult {
            requested_count: max_color_count as u32,
            result_count: generated_color_count as u32,
        }
    }
    fn create_result(&self, color_count: usize) -> Vec<u32> {
        let mut colors = vec![];
        for i in 0..color_count {
            let cube = &self.cubes[i];
            let weight = self.volume(cube, &self.weights);
            if weight > 0 {
                let r = self.volume(cube, &self.moments_r) / weight;
                let g = self.volume(cube, &self.moments_g) / weight;
                let b = self.volume(cube, &self.moments_b) / weight;
                let color = color_utils::argb_from_rgb(r, g, b);
                colors.push(color);
            }
        }
        colors
    }
    fn cut(&mut self, one: &mut Cube, two: &mut Cube) -> bool {
        let whole_r = self.volume(one, &self.moments_r);
        let whole_g = self.volume(one, &self.moments_g);
        let whole_b = self.volume(one, &self.moments_b);
        let whole_w = self.volume(one, &self.weights);

        let max_r_result = self.maximize(
            one,
            Direction::Red,
            one.r0 + 1,
            one.r1,
            whole_r,
            whole_g,
            whole_b,
            whole_w,
        );
        let max_g_result = self.maximize(
            one,
            Direction::Green,
            one.g0 + 1,
            one.g1,
            whole_r,
            whole_g,
            whole_b,
            whole_w,
        );
        let max_b_result = self.maximize(
            one,
            Direction::Blue,
            one.b0 + 1,
            one.b1,
            whole_r,
            whole_g,
            whole_b,
            whole_w,
        );

        let mut cut_direction = Direction::Red;
        let max_r = max_r_result.maximum;
        let max_g = max_g_result.maximum;
        let max_b = max_b_result.maximum;
        if max_r >= max_g && max_r >= max_b {
            if max_r_result.cut_location < 0 {
                return false;
            }
        } else if max_g >= max_r && max_g >= max_b {
            cut_direction = Direction::Green;
        } else {
            cut_direction = Direction::Blue;
        }

        two.r1 = one.r1;
        two.g1 = one.g1;
        two.b1 = one.b1;

        match cut_direction {
            Direction::Red => {
                one.r1 = max_r_result.cut_location as usize;
                two.r0 = one.r1;
                two.g0 = one.g0;
                two.b0 = one.b0;
            }
            Direction::Green => {
                one.g1 = max_g_result.cut_location as usize;
                two.r0 = one.r0;
                two.g0 = one.g1;
                two.b0 = one.b0;
            }
            Direction::Blue => {
                one.b1 = max_b_result.cut_location as usize;
                two.r0 = one.r0;
                two.g0 = one.g0;
                two.b0 = one.b1;
            }
        }

        one.vol = (one.r1 - one.r0) * (one.g1 - one.g0) * (one.b1 - one.b0);
        two.vol = (two.r1 - two.r0) * (two.g1 - two.g0) * (two.b1 - two.b0);
        true
    }
    fn maximize(
        &self,
        cube: &Cube,
        direction: Direction,
        first: usize,
        last: usize,
        whole_r: u32,
        whole_g: u32,
        whole_b: u32,
        whole_w: u32,
    ) -> MaximizeResult {
        let bottom_r = self.bottom(cube, direction.clone(), &self.moments_r);
        let bottom_g = self.bottom(cube, direction.clone(), &self.moments_g);
        let bottom_b = self.bottom(cube, direction.clone(), &self.moments_b);
        let bottom_w = self.bottom(cube, direction.clone(), &self.weights);

        let mut max = 0.0;
        let mut cut: isize = -1;

        for i in first..last {
            let mut half_r = bottom_r + self.top(cube, direction.clone(), i, &self.moments_r);
            let mut half_g = bottom_g + self.top(cube, direction.clone(), i, &self.moments_g);
            let mut half_b = bottom_b + self.top(cube, direction.clone(), i, &self.moments_b);
            let mut half_w = bottom_w + self.top(cube, direction.clone(), i, &self.weights);

            if half_w == 0 {
                continue;
            }

            let mut temp_numerator = (half_r * half_r) + (half_g * half_g) + (half_b * half_b);
            let mut temp_denominator = half_w;
            let mut temp = temp_numerator / temp_denominator;

            half_r = whole_r - half_r;
            half_g = whole_g - half_g;
            half_b = whole_b - half_b;
            half_w = whole_w - half_w;
            if half_w == 0 {
                continue;
            }
            temp_numerator = (half_r * half_r) + (half_g * half_g) + (half_b * half_b);
            temp_denominator = half_w;
            temp += temp_numerator / temp_denominator;

            if temp as f64 > max {
                max = temp as f64;
                cut = i as isize;
            }
        }
        MaximizeResult {
            cut_location: cut as i32,
            maximum: max,
        }
    }

    fn volume(&self, cube: &Cube, moment: &[u32]) -> u32 {
        moment[self.get_index(cube.r1, cube.g1, cube.b1)]
            - moment[self.get_index(cube.r1, cube.g1, cube.b0)]
            - moment[self.get_index(cube.r1, cube.g0, cube.b1)]
            + moment[self.get_index(cube.r1, cube.g0, cube.b0)]
            - moment[self.get_index(cube.r0, cube.g1, cube.b1)]
            + moment[self.get_index(cube.r0, cube.g1, cube.b0)]
            + moment[self.get_index(cube.r0, cube.g0, cube.b1)]
            - moment[self.get_index(cube.r0, cube.g0, cube.b0)]
    }
    fn bottom(&self, cube: &Cube, direction: Direction, moment: &[u32]) -> u32 {
        let res: i32 = match direction {
            Direction::Red => {
                let temp0: i32 = moment[self.get_index(cube.r0, cube.g1, cube.b1)]
                    .try_into()
                    .unwrap();
                let temp1: i32 = moment[self.get_index(cube.r0, cube.g1, cube.b0)]
                    .try_into()
                    .unwrap();
                let temp2: i32 = moment[self.get_index(cube.r0, cube.g0, cube.b1)]
                    .try_into()
                    .unwrap();
                let temp3: i32 = moment[self.get_index(cube.r0, cube.g0, cube.b0)]
                    .try_into()
                    .unwrap();
                -temp0 + temp1 + temp2 - temp3
            }
            Direction::Green => {
                let temp0: i32 = moment[self.get_index(cube.r1, cube.g0, cube.b1)]
                    .try_into()
                    .unwrap();
                let temp1: i32 = moment[self.get_index(cube.r1, cube.g0, cube.b0)]
                    .try_into()
                    .unwrap();
                let temp2: i32 = moment[self.get_index(cube.r0, cube.g0, cube.b1)]
                    .try_into()
                    .unwrap();
                let temp3: i32 = moment[self.get_index(cube.r0, cube.g0, cube.b0)]
                    .try_into()
                    .unwrap();
                -temp0 + temp1 + temp2 - temp3
            }
            Direction::Blue => {
                let temp0: i32 = moment[self.get_index(cube.r1, cube.g1, cube.b0)]
                    .try_into()
                    .unwrap();
                let temp1: i32 = moment[self.get_index(cube.r1, cube.g0, cube.b0)]
                    .try_into()
                    .unwrap();
                let temp2: i32 = moment[self.get_index(cube.r0, cube.g1, cube.b0)]
                    .try_into()
                    .unwrap();
                let temp3: i32 = moment[self.get_index(cube.r0, cube.g0, cube.b0)]
                    .try_into()
                    .unwrap();
                -temp0 + temp1 + temp2 - temp3
            }
        };
        res.try_into().unwrap()
    }

    fn top(&self, cube: &Cube, direction: Direction, position: usize, moment: &[u32]) -> u32 {
        match direction {
            Direction::Red => {
                moment[self.get_index(position, cube.g1, cube.b1)]
                    - moment[self.get_index(position, cube.g1, cube.b0)]
                    - moment[self.get_index(position, cube.g0, cube.b1)]
                    + moment[self.get_index(position, cube.g0, cube.b0)]
            }
            Direction::Green => {
                moment[self.get_index(cube.r1, position, cube.b1)]
                    - moment[self.get_index(cube.r1, position, cube.b0)]
                    - moment[self.get_index(cube.r0, position, cube.b1)]
                    + moment[self.get_index(cube.r0, position, cube.b0)]
            }
            Direction::Blue => {
                moment[self.get_index(cube.r1, cube.g1, position)]
                    - moment[self.get_index(cube.r1, cube.g0, position)]
                    - moment[self.get_index(cube.r0, cube.g1, position)]
                    + moment[self.get_index(cube.r0, cube.g0, position)]
            }
        }
    }
    fn variance(&self, cube: &Cube) -> f64 {
        let dr = self.volume(cube, &self.moments_r);
        let dg = self.volume(cube, &self.moments_g);
        let db = self.volume(cube, &self.moments_b);
        let xx = self.moments[self.get_index(cube.r1, cube.g1, cube.b1)]
            - self.moments[self.get_index(cube.r1, cube.g1, cube.b0)]
            - self.moments[self.get_index(cube.r1, cube.g0, cube.b1)]
            + self.moments[self.get_index(cube.r1, cube.g0, cube.b0)]
            - self.moments[self.get_index(cube.r0, cube.g1, cube.b1)]
            + self.moments[self.get_index(cube.r0, cube.g1, cube.b0)]
            + self.moments[self.get_index(cube.r0, cube.g0, cube.b1)]
            - self.moments[self.get_index(cube.r0, cube.g0, cube.b0)];

        let hypotenuse = (dr * dr + dg * dg + db * db) as f64;
        let volume_ = self.volume(cube, &self.weights) as f64;
        xx - hypotenuse / volume_
    }
}

#[derive(Debug, PartialEq, Clone)]
enum Direction {
    Red,
    Green,
    Blue,
}

struct MaximizeResult {
    pub cut_location: i32,
    pub maximum: f64,
}

pub struct CreateCubesResult {
    pub requested_count: u32,
    pub result_count: u32,
}

impl CreateCubesResult {
    pub fn new(requested_count: u32, result_count: u32) -> Self {
        Self {
            requested_count,
            result_count,
        }
    }
}

#[derive(Debug, Default, Clone)]
struct Cube {
    pub r0: usize,
    pub r1: usize,
    pub g0: usize,
    pub g1: usize,
    pub b0: usize,
    pub b1: usize,
    pub vol: usize,
}

impl Cube {
    pub fn set_r1(&mut self, r1: usize) {
        self.r1 = r1;
    }
    pub fn set_g1(&mut self, g1: usize) {
        self.g1 = g1;
    }
    pub fn set_b1(&mut self, b1: usize) {
        self.b1 = b1;
    }
}

#[cfg(test)]
mod test {
    use indexmap::IndexSet;

    use crate::quantize::Quantizer;

    use super::QuantizerWu;

    const RED: u32 = 0xffff0000;
    const GREEN: u32 = 0xff00ff00;
    const BLUE: u32 = 0xff0000ff;
    const MAX_COLORS: u32 = 256;

    #[test]
    fn len_is_one_red() {
        let mut wu = QuantizerWu::new();
        let pixels = vec![RED];
        let result = wu.quantize(&pixels, MAX_COLORS);
        let colors: Vec<u32> = result.color_to_count.keys().copied().collect();
        assert_eq!(colors.len(), 1);
        assert_eq!(colors[0], RED);
    }

    #[test]
    fn len_is_one_random() {
        let mut wu = QuantizerWu::new();
        let pixels = vec![0xff141216];
        let result = wu.quantize(&pixels, MAX_COLORS);
        let colors: Vec<u32> = result.color_to_count.keys().copied().collect();
        assert_eq!(colors.len(), 1);
        assert_eq!(colors[0], 0xff141216);
    }

    #[test]
    fn len_is_one_green() {
        let mut wu = QuantizerWu::new();
        let pixels = vec![GREEN];
        let result = wu.quantize(&pixels, MAX_COLORS);
        let colors: Vec<u32> = result.color_to_count.keys().copied().collect();
        assert_eq!(colors.len(), 1);
        assert_eq!(colors[0], GREEN);
    }

    #[test]
    fn len_is_one_blue() {
        let mut wu = QuantizerWu::new();
        let pixels = vec![BLUE];
        let result = wu.quantize(&pixels, MAX_COLORS);
        let colors: Vec<u32> = result.color_to_count.keys().copied().collect();
        assert_eq!(colors.len(), 1);
        assert_eq!(colors[0], BLUE);
    }

    #[test]
    fn len_is_one_from_five_blue() {
        let mut wu = QuantizerWu::new();
        let pixels = vec![BLUE, BLUE, BLUE, BLUE, BLUE];
        let result = wu.quantize(&pixels, MAX_COLORS);
        let colors: Vec<u32> = result.color_to_count.keys().copied().collect();
        assert_eq!(colors.len(), 1);
        assert_eq!(colors[0], BLUE);
    }

    #[test]
    fn two_red_three_green() {
        let mut wu = QuantizerWu::new();
        let pixels = vec![RED, RED, GREEN, GREEN, GREEN];
        let result = wu.quantize(&pixels, MAX_COLORS);
        let colors: Vec<u32> = result.color_to_count.keys().copied().collect();
        let color_set: IndexSet<u32> = colors.iter().map(|color| *color).collect();
        assert_eq!(color_set.len(), 2);
        assert_eq!(colors[0], GREEN);
        assert_eq!(colors[1], RED);
    }

    #[test]
    fn one_red_one_green_one_blue() {
        let mut wu = QuantizerWu::new();
        let pixels = vec![RED, GREEN, BLUE];
        let result = wu.quantize(&pixels, MAX_COLORS);
        let colors: Vec<u32> = result.color_to_count.keys().copied().collect();
        let color_set: IndexSet<u32> = colors.iter().map(|color| *color).collect();
        assert_eq!(color_set.len(), 3);
        assert_eq!(colors[0], BLUE);
        assert_eq!(colors[1], RED);
        assert_eq!(colors[2], GREEN);
    }
}
