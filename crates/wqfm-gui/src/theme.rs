//! Colours and spacing of the app: a light theme with the icon's colours.

use eframe::egui::{
    self, Color32, CornerRadius, FontFamily, FontId, Margin, Stroke, TextStyle, Vec2,
};

pub const NAVY: Color32 = Color32::from_rgb(0x0B, 0x1B, 0x33);
pub const TEAL: Color32 = Color32::from_rgb(0x14, 0xB8, 0xA6);
pub const INDIGO: Color32 = Color32::from_rgb(0x63, 0x66, 0xF1);
pub const ORANGE: Color32 = Color32::from_rgb(0xEA, 0x58, 0x0C);

pub const CANVAS: Color32 = Color32::from_rgb(0xF8, 0xFA, 0xFC);
pub const SIDEBAR: Color32 = Color32::from_rgb(0xFF, 0xFF, 0xFF);
pub const CARD: Color32 = Color32::from_rgb(0xFF, 0xFF, 0xFF);
pub const SUNKEN: Color32 = Color32::from_rgb(0xF1, 0xF5, 0xF9);
pub const BORDER: Color32 = Color32::from_rgb(0xE2, 0xE8, 0xF0);
pub const INK: Color32 = Color32::from_rgb(0x0F, 0x17, 0x2A);
pub const MUTED: Color32 = Color32::from_rgb(0x64, 0x74, 0x8B);
pub const ERROR_BG: Color32 = Color32::from_rgb(0xFE, 0xF2, 0xF2);
pub const ERROR_INK: Color32 = Color32::from_rgb(0xB9, 0x1C, 0x1C);
pub const WARN_BG: Color32 = Color32::from_rgb(0xFF, 0xFB, 0xEB);
pub const WARN_INK: Color32 = Color32::from_rgb(0x92, 0x40, 0x0E);

pub const RADIUS: u8 = 12;

pub fn install(ctx: &egui::Context) {
    ctx.set_theme(egui::ThemePreference::Light);
    ctx.style_mut_of(egui::Theme::Light, |s| {
        use FontFamily::{Monospace, Proportional};
        s.text_styles = [
            (TextStyle::Small, FontId::new(12.0, Proportional)),
            (TextStyle::Body, FontId::new(14.5, Proportional)),
            (TextStyle::Button, FontId::new(14.5, Proportional)),
            (TextStyle::Heading, FontId::new(21.0, Proportional)),
            (TextStyle::Monospace, FontId::new(13.0, Monospace)),
        ]
        .into();
        let sp = &mut s.spacing;
        sp.item_spacing = Vec2::new(10.0, 9.0);
        sp.button_padding = Vec2::new(14.0, 7.0);
        sp.interact_size = Vec2::new(40.0, 30.0);
        sp.window_margin = Margin::same(16);
        sp.slider_width = 170.0;
        let v = &mut s.visuals;
        v.panel_fill = CANVAS;
        v.window_fill = CARD;
        v.extreme_bg_color = SUNKEN;
        v.selection.bg_fill = TEAL.gamma_multiply(0.35);
        v.selection.stroke = Stroke::new(1.0, TEAL);
        v.hyperlink_color = INDIGO;
        let r = CornerRadius::same(8);
        for w in [
            &mut v.widgets.noninteractive,
            &mut v.widgets.inactive,
            &mut v.widgets.hovered,
            &mut v.widgets.active,
            &mut v.widgets.open,
        ] {
            w.corner_radius = r;
        }
        v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, INK);
        v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, BORDER);
        v.widgets.inactive.bg_fill = SUNKEN;
        v.widgets.inactive.weak_bg_fill = SUNKEN;
        v.widgets.hovered.bg_stroke = Stroke::new(1.0, TEAL);
    });
}

/// A white card with a light border and round corners.
pub fn card() -> egui::Frame {
    egui::Frame::new()
        .fill(CARD)
        .stroke(Stroke::new(1.0, BORDER))
        .corner_radius(CornerRadius::same(RADIUS))
        .inner_margin(Margin::same(16))
}
