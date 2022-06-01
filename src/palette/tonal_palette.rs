use crate::{error::ArgumentError, hct::Hct};
use indexmap::{IndexMap, IndexSet};
use std::fmt::Debug;

/// A convenience class for retrieving colors that are constant in hue and
/// chroma, but vary in tone.
///
/// This class can be instantiated in two ways:
/// 1. [`TonalPalette::of`] From hue and chroma. (preferred)
/// 2. [`TonalPalette::from_list`] From a fixed-size ([`TonalPalette::COMMON_SIZE`]) list of ints
/// representing ARBG colors. Correctness (constant hue and chroma) of the input
/// is not enforced. [`TonalPalette::tone`] will only return the input colors corresponding to
/// [`TonalPalette::COMMON_TONES`].

#[derive(Debug)]
pub struct TonalPalette {
    hue: Option<f64>,
    chroma: Option<f64>,
    cache: IndexMap<u32, u32>,
}

impl PartialEq for TonalPalette {
    fn eq(&self, other: &Self) -> bool {
        if self.hue.is_some() && self.chroma.is_some() {
            self.hue == other.hue && self.chroma == other.chroma
        } else {
            let self_cache_set = self.cache.values().copied().collect::<IndexSet<u32>>();
            let other_cache_set = other.cache.values().copied().collect::<IndexSet<u32>>();
            self_cache_set
                .difference(&other_cache_set)
                .cloned()
                .next()
                .is_none()
        }
    }
}

impl TonalPalette {
    /// Commonly-used tone values.
    pub const COMMON_TONES: [u32; 13] = [0, 10, 20, 30, 40, 50, 60, 70, 80, 90, 95, 99, 100];
    pub const COMMON_SIZE: usize = Self::COMMON_TONES.len();
    fn from_hue_and_chroma(hue: f64, chroma: f64) -> TonalPalette {
        TonalPalette {
            hue: Some(hue),
            chroma: Some(chroma),
            cache: IndexMap::new(),
        }
    }

    fn from_cache(cache: IndexMap<u32, u32>) -> TonalPalette {
        TonalPalette {
            cache,
            hue: None,
            chroma: None,
        }
    }

    ///Create tones using the HCT hue and chroma from a color.
    pub fn from_int(argb: u32) -> TonalPalette {
        let hct = Hct::from_int(argb);
        TonalPalette::from_hue_and_chroma(hct.hue, hct.chroma)
    }

    /// Create colors using `hue` and `chroma`.
    pub fn of(hue: f64, chroma: f64) -> TonalPalette {
        TonalPalette::from_hue_and_chroma(hue, chroma)
    }

    /// Create colors from a fixed-size list of ARGB color ints.
    ///
    /// Inverse of [`TonalPalette::as_list`].
    pub fn from_list(colors: &[u32]) -> TonalPalette {
        let msg = format!(
            "colors.len() is {} while COMMON_SIZE is {}",
            colors.len(),
            Self::COMMON_SIZE
        );
        assert_eq!(colors.len(), Self::COMMON_SIZE, "{}", &msg);
        let mut cache = IndexMap::new();
        for (index, tone) in Self::COMMON_TONES.iter().enumerate() {
            cache.insert(*tone, colors[index]);
        }
        TonalPalette::from_cache(cache)
    }

    /// Returns the ARGB representation of an HCT color.
    ///
    /// If the struct was instantiated from [`TonalPalette::of`] with `hue` and `chroma`, will return the
    /// color with corresponding `tone`.
    /// If the class was instantiated from a fixed-size list of color ints, `tone`
    /// must be one of the values present in [`TonalPalette::COMMON_TONES`].
    pub fn tone(&mut self, tone_value: u32) -> std::result::Result<u32, ArgumentError> {
        if self.hue.is_none() || self.chroma.is_none() {
            if !self.cache.contains_key(&tone_value) {
                let err_arg = ArgumentError::new(format!("Invalid argument (tone: {}): When a TonalPalette is created with TonalPalette::from_list, tone must be one of {:?}", tone_value, Self::COMMON_TONES));
                return Err(err_arg);
            } else {
                return Ok(*self.cache.get(&tone_value).unwrap());
            }
        }
        let chroma = if tone_value as f64 >= 90.0 {
            self.chroma.unwrap().min(40.0)
        } else {
            self.chroma.unwrap()
        };
        let tone_entry = self.cache.entry(tone_value);
        Ok(*tone_entry
            .or_insert_with(|| Hct::new(self.hue.unwrap(), chroma, tone_value as f64).to_int()))
    }

    /// Returns a fixed-size list of ARGB color ints for common tone values.
    ///
    /// Inverse of [`TonalPalette::from_list`].
    pub fn as_list(&mut self) -> Vec<u32> {
        Self::COMMON_TONES
            .iter()
            .map(|tone_| self.tone(*tone_).unwrap())
            .collect()
    }
}

#[cfg(test)]
mod test {
    use crate::hct::Hct;

    use super::TonalPalette;

    #[test]
    fn tonal_palette_of_blue() {
        let mut blue = TonalPalette::from_int(0xff0000ff);

        assert_eq!(blue.tone(100), Ok(0xffffffff));
        assert_eq!(blue.tone(99), Ok(0xfffffbff));
        assert_eq!(blue.tone(95), Ok(0xfff1efff));
        assert_eq!(blue.tone(90), Ok(0xffe0e0ff));
        assert_eq!(blue.tone(80), Ok(0xffbec2ff));
        assert_eq!(blue.tone(70), Ok(0xff9da3ff));
        assert_eq!(blue.tone(60), Ok(0xff7c84ff));
        assert_eq!(blue.tone(50), Ok(0xff5a64ff));
        assert_eq!(blue.tone(40), Ok(0xff343dff));
        assert_eq!(blue.tone(30), Ok(0xff0000ef));
        assert_eq!(blue.tone(20), Ok(0xff0001ac));
        assert_eq!(blue.tone(10), Ok(0xff00006e));
        assert_eq!(blue.tone(0), Ok(0xff000000));

        assert_eq!(blue.tone(3), Ok(0xff00003e));
    }

    #[test]
    fn of_constructor_as_list() {
        let hct = Hct::from_int(0xff0000ff);
        let mut tones = TonalPalette::of(hct.hue, hct.chroma);

        assert_eq!(
            tones.as_list(),
            [
                0xff000000, 0xff00006e, 0xff0001ac, 0xff0000ef, 0xff343dff, 0xff5a64ff, 0xff7c84ff,
                0xff9da3ff, 0xffbec2ff, 0xffe0e0ff, 0xfff1efff, 0xfffffbff, 0xffffffff,
            ]
        );
    }

    #[test]
    fn from_list_constructor_as_list() {
        let ints: Vec<u32> = (0..TonalPalette::COMMON_SIZE).map(|i| i as u32).collect();
        let mut tones = TonalPalette::from_list(&ints);
        assert_eq!(tones.as_list(), ints);
    }

    #[test]
    fn tones_of_each_common_tone() {
        let ints: Vec<u32> = (0..TonalPalette::COMMON_SIZE).map(|i| i as u32).collect();
        let mut tones = TonalPalette::from_list(&ints);

        assert_eq!(tones.tone(100), Ok(12));
        assert_eq!(tones.tone(99), Ok(11));
        assert_eq!(tones.tone(95), Ok(10));
        assert_eq!(tones.tone(90), Ok(9));
        assert_eq!(tones.tone(80), Ok(8));
        assert_eq!(tones.tone(70), Ok(7));
        assert_eq!(tones.tone(60), Ok(6));
        assert_eq!(tones.tone(50), Ok(5));
        assert_eq!(tones.tone(40), Ok(4));
        assert_eq!(tones.tone(30), Ok(3));
        assert_eq!(tones.tone(20), Ok(2));
        assert_eq!(tones.tone(10), Ok(1));
        assert_eq!(tones.tone(0), Ok(0));

        assert!(tones.tone(3).is_err());
    }

    #[test]
    fn equality() {
        let hct_ab = Hct::from_int(0xff0000ff);
        let tones_a = TonalPalette::of(hct_ab.hue, hct_ab.chroma);
        let tones_b = TonalPalette::of(hct_ab.hue, hct_ab.chroma);
        let hct_c = Hct::from_int(0xff123456);
        let tones_c = TonalPalette::of(hct_c.hue, hct_c.chroma);

        assert_eq!(tones_a, tones_b);
        assert_ne!(tones_b, tones_c);
    }
}
