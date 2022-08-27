use crate::hct::Cam16;
use crate::utils::math_utils::{calculate_difference_degrees, sanitize_degrees_int};
use std::collections::HashMap;

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

pub fn score(colors_to_population: HashMap<u32, u32>) -> Vec<u32> {
    // Determine the total count of all colors.
    let mut population_sum = 0.0;

    for population in colors_to_population.values() {
        population_sum += *population as f64;
    }

    // Turn the count of each color into a proportion by dividing by the total
    // count. Also, fill a cache of CAM16 colors representing each color, and
    // record the proportion of colors for each CAM16 hue.
    let mut colors_to_cam: HashMap<u32, Cam16> = HashMap::new();
    let mut hue_proportions = vec![0.0; 361];
    for (color, population) in colors_to_population {
        let proportion = population as f64 / population_sum;

        let cam = Cam16::from_int(color);
        colors_to_cam.insert(color, cam);

        let hue = cam.hue.round() as usize;

        hue_proportions[hue] += proportion;
    }

    // Determine the proportion of the colors around each color, by summing the
    // proportions around each color's hue.
    let mut colors_to_excited_proportion: HashMap<u32, f64> = HashMap::new();
    for (color, cam) in &colors_to_cam {
        let hue = cam.hue.round() as usize;

        let mut excited_proportion = 0.0;
        let mut i = hue as i16 - 15;
        while i < hue as i16 + 15 {
            let neighbor_hue = sanitize_degrees_int(i);
            excited_proportion += hue_proportions[neighbor_hue as usize];
            i += 1;
        }
        colors_to_excited_proportion.insert(*color, excited_proportion);
    }

    // Score the colors by their proportion, as well as how chromatic they are.
    let mut colors_to_score: HashMap<u32, f64> = HashMap::new();
    for (color, cam) in &colors_to_cam {
        let proportion = colors_to_excited_proportion.get(color).unwrap();

        let proportion_score = proportion * 100.0 * WEIGHT_PROPORTION;

        let chroma_weight = if cam.chroma < TARGET_CHROMA {
            WEIGHT_CHROMA_BELOW
        } else {
            WEIGHT_CHROMA_ABOVE
        };
        let chroma_score = (cam.chroma - TARGET_CHROMA) * chroma_weight;

        let score = proportion_score + chroma_score;
        colors_to_score.insert(*color, score);
    }

    // Remove colors that are unsuitable, ex. very dark or unchromatic colors.
    // Also, remove colors that are very similar in hue.
    let filtered_colors = filter(&colors_to_excited_proportion, &colors_to_cam);
    let mut filtered_colors_to_score: HashMap<u32, f64> = HashMap::new();
    for color in filtered_colors {
        filtered_colors_to_score.insert(color, *colors_to_score.get(&color).unwrap());
    }

    // Ensure the list of colors returned is sorted such that the first in the
    // list is the most suitable, and the last is the least suitable.
    let mut entry_list: Vec<(u32, f64)> = filtered_colors_to_score.into_iter().collect();
    entry_list.sort_by(|(_, v0), (_, v1)| v0.total_cmp(v1).reverse());
    let mut colors_by_score_descending = vec![];
    for (color, _) in entry_list {
        let cam = colors_to_cam.get(&color);
        let mut duplicate_hue = false;

        for already_chosen_color in &colors_by_score_descending {
            let already_chosen_cam = colors_to_cam.get(&already_chosen_color);
            if calculate_difference_degrees(cam.unwrap().hue, already_chosen_cam.unwrap().hue) < 15.0
            {
                duplicate_hue = true;
                break;
            }
        }

        if duplicate_hue {
            continue;
        }
        colors_by_score_descending.push(color);
    }

    if colors_by_score_descending.is_empty() {
        colors_by_score_descending.push(0xff4285F4); // Google Blue
    }
    colors_by_score_descending
}

fn filter(
    colors_to_excited_proportion: &HashMap<u32, f64>,
    argb_to_hct: &HashMap<u32, Cam16>,
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
    use std::collections::HashMap;

    use super::score;

    #[test]
    fn prioritizes_chroma_when_proportions_equal() {
        let mut colors_to_population = HashMap::new();
        colors_to_population.insert(0xffff0000, 1);
        colors_to_population.insert(0xff00ff00, 1);
        colors_to_population.insert(0xff0000ff, 1);

        let ranked = score(colors_to_population);

        assert_eq!(ranked[0], 0xffff0000);
        assert_eq!(ranked[1], 0xff00ff00);
        assert_eq!(ranked[2], 0xff0000ff);
    }

    #[test]
    fn generates_google_blue_when_no_colors_available() {
        let mut colors_to_population = HashMap::new();
        colors_to_population.insert(0xff000000, 1);

        let ranked = score(colors_to_population);

        assert_eq!(ranked[0], 0xff4285F4);
    }

    #[test]
    fn dedupes_nearby_hues() {
        let mut colors_to_population = HashMap::new();
        colors_to_population.insert(0xff008772, 1);
        colors_to_population.insert(0xff318477, 1);

        let ranked = score(colors_to_population);

        assert_eq!(ranked.len(), 1);
        assert_eq!(ranked[0], 0xff008772);
    }

    /*#[test]
    fn maximizes_hue_distance() {
        let mut colors_to_population = HashMap::new();
        colors_to_population.insert(0xff008772, 1);
        colors_to_population.insert(0xff008587, 1);
        colors_to_population.insert(0xff007EBC, 1);

        let ranked = score(colors_to_population);

        assert_eq!(ranked.len(), 2);
        assert_eq!(ranked[0], 0xff007EBC);
        assert_eq!(ranked[1], 0xff008772);
    }*/
}
