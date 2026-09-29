//! About view + dependency inspector (yt-dlp / FFmpeg).
//!
//! CLI tool availability is checked automatically when the view is first
//! created (on app open) and again whenever the user clicks "Check Status".
//! Both checks run on the background thread so the UI never blocks.

use gpui_kit::component::button::Button;
use gpui_kit::component::{h_flex, v_flex, Icon};
use crate::ui::prelude::*;

use crate::services::{FfmpegService, YtDlpService};

pub struct AboutView {
    ytdlp_status: Option<String>,
    ffmpeg_status: Option<String>,
    checking: bool,
}

impl AboutView {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        // Kick off auto-check immediately so the status shows on first open
        // without requiring the user to click "Check Status".
        cx.spawn(async move |this, cx| {
            // Run both checks concurrently on the background thread.
            let (ytdlp_result, ffmpeg_result) = cx
                .background_spawn(async move {
                    let ytdlp = match YtDlpService::default().check_available() {
                        Ok(v) => v,
                        Err(e) => format!("Missing ({})", e),
                    };
                    let ffmpeg = match FfmpegService::default().check_available() {
                        Ok(v) => v,
                        Err(e) => format!("Missing ({})", e),
                    };
                    (ytdlp, ffmpeg)
                })
                .await;

            let _ = this.update(cx, |view, cx| {
                view.ytdlp_status = Some(ytdlp_result);
                view.ffmpeg_status = Some(ffmpeg_result);
                view.checking = false;
                cx.notify();
            });
        })
        .detach();

        Self {
            ytdlp_status: None,
            ffmpeg_status: None,
            checking: true, // show spinner while auto-check runs
        }
    }

    /// Trigger a fresh async check (used by the "Check Status" button).
    fn refresh_deps(&mut self, cx: &mut Context<Self>) {
        if self.checking {
            return; // already running
        }
        self.checking = true;
        cx.notify();

        cx.spawn(async move |this, cx| {
            let (ytdlp_result, ffmpeg_result) = cx
                .background_spawn(async move {
                    let ytdlp = match YtDlpService::default().check_available() {
                        Ok(v) => v,
                        Err(e) => format!("Missing ({})", e),
                    };
                    let ffmpeg = match FfmpegService::default().check_available() {
                        Ok(v) => v,
                        Err(e) => format!("Missing ({})", e),
                    };
                    (ytdlp, ffmpeg)
                })
                .await;

            let _ = this.update(cx, |view, cx| {
                view.ytdlp_status = Some(ytdlp_result);
                view.ffmpeg_status = Some(ffmpeg_result);
                view.checking = false;
                cx.notify();
            });
        })
        .detach();
    }

    fn open_url(url: &str) {
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            let _ = std::process::Command::new("cmd")
                .args(["/c", "start", "", url])
                .creation_flags(CREATE_NO_WINDOW)
                .spawn();
        }
        #[cfg(not(windows))]
        {
            let _ = std::process::Command::new("xdg-open").arg(url).spawn();
        }
    }
}

impl Render for AboutView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let ytdlp_str = self
            .ytdlp_status
            .clone()
            .unwrap_or_else(|| "Checking…".into());
        let ffmpeg_str = self
            .ffmpeg_status
            .clone()
            .unwrap_or_else(|| "Checking…".into());

        let ytdlp_ok = self.ytdlp_status.as_ref().map(|s| !s.starts_with("Missing")).unwrap_or(false);
        let ffmpeg_ok = self.ffmpeg_status.as_ref().map(|s| !s.starts_with("Missing")).unwrap_or(false);

        v_flex()
            .size_full()
            .p_6()
            .gap_4()
            .bg(cx.theme().background)
            .overflow_y_scrollbar()
            // App Branding Card
            .child(
                v_flex()
                    .w_full()
                    .p_6()
                    .gap_3()
                    .rounded(cx.theme().radius)
                    .bg(cx.theme().secondary)
                    .border_1()
                    .border_color(cx.theme().border)
                    .child(
                        h_flex()
                            .gap_3()
                            .items_center()
                            .child(
                                div()
                                    .p_3()
                                    .rounded(px(10.))
                                    .bg(cx.theme().primary)
                                    .child(Icon::new(IconName::Download).size(px(28.)).text_color(cx.theme().background)),
                            )
                            .child(
                                v_flex()
                                    .gap_0p5()
                                    .child(
                                        div()
                                            .text_2xl()
                                            .font_bold()
                                            .text_color(cx.theme().foreground)
                                            .child("Video Downloader"),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(cx.theme().muted_foreground)
                                            .child("High-Performance Desktop Media Downloader & Transcoder"),
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child("Rewritten in pure Rust with GPUI-kit, delivering native 120 FPS performance, fine-grained resolution & audio selection, format exploration, lossless remuxing, and multi-threaded yt-dlp queuing."),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .pt_2()
                            .child(
                                Button::new("gh-repo-btn")
                                    .outline()
                                    .icon(IconName::Globe)
                                    .label("GitHub Repository")
                                    .on_click(cx.listener(|_, _, _, _| {
                                        Self::open_url("https://github.com/fazi-gondal/downloader");
                                    })),
                            ),
                    ),
            )
            // System Dependencies Card
            .child(
                v_flex()
                    .gap_3()
                    .p_5()
                    .rounded(cx.theme().radius)
                    .border_1()
                    .border_color(cx.theme().border)
                    .bg(cx.theme().secondary)
                    .child(
                        h_flex()
                            .w_full()
                            .justify_between()
                            .items_center()
                            .child(
                                v_flex()
                                    .gap_0p5()
                                    .child(
                                        div()
                                            .font_semibold()
                                            .text_base()
                                            .text_color(cx.theme().foreground)
                                            .child("Runtime CLI Tools"),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(cx.theme().muted_foreground)
                                            .child(if self.checking {
                                                "Checking availability of CLI engines…"
                                            } else {
                                                "Underlying engines required for extraction and transcoding"
                                            }),
                                    ),
                            )
                            .child(
                                Button::new("check-deps-btn")
                                    .primary()
                                    .icon(IconName::RefreshCw)
                                    .label(if self.checking { "Checking…" } else { "Check Status" })
                                    .disabled(self.checking)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.refresh_deps(cx);
                                    })),
                            ),
                    )
                    // yt-dlp item
                    .child(
                        h_flex()
                            .w_full()
                            .p_3()
                            .gap_3()
                            .items_center()
                            .rounded(cx.theme().radius)
                            .bg(cx.theme().background)
                            .border_1()
                            .border_color(cx.theme().border)
                            .child(
                                div()
                                    .p_2()
                                    .rounded(px(6.))
                                    .bg(if self.checking {
                                        cx.theme().muted
                                    } else if ytdlp_ok {
                                        cx.theme().success.opacity(0.15)
                                    } else {
                                        cx.theme().secondary
                                    })
                                    .child(
                                        Icon::new(if self.checking {
                                            IconName::LoaderCircle
                                        } else if ytdlp_ok {
                                            IconName::Check
                                        } else {
                                            IconName::CircleAlert
                                        })
                                        .size(px(18.))
                                        .text_color(if self.checking {
                                            cx.theme().muted_foreground
                                        } else if ytdlp_ok {
                                            cx.theme().success
                                        } else {
                                            cx.theme().danger
                                        }),
                                    ),
                            )
                            .child(
                                v_flex()
                                    .flex_1()
                                    .gap_0p5()
                                    .child(
                                        div()
                                            .font_semibold()
                                            .text_sm()
                                            .text_color(cx.theme().foreground)
                                            .child("yt-dlp"),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(if ytdlp_ok { cx.theme().foreground } else { cx.theme().muted_foreground })
                                            .child(ytdlp_str),
                                    ),
                            )
                            .child(
                                Button::new("ytdlp-get-btn")
                                    .ghost()
                                    .icon(IconName::Globe)
                                    .label("yt-dlp.org")
                                    .on_click(cx.listener(|_, _, _, _| {
                                        Self::open_url("https://github.com/yt-dlp/yt-dlp");
                                    })),
                            ),
                    )
                    // FFmpeg item
                    .child(
                        h_flex()
                            .w_full()
                            .p_3()
                            .gap_3()
                            .items_center()
                            .rounded(cx.theme().radius)
                            .bg(cx.theme().background)
                            .border_1()
                            .border_color(cx.theme().border)
                            .child(
                                div()
                                    .p_2()
                                    .rounded(px(6.))
                                    .bg(if self.checking {
                                        cx.theme().muted
                                    } else if ffmpeg_ok {
                                        cx.theme().success.opacity(0.15)
                                    } else {
                                        cx.theme().secondary
                                    })
                                    .child(
                                        Icon::new(if self.checking {
                                            IconName::LoaderCircle
                                        } else if ffmpeg_ok {
                                            IconName::Check
                                        } else {
                                            IconName::CircleAlert
                                        })
                                        .size(px(18.))
                                        .text_color(if self.checking {
                                            cx.theme().muted_foreground
                                        } else if ffmpeg_ok {
                                            cx.theme().success
                                        } else {
                                            cx.theme().danger
                                        }),
                                    ),
                            )
                            .child(
                                v_flex()
                                    .flex_1()
                                    .gap_0p5()
                                    .child(
                                        div()
                                            .font_semibold()
                                            .text_sm()
                                            .text_color(cx.theme().foreground)
                                            .child("FFmpeg"),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(if ffmpeg_ok { cx.theme().foreground } else { cx.theme().muted_foreground })
                                            .child(ffmpeg_str),
                                    ),
                            )
                            .child(
                                Button::new("ffmpeg-get-btn")
                                    .ghost()
                                    .icon(IconName::Globe)
                                    .label("ffmpeg.org")
                                    .on_click(cx.listener(|_, _, _, _| {
                                        Self::open_url("https://ffmpeg.org/download.html");
                                    })),
                            ),
                    ),
            )
            // Tech Stack & Architecture Card
            .child(
                v_flex()
                    .gap_2()
                    .p_4()
                    .rounded(cx.theme().radius)
                    .border_1()
                    .border_color(cx.theme().border)
                    .bg(cx.theme().secondary)
                    .child(
                        div()
                            .font_semibold()
                            .text_sm()
                            .text_color(cx.theme().foreground)
                            .child("Architecture & Libraries"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child("• UI Framework: Zed GPUI + GPUI-kit (GPU-accelerated, immediate-mode desktop framework)\n• Video Extraction: yt-dlp CLI via asynchronous non-blocking process worker pool\n• Transcoding: FFmpeg via standard pipes with clean process-tree termination\n• System Controls: Native OS window frame with hardware minimize/maximize/close"),
                    ),
            )
    }
}
