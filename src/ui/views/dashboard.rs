//! Dashboard: URL input and media analysis entry point.
//!
//! Designed with Nocturnal Studio aesthetics: centered hero state,
//! sleek search bar, supported services display, and media cards on result.

use gpui_kit::component::button::Button;
use gpui_kit::component::input::{Input, InputState};
use gpui_kit::component::{h_flex, v_flex, Icon};
use crate::ui::prelude::*;

use crate::core::{AppRoute, AppState, CurrentMedia};
use crate::services::ytdlp::ExtractedInfo;
use crate::ui::components::media_card::{playlist_media_card, single_media_card};
use crate::ui::theme::{
    BORDER_SUBTLE, CORAL, RADIUS_CARD, RADIUS_CONTROL, SURFACE_ELEVATED, TEXT_MUTED,
    TEXT_PRIMARY,
};

pub struct DashboardView {
    state: Entity<AppState>,
    url_input: Entity<InputState>,
}

impl DashboardView {
    pub fn new(window: &mut Window, state: Entity<AppState>, cx: &mut Context<Self>) -> Self {
        let url_input = cx.new(|cx| {
            InputState::new(window, cx).placeholder("Paste a link and go…")
        });
        Self { state, url_input }
    }

    fn start_analysis(&mut self, cx: &mut Context<Self>) {
        let url = self.url_input.read(cx).value().trim().to_string();
        if url.is_empty() {
            self.state.update(cx, |s, cx| {
                s.set_error("Please enter a URL");
                cx.notify();
            });
            return;
        }
        if !(url.starts_with("http://") || url.starts_with("https://")) {
            self.state.update(cx, |s, cx| {
                s.set_error("URL must start with http:// or https://");
                cx.notify();
            });
            return;
        }

        self.state.update(cx, |s, cx| {
            s.analyzing = true;
            s.last_error = None;
            cx.notify();
        });

        let state = self.state.clone();
        let ytdlp = self.state.read(cx).ytdlp.clone();

        cx.spawn(async move |_, cx| {
            let result = cx
                .background_spawn(async move { ytdlp.extract_info(&url) })
                .await;

            state.update(cx, |s, cx| {
                match result {
                    Ok(ExtractedInfo::Single(media)) => s.set_single(media),
                    Ok(ExtractedInfo::Playlist(pl)) => s.set_playlist(pl),
                    Err(e) => s.set_error(e.to_string()),
                }
                cx.notify();
            });
        })
        .detach();
    }
}

impl Render for DashboardView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (analyzing, last_error, current_media) = {
            let s = self.state.read(cx);
            (s.analyzing, s.last_error.clone(), s.current_media.clone())
        };

        let has_media = !matches!(current_media, CurrentMedia::None);

        // Search Bar Widget
        let search_bar = h_flex()
            .w_full()
            .max_w(px(640.))
            .p_1p5()
            .rounded(RADIUS_CARD)
            .bg(SURFACE_ELEVATED)
            .border_1()
            .border_color(BORDER_SUBTLE)
            .items_center()
            .gap_2()
            .child(
                div()
                    .pl_3()
                    .child(Icon::new(IconName::Link).size(px(18.)).text_color(TEXT_MUTED)),
            )
            .child(div().flex_1().child(Input::new(&self.url_input)))
            .child(
                Button::new("analyze")
                    .primary()
                    .label(if analyzing { "Analyzing…" } else { "Analyze" })
                    .icon(IconName::ArrowRight)
                    .disabled(analyzing)
                    .loading(analyzing)
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.start_analysis(cx);
                    })),
            );

        if !has_media && !analyzing && last_error.is_none() {
            // HERO STATE (Vertically Centered)
            v_flex()
                .size_full()
                .items_center()
                .justify_center()
                .p_8()
                .gap_8()
                .child(
                    v_flex()
                        .items_center()
                        .gap_3()
                        .child(
                            h_flex()
                                .items_center()
                                .gap_1()
                                .child(
                                    div()
                                        .text_3xl()
                                        .font_bold()
                                        .text_color(TEXT_PRIMARY)
                                        .child("Extract. Convert. "),
                                )
                                .child(
                                    div()
                                        .text_3xl()
                                        .font_bold()
                                        .text_color(CORAL)
                                        .child("Master."),
                                ),
                        )
                        .child(
                            div()
                                .max_w(px(520.))
                                .text_center()
                                .text_sm()
                                .text_color(TEXT_MUTED)
                                .child("The ultimate media capture suite. Paste a link from any supported platform to start downloading and processing."),
                        ),
                )
                .child(search_bar)
                .child(
                    v_flex()
                        .items_center()
                        .gap_3()
                        .child(
                            div()
                                .text_xs()
                                .font_semibold()
                                .text_color(TEXT_MUTED)
                                .child("SUPPORTED SERVICES"),
                        )
                        .child(
                            h_flex()
                                .gap_6()
                                .items_center()
                                .child(
                                    h_flex()
                                        .gap_1p5()
                                        .items_center()
                                        .child(Icon::new(IconName::Tv).size(px(16.)).text_color(TEXT_MUTED))
                                        .child(div().text_sm().text_color(TEXT_MUTED).child("YouTube")),
                                )
                                .child(
                                    h_flex()
                                        .gap_1p5()
                                        .items_center()
                                        .child(Icon::new(IconName::Music).size(px(16.)).text_color(TEXT_MUTED))
                                        .child(div().text_sm().text_color(TEXT_MUTED).child("TikTok")),
                                )
                                .child(
                                    h_flex()
                                        .gap_1p5()
                                        .items_center()
                                        .child(Icon::new(IconName::Camera).size(px(16.)).text_color(TEXT_MUTED))
                                        .child(div().text_sm().text_color(TEXT_MUTED).child("Instagram")),
                                )
                                .child(
                                    h_flex()
                                        .gap_1p5()
                                        .items_center()
                                        .child(Icon::new(IconName::Globe).size(px(16.)).text_color(TEXT_MUTED))
                                        .child(div().text_sm().text_color(TEXT_MUTED).child("Web Streams")),
                                ),
                        ),
                )
        } else {
            // ACTIVE / RESULTS STATE (Search bar on top)
            v_flex()
                .size_full()
                .p_8()
                .gap_6()
                .items_center()
                .child(search_bar)
                .when(analyzing, |this| {
                    this.child(
                        h_flex()
                            .p_4()
                            .rounded(RADIUS_CONTROL)
                            .bg(SURFACE_ELEVATED)
                            .border_1()
                            .border_color(BORDER_SUBTLE)
                            .items_center()
                            .gap_3()
                            .child(Icon::new(IconName::LoaderCircle).size(px(20.)).text_color(CORAL))
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(TEXT_PRIMARY)
                                    .child("Analyzing media streams… (fetching metadata via yt-dlp)"),
                            ),
                    )
                })
                .when_some(last_error, |this, err| {
                    let mut bg = CORAL;
                    bg.a = 0.12;
                    let mut border = CORAL;
                    border.a = 0.35;
                    this.child(
                        h_flex()
                            .w_full()
                            .max_w(px(640.))
                            .p_4()
                            .rounded(RADIUS_CONTROL)
                            .bg(bg)
                            .border_1()
                            .border_color(border)
                            .items_center()
                            .gap_3()
                            .child(Icon::new(IconName::CircleAlert).size(px(20.)).text_color(CORAL))
                            .child(
                                div()
                                    .text_sm()
                                    .font_medium()
                                    .text_color(CORAL)
                                    .child(err),
                            ),
                    )
                })
                .when(!analyzing, |this| {
                    match current_media {
                        CurrentMedia::None => this,
                        CurrentMedia::Single(media) => {
                            this.child(
                                v_flex()
                                    .w_full()
                                    .max_w(px(760.))
                                    .gap_4()
                                    .child(single_media_card(&media))
                                    .child(
                                        h_flex()
                                            .w_full()
                                            .justify_between()
                                            .items_center()
                                            .child(
                                                Button::new("explore-formats")
                                                    .outline()
                                                    .icon(IconName::Layers)
                                                    .label("View Available Formats")
                                                    .on_click(cx.listener(|this, _, _, cx| {
                                                        this.state.update(cx, |s, cx| {
                                                            s.active_route = AppRoute::Config;
                                                            cx.notify();
                                                        });
                                                    })),
                                            )
                                            .child(
                                                Button::new("continue")
                                                    .primary()
                                                    .icon(IconName::ArrowRight)
                                                    .label("Continue to Configure")
                                                    .on_click(cx.listener(|this, _, _, cx| {
                                                        this.state.update(cx, |s, cx| {
                                                            s.active_route = AppRoute::Config;
                                                            cx.notify();
                                                        });
                                                    })),
                                            ),
                                    ),
                            )
                        }
                        CurrentMedia::Playlist(playlist) => {
                            this.child(
                                v_flex()
                                    .w_full()
                                    .max_w(px(760.))
                                    .gap_4()
                                    .child(playlist_media_card(&playlist))
                                    .child(
                                        h_flex()
                                            .w_full()
                                            .justify_end()
                                            .child(
                                                Button::new("continue-pl")
                                                    .primary()
                                                    .icon(IconName::ArrowRight)
                                                    .label("Continue to Configure")
                                                    .on_click(cx.listener(|this, _, _, cx| {
                                                        this.state.update(cx, |s, cx| {
                                                            s.active_route = AppRoute::Config;
                                                            cx.notify();
                                                        });
                                                    })),
                                            ),
                                    ),
                            )
                        }
                    }
                })
        }
    }
}
