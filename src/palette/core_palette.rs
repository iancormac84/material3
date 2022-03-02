use crate::{hct::Cam16, palette::tonal_palette::TonalPalette};

/// An intermediate concept between the key color for a UI theme, and a full
/// color scheme. 5 tonal palettes are generated, all except one use the same
/// hue as the key color, and all vary in chroma.
#[derive(Debug, PartialEq)]
pub struct CorePalette {
    pub primary: TonalPalette,
    pub secondary: TonalPalette,
    pub tertiary: TonalPalette,
    pub neutral: TonalPalette,
    pub neutral_variant: TonalPalette,
    pub error: TonalPalette,
}

impl CorePalette {
    pub const SIZE: usize = 5;
    /// Create a [`CorePalette`] from a source ARGB color.
    pub fn of(argb: u32) -> CorePalette {
        let cam = Cam16::from_int(argb);
        CorePalette {
            primary: TonalPalette::of(cam.hue, (48f64).max(cam.chroma)),
            secondary: TonalPalette::of(cam.hue, 16.0),
            tertiary: TonalPalette::of(cam.hue + 60.0, 24.0),
            neutral: TonalPalette::of(cam.hue, 4.0),
            neutral_variant: TonalPalette::of(cam.hue, 8.0),
            error: TonalPalette::of(25.0, 84.0),
        }
    }

    /// Create a [`CorePalette`] from a fixed-size list of ARGB color ints
    /// representing concatenated tonal palettes.
    ///
    /// Inverse of [`CorePalette::as_list`].
    pub fn from_list(colors: &[u32]) -> CorePalette {
        assert_eq!(colors.len(), Self::SIZE * TonalPalette::COMMON_SIZE);
        CorePalette {
            primary: TonalPalette::from_list(get_partition(colors, 0, TonalPalette::COMMON_SIZE)),
            secondary: TonalPalette::from_list(get_partition(colors, 1, TonalPalette::COMMON_SIZE)),
            tertiary: TonalPalette::from_list(get_partition(colors, 2, TonalPalette::COMMON_SIZE)),
            neutral: TonalPalette::from_list(get_partition(colors, 3, TonalPalette::COMMON_SIZE)),
            neutral_variant: TonalPalette::from_list(get_partition(
                colors,
                4,
                TonalPalette::COMMON_SIZE,
            )),
            error: TonalPalette::of(25.0, 84.0),
        }
    }

    /// Returns a list of ARGB color from concatenated tonal palettes.
    ///
    /// Inverse of [`CorePalette::from_list`].
    pub fn as_list(&mut self) -> Vec<u32> {
        vec![
            self.primary.as_list(),
            self.secondary.as_list(),
            self.tertiary.as_list(),
            self.neutral.as_list(),
            self.neutral_variant.as_list(),
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}

#[cfg(test)]
mod test {
    use super::CorePalette;
    use crate::palette::TonalPalette;

    #[test]
    fn as_list() {
        let ints: Vec<u32> = (0..CorePalette::SIZE * TonalPalette::COMMON_SIZE)
            .map(|i| i as u32)
            .collect();

        let mut core_palette = CorePalette::from_list(&ints);
        assert_eq!(core_palette.as_list(), ints);
    }

    #[test]
    fn equality() {
        let core_palette_a = CorePalette::of(0xff0000ff);
        let core_palette_b = CorePalette::of(0xff0000ff);
        let core_palette_c = CorePalette::of(0xff123456);

        assert_eq!(core_palette_a, core_palette_b);
        assert_ne!(core_palette_b, core_palette_c);
    }
}

// Returns a partition from a list.
//
// For example, given a list with 2 partitions of size 3.
// range = [1, 2, 3, 4, 5, 6];
//
// range.get_partition(0, 3) // [1, 2, 3]
// range.get_partition(1, 3) // [4, 5, 6]
fn get_partition(list: &[u32], partition_number: usize, partition_size: usize) -> &[u32] {
    &list[partition_number * partition_size..(partition_number + 1) * partition_size]
}
