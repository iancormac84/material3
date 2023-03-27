use std::{collections::HashMap, cmp::Ordering};

use crate::{
    hct::Cam16,
    utils::{
        color_utils::lstar_from_argb,
        math_utils::{calculate_difference_degrees, sanitize_degrees_int},
    },
};

const TARGET_CHROMA: f64 = 48.0;
const WEIGHT_PROPORTION: f64 = 0.7;
const WEIGHT_CHROMA_ABOVE: f64 = 0.3;
const WEIGHT_CHROMA_BELOW: f64 = 0.1;
const CUT_OFF_CHROMA: f64 = 5.0;
const CUT_OFF_EXCITED_PROPORTION: f64 = 0.01;
const CUT_OFF_TONE: f64 = 10.0;

#[derive(Clone, Copy)]
struct AnnotatedColor {
    pub argb: u32,
    pub cam: Cam16,
    pub excited_proportion: f64,
    pub score: f64,
}

fn argb_and_score_comparator(a: &AnnotatedColor, b: &AnnotatedColor) -> Ordering {
    match a.score > b.score {
        true => Ordering::Greater,
        false => match a.score == b.score {
            true => Ordering::Equal,
            false => Ordering::Less,
        }
    }
}

fn is_acceptable_color(color: &AnnotatedColor) -> bool {
    color.cam.chroma >= CUT_OFF_CHROMA
        && lstar_from_argb(color.argb) >= CUT_OFF_TONE
        && color.excited_proportion >= CUT_OFF_EXCITED_PROPORTION
}

fn colors_are_too_close(color_one: &AnnotatedColor, color_two: &AnnotatedColor) -> bool {
    calculate_difference_degrees(color_one.cam.hue, color_two.cam.hue) < 15.0
}

pub fn ranked_suggestions(argb_to_population: &HashMap<u32, u32>) -> Vec<u32> {
    let mut population_sum = 0.0;
    let input_size = argb_to_population.len();

    let mut argbs = vec![0; input_size];
    let mut populations = vec![0; input_size];

    for (key, value) in argb_to_population {
        argbs.push(*key);
        populations.push(*value);
    }

    for p in &populations {
        population_sum += *p as f64;
    }

    let mut hue_proportions = [0.0; 361];
    let mut colors = vec![];

    for i in 0..input_size {
        let proportion = populations[i] as f64 / population_sum;

        let cam = Cam16::from_int(argbs[i]);

        let hue = sanitize_degrees_int(cam.hue.round() as i16) as usize;
        hue_proportions[hue] += proportion;

        colors.push(AnnotatedColor {
            argb: argbs[i],
            cam,
            excited_proportion: 0.0,
            score: -1.0,
        });
    }

    for i in 0..input_size {
        let hue = colors[i].cam.hue.round() as i16;
        for j in (hue - 15)..(hue + 15) {
            let hue = sanitize_degrees_int(j) as usize;
            colors[i].excited_proportion += hue_proportions[hue];
        }
    }

    for i in 0..input_size {
        let proportion_score = colors[i].excited_proportion * 100.0 * WEIGHT_PROPORTION;

        let chroma = colors[i].cam.chroma;
        let chroma_weight = if chroma > TARGET_CHROMA {
            WEIGHT_CHROMA_ABOVE
        } else {
            WEIGHT_CHROMA_BELOW
        };
        let chroma_score = (chroma - TARGET_CHROMA) * chroma_weight;

        colors[i].score = chroma_score + proportion_score;
    }

    colors.sort_by(|a, b| argb_and_score_comparator(a, b));

    let mut selected_colors = vec![];

    for i in 0..input_size {
        if !is_acceptable_color(&colors[i]) {
            continue;
        }

        let mut is_duplicate_color = false;
        for j in 0..selected_colors.len() {
            if colors_are_too_close(&selected_colors[j], &colors[i]) {
                is_duplicate_color = true;
                break;
            }
        }

        if is_duplicate_color {
            continue;
        }

        selected_colors.push(colors[i]);
    }

    // Use google blue if no colors are selected.
    if selected_colors.is_empty() {
        selected_colors.push(AnnotatedColor {
            argb: 0xFF4285F4,
            cam: Cam16::default(),
            excited_proportion: 0.0,
            score: 0.0,
        });
    }

    let mut return_value = vec![0; selected_colors.len()];

    for j in 0..selected_colors.len() {
        return_value[j] = selected_colors[j].argb;
    }

    return_value
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use crate::score::ranked_suggestions;

    #[test]
    fn prioritizes_chroma_when_proportions_equal() {
        let mut colors_to_population = HashMap::new();
        colors_to_population.insert(0xffff0000, 1);
        colors_to_population.insert(0xffffffff, 1);
        colors_to_population.insert(0xff0000ff, 1);

        let ranked = ranked_suggestions(&colors_to_population);

        assert_eq!(ranked[2], 0xff0000ff);
    }

    #[test]
    fn prioritizes_chroma_when_proportions_equal2() {
        let mut colors_to_population = HashMap::new();
        colors_to_population.insert(0xffff0000, 1);
        colors_to_population.insert(0xff00ff00, 1);
        colors_to_population.insert(0xff0000ff, 1);

        let ranked = ranked_suggestions(&colors_to_population);

        assert_eq!(ranked[0], 0xffff0000);
        assert_eq!(ranked[1], 0xff00ff00);
        assert_eq!(ranked[2], 0xff0000ff);
    }

    #[test]
    fn generates_google_blue_when_no_colors_available() {
        let mut colors_to_population = HashMap::new();
        colors_to_population.insert(0xff000000, 1);

        let ranked = ranked_suggestions(&colors_to_population);

        assert_eq!(ranked[0], 0xff4285F4);
    }

    #[test]
    fn dedupes_nearby_hues() {
        let mut colors_to_population = HashMap::new();
        colors_to_population.insert(0xff008772, 1);
        colors_to_population.insert(0xff318477, 1);

        let ranked = ranked_suggestions(&colors_to_population);

        assert_eq!(ranked.len(), 1);
        assert_eq!(ranked[0], 0xff008772);
    }

    /*#[test]
    fn maximizes_hue_distance() {
        let mut colors_to_population = HashMap::new();
        colors_to_population.insert(0xff008772, 1);
        colors_to_population.insert(0xff008587, 1);
        colors_to_population.insert(0xff007EBC, 1);

        let ranked = ranked_suggestions(colors_to_population);

        assert_eq!(ranked.len(), 2);
        assert_eq!(ranked[0], 0xff007EBC);
        assert_eq!(ranked[1], 0xff008772);
    }*/
}