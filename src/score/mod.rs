use crate::hct::Hct;
use crate::utils::math_utils::{calculate_difference_degrees, sanitize_degrees_int};
use indexmap::IndexMap;
use std::cmp::Ordering;

#[derive(Debug)]
pub struct ArgbAndScore {
    pub argb: u32,
    pub score: f64,
}

impl ArgbAndScore {
    pub fn new(argb: u32, score: f64) -> ArgbAndScore {
        Self { argb, score }
    }
}

impl PartialEq for ArgbAndScore {
    fn eq(&self, other: &Self) -> bool {
        self.score.eq(&other.score)
    }
}

impl PartialOrd for ArgbAndScore {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.score.partial_cmp(&other.score)
    }
}

/*impl Ord for ArgbAndScore {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.score > other.score {
            Ordering::Less
        } else if self.score == other.score {
            Ordering::Equal
        } else {
            Ordering::Greater
        }
    }
}*/

const TARGET_CHROMA: f64 = 48.0;
const WEIGHT_PROPORTION: f64 = 0.7;
const WEIGHT_CHROMA_ABOVE: f64 = 0.3;
const WEIGHT_CHROMA_BELOW: f64 = 0.1;
const CUT_OFF_CHROMA: f64 = 5.0;
const CUT_OFF_EXCITED_PROPORTION: f64 = 0.01;

/// Given a large set of colors, remove colors that are unsuitable for a UI
/// theme, and rank the rest based on suitability.
///
/// Enables use of a high cluster count for image quantization, thus ensuring
/// colors aren't muddied, while curating the high cluster count to a much
///  smaller number of appropriate choices.

/// Given a map with keys of colors and values of how often the color appears,
/// rank the colors based on suitability for being used for a UI theme.
///
/// `colors_to_population` is a map with keys of colors and values of often the
/// color appears, usually from a source image.
///
/// The list returned is of length <= `desired`. The recommended color is the
/// first item, the least suitable is the last. There will always be at least
/// one color returned. If all the input colors were not suitable for a theme,
/// Google Blue will be provided as a default fallback color. The default
/// number of colors returned is 4, simply because thats the # of colors
/// display in Android 12's wallpaper picker.

pub fn score(colors_to_population: IndexMap<u32, u32>, desired: usize, filter: bool) -> Vec<u32> {
    let mut population_sum = 0.0;

    for population in colors_to_population.values() {
        population_sum += *population as f64;
    }

    // Turn the count of each color into a proportion by dividing by the total
    // count. Also, fill a cache of CAM16 colors representing each color, and
    // record the proportion of colors for each CAM16 hue.
    let mut argb_to_raw_proportion: IndexMap<u32, f64> = IndexMap::new();
    let mut argb_to_hct: IndexMap<u32, Hct> = IndexMap::new();
    let mut hue_proportions = vec![0.0; 360];
    for color in colors_to_population.keys() {
        let population = *colors_to_population.get(color).unwrap();
        let proportion = population as f64 / population_sum;
        argb_to_raw_proportion.insert(*color, proportion);

        let hct = Hct::from_int(*color);
        let hue = hct.hue.floor();
        argb_to_hct.insert(*color, hct);

        hue_proportions[hue as usize] += proportion;
    }

    // Determine the proportion of the colors around each color, by summing the
    // proportions around each color's hue.
    let mut argb_to_hue_proportion: IndexMap<u32, f64> = IndexMap::new();
    for (color, hct) in &argb_to_hct {
        let hue = hct.hue.round() as i16;

        let mut excited_proportion = 0.0;
        let mut i = hue - 15;
        while i < hue + 15 {
            let neighbor_hue = sanitize_degrees_int(i);
            excited_proportion += hue_proportions[neighbor_hue as usize];
            i += 1;
        }
        argb_to_hue_proportion.insert(*color, excited_proportion);
    }

    // Remove colors that are unsuitable, ex. very dark or unchromatic colors.
    // Also, remove colors that are very similar in hue.
    let filtered_colors = if filter {
        run_filter(&argb_to_hue_proportion, &argb_to_hct)
    } else {
        argb_to_hue_proportion.keys().copied().collect()
    };

    // Score the colors by their proportion, as well as how chromatic they are.
    let mut argb_to_score: IndexMap<u32, f64> = IndexMap::new();
    for color in filtered_colors {
        let cam = argb_to_hct.get(&color).unwrap();
        let proportion = argb_to_hue_proportion.get(&color).unwrap();

        let proportion_score = proportion * 100.0 * WEIGHT_PROPORTION;

        let chroma_weight = if cam.chroma < TARGET_CHROMA {
            WEIGHT_CHROMA_BELOW
        } else {
            WEIGHT_CHROMA_ABOVE
        };
        let chroma_score = (cam.chroma - TARGET_CHROMA) * chroma_weight;

        let score = proportion_score + chroma_score;
        argb_to_score.insert(color, score);
    }

    let mut argb_and_score_sorted: Vec<(u32, f64)> = argb_to_score
        .iter()
        .map(|entry| (*entry.0, *entry.1))
        .collect();
    println!("argb_and_score_sorted is {:?}", argb_and_score_sorted);
    argb_and_score_sorted.sort_by(|a, b| (b.1).partial_cmp(&a.1).unwrap());
    println!("argb_and_score_sorted is now {:?}", argb_and_score_sorted);
    let argbs_score_sorted: Vec<u32> = argb_and_score_sorted.iter().map(|e| e.0).collect();
    println!("argbs_score_sorted is again now {:?}", argbs_score_sorted);

    let mut final_colors_to_score: IndexMap<u32, f64> = IndexMap::new();
    let mut difference_degrees = 90.0;
    while difference_degrees >= 15.0 {
        final_colors_to_score.clear();
        for color in &argbs_score_sorted {
            let mut duplicate_hue = false;
            let cam = argb_to_hct.get(color).unwrap();
            for already_chosen_color in final_colors_to_score.keys() {
                let already_chosen_cam = argb_to_hct.get(already_chosen_color).unwrap();
                if calculate_difference_degrees(cam.hue, already_chosen_cam.hue)
                    < difference_degrees
                {
                    duplicate_hue = true;
                    break;
                }
            }
            if !duplicate_hue {
                final_colors_to_score.insert(*color, *argb_to_score.get(color).unwrap());
            }
        }
        if final_colors_to_score.len() >= desired {
            break;
        }
        difference_degrees -= 1.0;
    }
    println!("final_colors_to_score is {:?}", final_colors_to_score);

    // Ensure the list of colors returned is sorted such that the first in the
    // list is the most suitable, and the last is the least suitable.
    let colors_by_score_descending: Vec<ArgbAndScore> = final_colors_to_score
        .iter()
        .map(|entry| ArgbAndScore::new(*entry.0, *entry.1))
        .collect();
    println!(
        "colors_by_score_descending is {:?}",
        colors_by_score_descending
    );
    /*colors_by_score_descending.sort_unstable();
    println!("colors_by_score_descending is now {:?}", colors_by_score_descending);*/

    // Ensure that at least one color is returned.
    if colors_by_score_descending.is_empty() {
        return vec![0xff4285f4]; // Google Blue
    }
    colors_by_score_descending.iter().map(|e| e.argb).collect()
}

fn run_filter(
    colors_to_excited_proportion: &IndexMap<u32, f64>,
    argb_to_hct: &IndexMap<u32, Hct>,
) -> Vec<u32> {
    let mut filtered = vec![];

    for (color, hct) in argb_to_hct {
        let proportion = *colors_to_excited_proportion.get(color).unwrap();

        if hct.chroma >= CUT_OFF_CHROMA && proportion > CUT_OFF_EXCITED_PROPORTION {
            filtered.push(*color);
        } else {
            println!("rejecting color {}", color);
        }
    }
    filtered
}

#[cfg(test)]
mod test {
    use indexmap::IndexMap;

    use super::score;

    #[test]
    fn prioritizes_chroma_when_proportions_equal() {
        let mut colors_to_population = IndexMap::new();
        colors_to_population.insert(0xffff0000, 1);
        colors_to_population.insert(0xff00ff00, 1);
        colors_to_population.insert(0xff0000ff, 1);

        let ranked = score(colors_to_population, 4, true);

        assert_eq!(ranked[0], 0xffff0000);
        assert_eq!(ranked[1], 0xff00ff00);
        assert_eq!(ranked[2], 0xff0000ff);
    }

    #[test]
    fn generates_google_blue_when_no_colors_available() {
        let mut colors_to_population = IndexMap::new();
        colors_to_population.insert(0xff000000, 1);

        let ranked = score(colors_to_population, 4, true);

        assert_eq!(ranked[0], 0xff4285F4);
    }

    #[test]
    fn dedupes_nearby_hues() {
        let mut colors_to_population = IndexMap::new();
        colors_to_population.insert(0xff008772, 1);
        colors_to_population.insert(0xff318477, 1);

        let ranked = score(colors_to_population, 4, true);

        assert_eq!(ranked.len(), 1);
        assert_eq!(ranked[0], 0xff008772);
    }

    #[test]
    fn maximizes_hue_distance() {
        let mut colors_to_population = IndexMap::new();
        colors_to_population.insert(0xff008772, 1);
        colors_to_population.insert(0xff008587, 1);
        colors_to_population.insert(0xff007EBC, 1);

        let ranked = score(colors_to_population, 2, true);

        assert_eq!(ranked.len(), 2);
        assert_eq!(ranked[0], 0xff007EBC);
        assert_eq!(ranked[1], 0xff008772);
    }
}
