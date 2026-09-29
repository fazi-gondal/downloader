//! Empty state component for views with zero items.

use gpui_kit::component::{v_flex, Icon};
use crate::ui::prelude::*;

pub fn empty_state(
    icon: IconName,
    title: impl Into<SharedString>,
    subtitle: impl Into<SharedString>,
) -> impl IntoElement {
    v_flex()
        .w_full()
        .py_12()
        .items_center()
        .justify_center()
        .gap_3()
        .child(
            div()
                .p_4()
                .rounded_full()
                .bg(cx_or_default_bg())
                .child(Icon::new(icon).size(px(36.)).text_color(muted_fg())),
        )
        .child(
            div()
                .text_lg()
                .font_semibold()
                .text_color(fg())
                .child(title.into()),
        )
        .child(
            div()
                .text_sm()
                .text_color(muted_fg())
                .child(subtitle.into()),
        )
}

fn cx_or_default_bg() -> Hsla {
    crate::ui::theme::SURFACE_ELEVATED
}

fn fg() -> Hsla {
    crate::ui::theme::TEXT_PRIMARY
}

fn muted_fg() -> Hsla {
    crate::ui::theme::TEXT_MUTED
}
