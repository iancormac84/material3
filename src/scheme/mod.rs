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
    pub outline_variant: u32,
    pub shadow: u32,
    pub scrim: u32,
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
        outline_variant: u32,
        shadow: u32,
        scrim: u32,
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
            outline_variant,
            shadow,
            scrim,
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

    pub fn light_content(color: u32) -> Scheme {
        let mut palette = CorePalette::content_of(color);
        Scheme::light_from_core_palette(&mut palette)
    }

    pub fn dark_content(color: u32) -> Scheme {
        let mut palette = CorePalette::content_of(color);
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
            outline_variant: palette.neutral_variant.tone(80).unwrap(),
            shadow: palette.neutral.tone(0).unwrap(),
            scrim: palette.neutral.tone(0).unwrap(),
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
            outline_variant: palette.neutral_variant.tone(30).unwrap(),
            shadow: palette.neutral.tone(0).unwrap(),
            scrim: palette.neutral.tone(0).unwrap(),
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
        assert_eq!(scheme.primary, 0xff343DFF);
    }

    #[test]
    fn blue_dark_scheme() {
        let scheme = Scheme::dark(0xff0000ff);
        assert_eq!(scheme.primary, 0xffBEC2FF);
    }

    #[test]
    fn third_party_light_scheme() {
        let scheme = Scheme::light(0xff6750A4);
        assert_eq!(scheme.primary, 0xff6750A4);
        assert_eq!(scheme.secondary, 0xff625B71);
        assert_eq!(scheme.tertiary, 0xff7E5260);
        assert_eq!(scheme.surface, 0xffFFFBFF);
        assert_eq!(scheme.on_surface, 0xff1C1B1E);
    }

    #[test]
    fn third_party_dark_scheme() {
        let scheme = Scheme::dark(0xff6750A4);
        assert_eq!(scheme.primary, 0xffCFBCFF);
        assert_eq!(scheme.secondary, 0xffCBC2DB);
        assert_eq!(scheme.tertiary, 0xffEFB8C8);
        assert_eq!(scheme.surface, 0xff1c1b1e);
        assert_eq!(scheme.on_surface, 0xffE6E1E6);
    }

    #[test]
    fn light_scheme_from_high_chroma_color() {
        let scheme = Scheme::light(0xfffa2bec);
        assert_eq!(scheme.primary, 0xffab00a2);
        assert_eq!(scheme.on_primary, 0xffffffff);
        assert_eq!(scheme.primary_container, 0xffffd7f3);
        assert_eq!(scheme.on_primary_container, 0xff390035);
        assert_eq!(scheme.secondary, 0xff6e5868);
        assert_eq!(scheme.on_secondary, 0xffffffff);
        assert_eq!(scheme.secondary_container, 0xfff8daee);
        assert_eq!(scheme.on_secondary_container, 0xff271624);
        assert_eq!(scheme.tertiary, 0xff815343);
        assert_eq!(scheme.on_tertiary, 0xffffffff);
        assert_eq!(scheme.tertiary_container, 0xffffdbd0);
        assert_eq!(scheme.on_tertiary_container, 0xff321207);
        assert_eq!(scheme.error, 0xffba1a1a);
        assert_eq!(scheme.on_error, 0xffffffff);
        assert_eq!(scheme.error_container, 0xffffdad6);
        assert_eq!(scheme.on_error_container, 0xff410002);
        assert_eq!(scheme.background, 0xfffffbff);
        assert_eq!(scheme.on_background, 0xff1f1a1d);
        assert_eq!(scheme.surface, 0xfffffbff);
        assert_eq!(scheme.on_surface, 0xff1f1a1d);
        assert_eq!(scheme.surface_variant, 0xffeedee7);
        assert_eq!(scheme.on_surface_variant, 0xff4e444b);
        assert_eq!(scheme.outline, 0xff80747b);
        assert_eq!(scheme.outline_variant, 0xffd2c2cb);
        assert_eq!(scheme.shadow, 0xff000000);
        assert_eq!(scheme.scrim, 0xff000000);
        assert_eq!(scheme.inverse_surface, 0xff342f32);
        assert_eq!(scheme.inverse_on_surface, 0xfff8eef2);
        assert_eq!(scheme.inverse_primary, 0xffffabee);
    }

    #[test]
    fn dark_scheme_from_high_chroma_color() {
        let scheme = Scheme::dark(0xfffa2bec);
        assert_eq!(scheme.primary, 0xffffabee);
        assert_eq!(scheme.on_primary, 0xff5c0057);
        assert_eq!(scheme.primary_container, 0xff83007b);
        assert_eq!(scheme.on_primary_container, 0xffffd7f3);
        assert_eq!(scheme.secondary, 0xffdbbed1);
        assert_eq!(scheme.on_secondary, 0xff3e2a39);
        assert_eq!(scheme.secondary_container, 0xff554050);
        assert_eq!(scheme.on_secondary_container, 0xfff8daee);
        assert_eq!(scheme.tertiary, 0xfff5b9a5);
        assert_eq!(scheme.on_tertiary, 0xff4c2619);
        assert_eq!(scheme.tertiary_container, 0xff663c2d);
        assert_eq!(scheme.on_tertiary_container, 0xffffdbd0);
        assert_eq!(scheme.error, 0xffffb4ab);
        assert_eq!(scheme.on_error, 0xff690005);
        assert_eq!(scheme.error_container, 0xff93000a);
        assert_eq!(scheme.on_error_container, 0xffffb4ab);
        assert_eq!(scheme.background, 0xff1f1a1d);
        assert_eq!(scheme.on_background, 0xffeae0e4);
        assert_eq!(scheme.surface, 0xff1f1a1d);
        assert_eq!(scheme.on_surface, 0xffeae0e4);
        assert_eq!(scheme.surface_variant, 0xff4e444b);
        assert_eq!(scheme.on_surface_variant, 0xffd2c2cb);
        assert_eq!(scheme.outline, 0xff9a8d95);
        assert_eq!(scheme.outline_variant, 0xff4e444b);
        assert_eq!(scheme.shadow, 0xff000000);
        assert_eq!(scheme.scrim, 0xff000000);
        assert_eq!(scheme.inverse_surface, 0xffeae0e4);
        assert_eq!(scheme.inverse_on_surface, 0xff342f32);
        assert_eq!(scheme.inverse_primary, 0xffab00a2);
    }

    #[test]
    fn light_content_scheme_from_high_chroma_color() {
        let scheme = Scheme::light_content(0xfffa2bec);
        assert_eq!(scheme.primary, 0xffab00a2);
        assert_eq!(scheme.on_primary, 0xffffffff);
        assert_eq!(scheme.primary_container, 0xffffd7f3);
        assert_eq!(scheme.on_primary_container, 0xff390035);
        assert_eq!(scheme.secondary, 0xff7f4e75);
        assert_eq!(scheme.on_secondary, 0xffffffff);
        assert_eq!(scheme.secondary_container, 0xffffd7f3);
        assert_eq!(scheme.on_secondary_container, 0xff330b2f);
        assert_eq!(scheme.tertiary, 0xff9c4323);
        assert_eq!(scheme.on_tertiary, 0xffffffff);
        assert_eq!(scheme.tertiary_container, 0xffffdbd0);
        assert_eq!(scheme.on_tertiary_container, 0xff390c00);
        assert_eq!(scheme.error, 0xffba1a1a);
        assert_eq!(scheme.on_error, 0xffffffff);
        assert_eq!(scheme.error_container, 0xffffdad6);
        assert_eq!(scheme.on_error_container, 0xff410002);
        assert_eq!(scheme.background, 0xfffffbff);
        assert_eq!(scheme.on_background, 0xff1f1a1d);
        assert_eq!(scheme.surface, 0xfffffbff);
        assert_eq!(scheme.on_surface, 0xff1f1a1d);
        assert_eq!(scheme.surface_variant, 0xffeedee7);
        assert_eq!(scheme.on_surface_variant, 0xff4e444b);
        assert_eq!(scheme.outline, 0xff80747b);
        assert_eq!(scheme.outline_variant, 0xffd2c2cb);
        assert_eq!(scheme.shadow, 0xff000000);
        assert_eq!(scheme.scrim, 0xff000000);
        assert_eq!(scheme.inverse_surface, 0xff342f32);
        assert_eq!(scheme.inverse_on_surface, 0xfff8eef2);
        assert_eq!(scheme.inverse_primary, 0xffffabee);
    }

    #[test]
    fn dark_content_scheme_from_high_chroma_color() {
        let scheme = Scheme::dark_content(0xfffa2bec);
        assert_eq!(scheme.primary, 0xffffabee);
        assert_eq!(scheme.on_primary, 0xff5c0057);
        assert_eq!(scheme.primary_container, 0xff83007b);
        assert_eq!(scheme.on_primary_container, 0xffffd7f3);
        assert_eq!(scheme.secondary, 0xfff0b4e1);
        assert_eq!(scheme.on_secondary, 0xff4b2145);
        assert_eq!(scheme.secondary_container, 0xff64375c);
        assert_eq!(scheme.on_secondary_container, 0xffffd7f3);
        assert_eq!(scheme.tertiary, 0xffffb59c);
        assert_eq!(scheme.on_tertiary, 0xff5c1900);
        assert_eq!(scheme.tertiary_container, 0xff7d2c0d);
        assert_eq!(scheme.on_tertiary_container, 0xffffdbd0);
        assert_eq!(scheme.error, 0xffffb4ab);
        assert_eq!(scheme.on_error, 0xff690005);
        assert_eq!(scheme.error_container, 0xff93000a);
        assert_eq!(scheme.on_error_container, 0xffffb4ab);
        assert_eq!(scheme.background, 0xff1f1a1d);
        assert_eq!(scheme.on_background, 0xffeae0e4);
        assert_eq!(scheme.surface, 0xff1f1a1d);
        assert_eq!(scheme.on_surface, 0xffeae0e4);
        assert_eq!(scheme.surface_variant, 0xff4e444b);
        assert_eq!(scheme.on_surface_variant, 0xffd2c2cb);
        assert_eq!(scheme.outline, 0xff9a8d95);
        assert_eq!(scheme.outline_variant, 0xff4e444b);
        assert_eq!(scheme.shadow, 0xff000000);
        assert_eq!(scheme.scrim, 0xff000000);
        assert_eq!(scheme.inverse_surface, 0xffeae0e4);
        assert_eq!(scheme.inverse_on_surface, 0xff342f32);
        assert_eq!(scheme.inverse_primary, 0xffab00a2);
    }
}
