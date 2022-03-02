use indexmap::{IndexMap, IndexSet};
use material_color_utils::{
    quantize::{wu::QuantizerWu, Quantizer},
    score,
};

const RED: u32 = 0xffff0000;
const GREEN: u32 = 0xff00ff00;
const BLUE: u32 = 0xff0000ff;
const MAX_COLORS: u32 = 256;

fn main() {
    let mut wu = QuantizerWu::new();
    let pixels = vec![RED, GREEN, BLUE];
    let result = wu.quantize(&pixels, MAX_COLORS);
    let colors: Vec<u32> = result.color_to_count.keys().copied().collect();
    let color_set: IndexSet<u32> = colors.iter().map(|color| *color).collect();
    println!("color_set.len() is {}", color_set.len());
    println!("colors is {:?}", colors);

    let mut colors_to_population = IndexMap::new();
    colors_to_population.insert(0xffff0000, 1);
    colors_to_population.insert(0xff00ff00, 1);
    colors_to_population.insert(0xff0000ff, 1);

    let ranked = score(colors_to_population, 4, true);
    println!("ranked is {:?}", ranked);
}
