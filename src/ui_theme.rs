use std::collections::BTreeMap;

use egui::{Color32, FontData, FontDefinitions, FontFamily, FontId, Stroke, Style, TextStyle};

pub fn load_theme(ctx: &egui::Context) {
    let mut style = Style {
        ..Default::default()
    };

    style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(35, 35, 38);
    style.visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(35, 35, 38);
    style.visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(42, 42, 46);
    style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(42, 42, 46);
    style.visuals.widgets.active.weak_bg_fill = Color32::from_rgb(50, 50, 55);
    style.visuals.widgets.active.bg_fill = Color32::from_rgb(50, 50, 55);

    style.visuals.widgets.active.bg_stroke = Stroke::NONE;
    style.visuals.widgets.inactive.bg_stroke = Stroke::NONE;
    style.visuals.widgets.hovered.bg_stroke = Stroke::NONE;
    style.visuals.widgets.active.fg_stroke = Stroke::new(2.0, Color32::from_rgb(54, 98, 54));
    style.visuals.widgets.inactive.fg_stroke = Stroke::new(2.0, Color32::from_rgb(54, 98, 54));
    style.visuals.widgets.hovered.fg_stroke = Stroke::new(2.0, Color32::from_rgb(54, 98, 54));
    style.visuals.widgets.active.expansion = 0.0;
    style.visuals.widgets.inactive.expansion = 0.0;
    style.visuals.widgets.hovered.expansion = 0.0;
    style.visuals.override_text_color = Some(Color32::from_rgb(235, 235, 235));

    let (font_definitions, text_styles) = get_style_with_updated_font();
    ctx.set_fonts(font_definitions);
    style.text_styles = text_styles;
    ctx.set_style(style);
}

pub fn get_style_with_updated_font() -> (FontDefinitions, BTreeMap<TextStyle, FontId>) {
    let mut fonts = FontDefinitions::default();

    fonts.font_data.insert(
        "SegoeUI".to_owned(),
        FontData::from_static(include_bytes!("../fonts/Segoe-UI-Variable-Static-Text.ttf")),
    );

    fonts
        .families
        .get_mut(&FontFamily::Proportional)
        .unwrap()
        .insert(0, "SegoeUI".to_owned());

    use FontFamily::{Monospace, Proportional};
    return (
        fonts,
        [
            (TextStyle::Small, FontId::new(10.0, Proportional)),
            (TextStyle::Body, FontId::new(12.0, Proportional)),
            (TextStyle::Monospace, FontId::new(12.0, Monospace)),
            (TextStyle::Button, FontId::new(12.0, Proportional)),
            (TextStyle::Heading, FontId::new(16.0, Proportional)),
        ]
        .into(),
    );
}