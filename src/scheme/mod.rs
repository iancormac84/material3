use crate::palette::CorePalette;

/// This structure is the same concept as Flutter's ColorScheme class,
/// inlined into libmonet to ensure parity across languages.
pub struct Scheme {
    pub primary: u32,
    pub on_primary: u32,
    pub primary_container: u32,
    pub on_primary_container: u32,
    pub secondary: u32,
    pub on_secondary: u32,
    pub secondary_container: u32,
    pub on_secondary_container: u32,
    pub tertiary: u32,
    pub on_tertiary: u32,
    pub tertiary_container: u32,
    pub on_tertiary_container: u32,
    pub error: u32,
    pub on_error: u32,
    pub error_container: u32,
    pub on_error_container: u32,
    pub background: u32,
    pub on_background: u32,
    pub surface: u32,
    pub on_surface: u32,
    pub surface_variant: u32,
    pub on_surface_variant: u32,
    pub outline: u32,
    pub shadow: u32,
    pub inverse_surface: u32,
    pub inverse_on_surface: u32,
    pub inverse_primary: u32,
}

impl Scheme {
    pub fn new(
        primary: u32,
        on_primary: u32,
        primary_container: u32,
        on_primary_container: u32,
        secondary: u32,
        on_secondary: u32,
        secondary_container: u32,
        on_secondary_container: u32,
        tertiary: u32,
        on_tertiary: u32,
        tertiary_container: u32,
        on_tertiary_container: u32,
        error: u32,
        on_error: u32,
        error_container: u32,
        on_error_container: u32,
        background: u32,
        on_background: u32,
        surface: u32,
        on_surface: u32,
        surface_variant: u32,
        on_surface_variant: u32,
        outline: u32,
        shadow: u32,
        inverse_surface: u32,
        inverse_on_surface: u32,
        inverse_primary: u32,
    ) -> Scheme {
        Scheme {
            primary,
            on_primary,
            primary_container,
            on_primary_container,
            secondary,
            on_secondary,
            secondary_container,
            on_secondary_container,
            tertiary,
            on_tertiary,
            tertiary_container,
            on_tertiary_container,
            error,
            on_error,
            error_container,
            on_error_container,
            background,
            on_background,
            surface,
            on_surface,
            surface_variant,
            on_surface_variant,
            outline,
            shadow,
            inverse_surface,
            inverse_on_surface,
            inverse_primary,
        }
    }

    pub fn light(color: u32) -> Scheme {
        let mut palette = CorePalette::of(color);
        Scheme::light_from_core_palette(&mut palette)
    }

    pub fn dark(color: u32) -> Scheme {
        let mut palette = CorePalette::of(color);
        Scheme::dark_from_core_palette(&mut palette)
    }

    pub fn light_from_core_palette(palette: &mut CorePalette) -> Scheme {
        Scheme {
            primary: palette.primary.tone(40).unwrap(),
            on_primary: palette.primary.tone(100).unwrap(),
            primary_container: palette.primary.tone(90).unwrap(),
            on_primary_container: palette.primary.tone(10).unwrap(),
            secondary: palette.secondary.tone(40).unwrap(),
            on_secondary: palette.secondary.tone(100).unwrap(),
            secondary_container: palette.secondary.tone(90).unwrap(),
            on_secondary_container: palette.secondary.tone(10).unwrap(),
            tertiary: palette.tertiary.tone(40).unwrap(),
            on_tertiary: palette.tertiary.tone(100).unwrap(),
            tertiary_container: palette.tertiary.tone(90).unwrap(),
            on_tertiary_container: palette.tertiary.tone(10).unwrap(),
            error: palette.error.tone(40).unwrap(),
            on_error: palette.error.tone(100).unwrap(),
            error_container: palette.error.tone(90).unwrap(),
            on_error_container: palette.error.tone(10).unwrap(),
            background: palette.neutral.tone(99).unwrap(),
            on_background: palette.neutral.tone(10).unwrap(),
            surface: palette.neutral.tone(99).unwrap(),
            on_surface: palette.neutral.tone(10).unwrap(),
            surface_variant: palette.neutral_variant.tone(90).unwrap(),
            on_surface_variant: palette.neutral_variant.tone(30).unwrap(),
            outline: palette.neutral_variant.tone(50).unwrap(),
            shadow: palette.neutral.tone(0).unwrap(),
            inverse_surface: palette.neutral.tone(20).unwrap(),
            inverse_on_surface: palette.neutral.tone(95).unwrap(),
            inverse_primary: palette.primary.tone(80).unwrap(),
        }
    }

    pub fn dark_from_core_palette(palette: &mut CorePalette) -> Scheme {
        Scheme {
            primary: palette.primary.tone(80).unwrap(),
            on_primary: palette.primary.tone(20).unwrap(),
            primary_container: palette.primary.tone(30).unwrap(),
            on_primary_container: palette.primary.tone(90).unwrap(),
            secondary: palette.secondary.tone(80).unwrap(),
            on_secondary: palette.secondary.tone(20).unwrap(),
            secondary_container: palette.secondary.tone(30).unwrap(),
            on_secondary_container: palette.secondary.tone(90).unwrap(),
            tertiary: palette.tertiary.tone(80).unwrap(),
            on_tertiary: palette.tertiary.tone(20).unwrap(),
            tertiary_container: palette.tertiary.tone(30).unwrap(),
            on_tertiary_container: palette.tertiary.tone(90).unwrap(),
            error: palette.error.tone(80).unwrap(),
            on_error: palette.error.tone(20).unwrap(),
            error_container: palette.error.tone(30).unwrap(),
            on_error_container: palette.error.tone(80).unwrap(),
            background: palette.neutral.tone(10).unwrap(),
            on_background: palette.neutral.tone(90).unwrap(),
            surface: palette.neutral.tone(10).unwrap(),
            on_surface: palette.neutral.tone(90).unwrap(),
            surface_variant: palette.neutral_variant.tone(30).unwrap(),
            on_surface_variant: palette.neutral_variant.tone(80).unwrap(),
            outline: palette.neutral_variant.tone(60).unwrap(),
            shadow: palette.neutral.tone(0).unwrap(),
            inverse_surface: palette.neutral.tone(90).unwrap(),
            inverse_on_surface: palette.neutral.tone(20).unwrap(),
            inverse_primary: palette.primary.tone(40).unwrap(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::scheme::Scheme;

    #[test]
    fn blue_light_scheme() {
        let scheme = Scheme::light(0xff0000ff);
        assert_eq!(scheme.primary, 0xff333CFF);
    }

    #[test]
    fn blue_dark_scheme() {
        let scheme = Scheme::dark(0xff0000ff);
        assert_eq!(scheme.primary, 0xffBDC2FF);
    }

    #[test]
    fn third_party_light_scheme() {
        let scheme = Scheme::light(0xff6750A4);
        assert_eq!(scheme.primary, 0xff6750A4);
        assert_eq!(scheme.secondary, 0xff625B71);
        assert_eq!(scheme.tertiary, 0xff7D5260);
        assert_eq!(scheme.surface, 0xfffffbfe);
        assert_eq!(scheme.on_surface, 0xff1C1B1E);
    }

    #[test]
    fn third_party_dark_scheme() {
        let scheme = Scheme::dark(0xff6750A4);
        assert_eq!(scheme.primary, 0xffd0bcff);
        assert_eq!(scheme.secondary, 0xffCBC2DB);
        assert_eq!(scheme.tertiary, 0xffEFB8C8);
        assert_eq!(scheme.surface, 0xff1c1b1e);
        assert_eq!(scheme.on_surface, 0xffE6E1E5);
    }
}
