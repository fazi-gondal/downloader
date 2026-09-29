//! Uppercase status pills with tinted backgrounds, per the Nocturnal Studio design system.

use gpui_kit::component::h_flex;
use crate::ui::prelude::*;
use crate::ui::theme::{
    CORAL, PILL_AMBER, PILL_GREEN, PILL_RED, RADIUS_PILL, TEAL_BRIGHT, TEXT_MUTED,
};
use crate::models::download::DownloadStatus;

pub fn status_pill(text: impl Into<SharedString>, color: Hsla) -> impl IntoElement {
    let mut bg = color;
    bg.a = 0.15;
    let mut border = color;
    border.a = 0.30;

    h_flex()
        .px_2()
        .py(px(2.))
        .rounded(RADIUS_PILL)
        .bg(bg)
        .border_1()
        .border_color(border)
        .items_center()
        .child(
            div()
                .text_xs()
                .font_semibold()
                .text_color(color)
                .child(text.into().to_uppercase()),
        )
}

pub fn download_status_pill(status: DownloadStatus) -> impl IntoElement {
    match status {
        DownloadStatus::Queued => status_pill("Queued", TEXT_MUTED),
        DownloadStatus::Analyzing => status_pill("Analyzing", PILL_AMBER),
        DownloadStatus::Downloading => status_pill("Downloading", CORAL),
        DownloadStatus::Processing => status_pill("Processing", TEAL_BRIGHT),
        DownloadStatus::Completed => status_pill("Completed", PILL_GREEN),
        DownloadStatus::Failed => status_pill("Error", PILL_RED),
        DownloadStatus::Cancelled => status_pill("Cancelled", TEXT_MUTED),
    }
}
