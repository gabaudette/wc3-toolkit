//! The Arcane look of `docs/design.md`: its colour tokens, Cormorant SC for
//! headings and Nunito Sans for text.

use std::sync::Arc;

use eframe::egui::{
    self, Button, Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, RichText,
    Stroke, TextStyle,
};

pub const SURFACE_0: Color32 = Color32::from_rgb(0x06, 0x0a, 0x13);
pub const SURFACE_1: Color32 = Color32::from_rgb(0x0d, 0x14, 0x24);
pub const SURFACE_2: Color32 = Color32::from_rgb(0x14, 0x20, 0x39);
pub const WELL: Color32 = Color32::from_rgb(0x08, 0x10, 0x1f);
pub const LINE: Color32 = Color32::from_rgb(0x1d, 0x2a, 0x45);
pub const FIELD: Color32 = Color32::from_rgb(0x5f, 0x72, 0x99);
pub const FRAME: Color32 = Color32::from_rgb(0x8a, 0x74, 0x44);
pub const INK: Color32 = Color32::from_rgb(0xe8, 0xec, 0xf5);
pub const MUTED: Color32 = Color32::from_rgb(0xa9, 0xb4, 0xcc);
pub const FAINT: Color32 = Color32::from_rgb(0x8f, 0x9b, 0xb5);
pub const GOLD: Color32 = Color32::from_rgb(0xe2, 0xc2, 0x7d);
pub const GOLD_BRIGHT: Color32 = Color32::from_rgb(0xf3, 0xdb, 0xa6);
pub const GOLD_SOFT: Color32 = Color32::from_rgb(0x2a, 0x24, 0x14);
pub const GLOW: Color32 = Color32::from_rgb(0x6f, 0xd3, 0xe0);
pub const FOCUS: Color32 = Color32::from_rgb(0xe8, 0xec, 0xf5);
pub const DANGER: Color32 = Color32::from_rgb(0xff, 0x8a, 0x8a);

const BODY: &str = "nunito";
const BOLD: &str = "nunito-bold";
const DISPLAY: &str = "cormorant";
const BODY_SIZE: f32 = 15.0;
const SMALL_SIZE: f32 = 13.0;
const HEADING_SIZE: f32 = 30.0;
const TITLE_SIZE: f32 = 24.0;
pub const RADIUS: u8 = 3;

pub fn apply(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();

    for (name, data) in [
        (BODY, &include_bytes!("../fonts/NunitoSans-Regular.ttf")[..]),
        (BOLD, &include_bytes!("../fonts/NunitoSans-Bold.ttf")[..]),
        (
            DISPLAY,
            &include_bytes!("../fonts/CormorantSC-Bold.ttf")[..],
        ),
    ] {
        fonts
            .font_data
            .insert(name.to_owned(), Arc::new(FontData::from_static(data)));
    }

    let fallbacks = fonts
        .families
        .get(&FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();

    for (family, first) in [
        (FontFamily::Proportional, BODY),
        (FontFamily::Name(BOLD.into()), BOLD),
        (FontFamily::Name(DISPLAY.into()), DISPLAY),
    ] {
        let mut list = vec![first.to_owned()];

        list.extend(fallbacks.iter().cloned());
        fonts.families.insert(family, list);
    }

    ctx.set_fonts(fonts);
    ctx.set_theme(egui::Theme::Dark);
    ctx.all_styles_mut(|style| {
        style.text_styles.insert(
            TextStyle::Body,
            FontId::new(BODY_SIZE, FontFamily::Proportional),
        );
        style.text_styles.insert(
            TextStyle::Button,
            FontId::new(BODY_SIZE, FontFamily::Proportional),
        );
        style.text_styles.insert(
            TextStyle::Small,
            FontId::new(SMALL_SIZE, FontFamily::Proportional),
        );
        style.text_styles.insert(
            TextStyle::Heading,
            FontId::new(HEADING_SIZE, FontFamily::Name(DISPLAY.into())),
        );
        style.spacing.item_spacing = egui::vec2(8.0, 10.0);
        style.spacing.button_padding = egui::vec2(12.0, 6.0);
        style.spacing.interact_size.y = 30.0;

        let visuals = &mut style.visuals;
        let radius = CornerRadius::same(RADIUS);

        visuals.dark_mode = true;
        visuals.override_text_color = Some(INK);
        visuals.panel_fill = SURFACE_1;
        visuals.window_fill = SURFACE_1;
        visuals.extreme_bg_color = WELL;
        visuals.faint_bg_color = SURFACE_2;
        visuals.error_fg_color = DANGER;
        visuals.warn_fg_color = GOLD;
        visuals.hyperlink_color = GLOW;
        visuals.selection.bg_fill = GOLD_SOFT;
        visuals.selection.stroke = Stroke::new(1.0, GOLD);

        let widgets = &mut visuals.widgets;

        widgets.noninteractive.bg_stroke = Stroke::new(1.0, LINE);
        widgets.noninteractive.fg_stroke = Stroke::new(1.0, INK);
        widgets.inactive.bg_fill = SURFACE_2;
        widgets.inactive.weak_bg_fill = SURFACE_2;
        widgets.inactive.bg_stroke = Stroke::new(1.0, FIELD);
        widgets.inactive.fg_stroke = Stroke::new(1.0, INK);
        widgets.hovered.bg_fill = SURFACE_2;
        widgets.hovered.weak_bg_fill = SURFACE_2;
        widgets.hovered.bg_stroke = Stroke::new(1.0, FOCUS);
        widgets.hovered.fg_stroke = Stroke::new(1.0, INK);
        widgets.active.bg_fill = GOLD_SOFT;
        widgets.active.weak_bg_fill = GOLD_SOFT;
        widgets.active.bg_stroke = Stroke::new(2.0, FOCUS);
        widgets.active.fg_stroke = Stroke::new(1.0, GOLD_BRIGHT);
        widgets.open = widgets.active;

        for state in [
            &mut widgets.noninteractive,
            &mut widgets.inactive,
            &mut widgets.hovered,
            &mut widgets.active,
            &mut widgets.open,
        ] {
            state.corner_radius = radius;
        }
    });
}

pub fn heading(text: &str) -> RichText {
    RichText::new(text).heading().color(GOLD)
}

pub fn title(text: &str) -> RichText {
    RichText::new(text)
        .family(FontFamily::Name(DISPLAY.into()))
        .size(TITLE_SIZE)
        .color(GOLD)
}

pub fn strong(text: &str) -> RichText {
    RichText::new(text).family(FontFamily::Name(BOLD.into()))
}

/// A step that succeeded.
pub fn done(text: &str) -> RichText {
    strong(text).color(GLOW)
}

pub fn muted(text: &str) -> RichText {
    RichText::new(text).color(MUTED)
}

pub fn faint(text: &str) -> RichText {
    RichText::new(text).color(FAINT).small()
}

pub fn path(text: &str) -> RichText {
    RichText::new(text)
        .family(FontFamily::Monospace)
        .size(SMALL_SIZE)
        .color(FAINT)
}

/// A step not reached yet, in the side list.
pub fn upcoming(text: &str) -> RichText {
    RichText::new(text).color(FAINT)
}

pub fn error(text: &str) -> RichText {
    RichText::new(text).color(DANGER)
}

/// The step's main action: gold on gold-soft, as the app's primary buttons.
pub fn primary(text: &str) -> Button<'static> {
    Button::new(RichText::new(text.to_owned()).color(GOLD_BRIGHT))
        .fill(GOLD_SOFT)
        .stroke(Stroke::new(1.0, GOLD))
}
