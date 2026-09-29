//! Nocturnal Studio design system: colors, radii, tokens, and theme configuration.
//!
//! Tonal layering instead of Material shadows: hierarchy comes from stacking
//! surfaces (base -> elevated -> highest) separated by 1px subtle borders.
//! Coral (#FF5C49) is the primary accent (downloads / CTAs), teal (#32A097)
//! identifies the converter module and ffmpeg processing.

use gpui_kit::component::{Theme, ThemeMode};
use gpui_kit::*;

pub const CORAL: Hsla = hsla(6.0 / 360.0, 1.0, 0.645, 1.0); // #FF5C49
pub const CORAL_HOVER: Hsla = hsla(6.0 / 360.0, 1.0, 0.69, 1.0); // #FF7261
pub const CORAL_CONTAINER: Hsla = hsla(6.0 / 360.0, 1.0, 0.645, 0.15);

pub const TEAL: Hsla = hsla(175.0 / 360.0, 0.52, 0.41, 1.0); // #32A097
pub const TEAL_BRIGHT: Hsla = hsla(174.0 / 360.0, 0.56, 0.64, 1.0); // #71D7CD
pub const TEAL_CONTAINER: Hsla = hsla(175.0 / 360.0, 0.52, 0.41, 0.18);

pub const WINDOW_BG: Hsla = hsla(220.0 / 360.0, 0.17, 0.08, 1.0); // #111318
pub const SURFACE_BASE: Hsla = hsla(223.0 / 360.0, 0.14, 0.10, 1.0); // #16181D
pub const SURFACE_ELEVATED: Hsla = hsla(220.0 / 360.0, 0.13, 0.14, 1.0); // #1E2127
pub const SURFACE_HIGHEST: Hsla = hsla(222.0 / 360.0, 0.12, 0.19, 1.0); // #2A2D35
pub const SURFACE_INSET: Hsla = hsla(220.0 / 360.0, 0.14, 0.08, 1.0); // #121418

pub const BORDER_SUBTLE: Hsla = hsla(0.0, 0.0, 1.0, 0.08); // 8% white
pub const BORDER_STRONG: Hsla = hsla(0.0, 0.0, 1.0, 0.16); // 16% white

pub const TEXT_PRIMARY: Hsla = hsla(240.0 / 360.0, 0.15, 0.90, 1.0); // #E2E2E9
pub const TEXT_MUTED: Hsla = hsla(206.0 / 360.0, 0.06, 0.63, 1.0); // #9BA1A6

pub const PILL_GREEN: Hsla = hsla(122.0 / 360.0, 0.39, 0.49, 1.0); // #4CAF50
pub const PILL_RED: Hsla = hsla(359.0 / 360.0, 0.82, 0.63, 1.0); // #EF5350
pub const PILL_AMBER: Hsla = hsla(38.0 / 360.0, 0.92, 0.50, 1.0); // #F59E0B
pub const PILL_BLUE: Hsla = hsla(207.0 / 360.0, 0.90, 0.61, 1.0); // #42A5F5

pub const RADIUS_CARD: Pixels = px(16.0);
pub const RADIUS_CONTROL: Pixels = px(10.0);
pub const RADIUS_PILL: Pixels = px(999.0);

pub fn init_nocturnal_theme(cx: &mut App) {
    Theme::change(ThemeMode::Dark, None, cx);
    Theme::update(cx, |theme| {
        theme.primary = CORAL;
        theme.primary_foreground = TEXT_PRIMARY;
        theme.background = WINDOW_BG;
        theme.secondary = SURFACE_ELEVATED;
        theme.secondary_foreground = TEXT_PRIMARY;
        theme.muted = SURFACE_HIGHEST;
        theme.muted_foreground = TEXT_MUTED;
        theme.border = BORDER_SUBTLE;
        theme.sidebar = SURFACE_BASE;
        theme.sidebar_foreground = TEXT_PRIMARY;
        theme.sidebar_border = BORDER_SUBTLE;
        theme.sidebar_primary = CORAL;
        theme.radius = RADIUS_CONTROL;
    });
}
