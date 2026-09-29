//! Card showing analyzed media metadata (thumbnail preview, title, author, duration).

use gpui_kit::component::{h_flex, v_flex, Icon};
use crate::ui::prelude::*;
use crate::models::media::{MediaInfo, PlaylistInfo};
use crate::ui::theme::{
    BORDER_SUBTLE, RADIUS_CARD, RADIUS_CONTROL, RADIUS_PILL, SURFACE_ELEVATED,
    SURFACE_HIGHEST, TEXT_PRIMARY, TEXT_MUTED,
};

fn meta_pill(icon: IconName, text: impl Into<SharedString>) -> impl IntoElement {
    h_flex()
        .px_3()
        .py_1()
        .rounded(RADIUS_PILL)
        .bg(SURFACE_HIGHEST)
        .gap_1p5()
        .items_center()
        .child(Icon::new(icon).size(px(14.)).text_color(TEXT_MUTED))
        .child(
            div()
                .text_xs()
                .font_medium()
                .text_color(TEXT_MUTED)
                .child(text.into()),
        )
}

pub fn single_media_card(media: &MediaInfo) -> impl IntoElement {
    let duration_str = if let Some(secs) = media.duration {
        let total_secs = secs.round() as u64;
        let h = total_secs / 3600;
        let m = (total_secs % 3600) / 60;
        let s = total_secs % 60;
        if h > 0 {
            format!("{h:02}:{m:02}:{s:02}")
        } else {
            format!("{m:02}:{s:02}")
        }
    } else {
        "Unknown".to_string()
    };

    h_flex()
        .w_full()
        .p_4()
        .rounded(RADIUS_CARD)
        .bg(SURFACE_ELEVATED)
        .border_1()
        .border_color(BORDER_SUBTLE)
        .gap_4()
        .items_center()
        .child(
            // 16:9 Thumbnail Box
            div()
                .w(px(180.))
                .h(px(101.))
                .rounded(RADIUS_CONTROL)
                .bg(SURFACE_HIGHEST)
                .flex()
                .items_center()
                .justify_center()
                .child(Icon::new(IconName::Film).size(px(36.)).text_color(TEXT_MUTED)),
        )
        .child(
            v_flex()
                .flex_1()
                .gap_3()
                .justify_center()
                .child(
                    div()
                        .text_lg()
                        .font_bold()
                        .text_color(TEXT_PRIMARY)
                        .child(media.title.clone()),
                )
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .when_some(media.uploader.as_ref(), |this, uploader| {
                            this.child(meta_pill(IconName::User, uploader.clone()))
                        })
                        .child(meta_pill(IconName::Clock, duration_str))
                        .child(meta_pill(
                            IconName::Layers,
                            format!("{} formats", media.formats.len()),
                        )),
                ),
        )
}

pub fn playlist_media_card(playlist: &PlaylistInfo) -> impl IntoElement {
    h_flex()
        .w_full()
        .p_4()
        .rounded(RADIUS_CARD)
        .bg(SURFACE_ELEVATED)
        .border_1()
        .border_color(BORDER_SUBTLE)
        .gap_4()
        .items_center()
        .child(
            div()
                .w(px(180.))
                .h(px(101.))
                .rounded(RADIUS_CONTROL)
                .bg(SURFACE_HIGHEST)
                .flex()
                .items_center()
                .justify_center()
                .child(Icon::new(IconName::ListVideo).size(px(36.)).text_color(TEXT_MUTED)),
        )
        .child(
            v_flex()
                .flex_1()
                .gap_3()
                .justify_center()
                .child(
                    div()
                        .text_lg()
                        .font_bold()
                        .text_color(TEXT_PRIMARY)
                        .child(playlist.title.clone()),
                )
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .child(meta_pill(
                            IconName::ListOrdered,
                            format!("{} videos found", playlist.entries.len()),
                        ))
                        .when_some(playlist.uploader.as_ref(), |this, uploader| {
                            this.child(meta_pill(IconName::User, uploader.clone()))
                        }),
                ),
        )
}
