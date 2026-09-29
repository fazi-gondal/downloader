//! Config view: mode, resolution ladder, container, fps, audio format/quality,
//! subtitle modes, multi-audio, playlist picker, and format explorer.

use gpui_kit::component::button::Button;
use gpui_kit::component::checkbox::Checkbox;
use gpui_kit::component::{h_flex, v_flex, Icon};
use crate::ui::prelude::*;

use crate::core::{AppState, CurrentMedia};
use crate::models::{
    AudioFormat, AudioQualityPreset, AudioTracksMode, Container, DownloadMode, ResolutionPreset, StreamType,
    SubtitleMode,
};
use crate::services::FormatBuilder;


use gpui_kit::component::scroll::{Scrollbar, ScrollbarMode};
use gpui_kit::ScrollHandle;

pub struct ConfigView {
    state: Entity<AppState>,
    show_format_explorer: bool,
    subtitles_scroll_handle: ScrollHandle,
    audio_scroll_handle: ScrollHandle,
}

impl ConfigView {
    pub fn new(_window: &mut Window, state: Entity<AppState>, _cx: &mut Context<Self>) -> Self {
        Self {
            state,
            show_format_explorer: false,
            subtitles_scroll_handle: ScrollHandle::default(),
            audio_scroll_handle: ScrollHandle::default(),
        }
    }

    fn set_mode(&mut self, mode: DownloadMode, cx: &mut Context<Self>) {
        self.state.update(cx, |s, cx| {
            s.current_options.mode = mode;
            cx.notify();
        });
    }

    fn set_resolution(&mut self, res: ResolutionPreset, cx: &mut Context<Self>) {
        self.state.update(cx, |s, cx| {
            s.current_options.resolution = res;
            // Clear manual video format override when choosing a preset
            s.current_options.video_format_id = None;
            cx.notify();
        });
    }

    fn set_container(&mut self, container: Container, cx: &mut Context<Self>) {
        self.state.update(cx, |s, cx| {
            s.current_options.container = container;
            cx.notify();
        });
    }

    fn set_fps(&mut self, fps: Option<u32>, cx: &mut Context<Self>) {
        self.state.update(cx, |s, cx| {
            s.current_options.fps_filter = fps;
            cx.notify();
        });
    }

    fn set_audio_format(&mut self, format: AudioFormat, cx: &mut Context<Self>) {
        self.state.update(cx, |s, cx| {
            s.current_options.audio_format = format;
            cx.notify();
        });
    }

    fn set_audio_quality(&mut self, quality: AudioQualityPreset, cx: &mut Context<Self>) {
        self.state.update(cx, |s, cx| {
            s.current_options.audio_quality = quality;
            cx.notify();
        });
    }

    fn set_subtitle_mode(&mut self, mode: SubtitleMode, cx: &mut Context<Self>) {
        self.state.update(cx, |s, cx| {
            s.current_options.subtitle_mode = mode;
            cx.notify();
        });
    }

    fn toggle_subtitle_lang(&mut self, lang: String, cx: &mut Context<Self>) {
        self.state.update(cx, |s, cx| {
            if let Some(pos) = s.current_options.subtitle_langs.iter().position(|l| l == &lang) {
                s.current_options.subtitle_langs.remove(pos);
            } else {
                s.current_options.subtitle_langs.push(lang);
            }
            cx.notify();
        });
    }

    fn select_all_subtitles(&mut self, langs: Vec<String>, cx: &mut Context<Self>) {
        self.state.update(cx, |s, cx| {
            s.current_options.subtitle_langs = langs;
            cx.notify();
        });
    }

    fn clear_subtitles(&mut self, cx: &mut Context<Self>) {
        self.state.update(cx, |s, cx| {
            s.current_options.subtitle_langs.clear();
            cx.notify();
        });
    }

    fn set_audio_tracks_mode(&mut self, mode: AudioTracksMode, cx: &mut Context<Self>) {
        self.state.update(cx, |s, cx| {
            s.current_options.audio_tracks_mode = mode;
            match mode {
                AudioTracksMode::Default => {
                    s.current_options.multi_audio = false;
                    s.current_options.selected_audio_langs.clear();
                }
                AudioTracksMode::All => {
                    s.current_options.multi_audio = true;
                    s.current_options.selected_audio_langs.clear();
                }
                AudioTracksMode::Custom => {
                    s.current_options.multi_audio = false;
                }
            }
            cx.notify();
        });
    }

    fn select_all_audio_tracks(&mut self, tracks: Vec<String>, cx: &mut Context<Self>) {
        self.state.update(cx, |s, cx| {
            s.current_options.selected_audio_langs = tracks;
            cx.notify();
        });
    }

    fn clear_audio_tracks(&mut self, cx: &mut Context<Self>) {
        self.state.update(cx, |s, cx| {
            s.current_options.selected_audio_langs.clear();
            cx.notify();
        });
    }

    fn toggle_multi_audio(&mut self, checked: bool, cx: &mut Context<Self>) {
        self.state.update(cx, |s, cx| {
            s.current_options.multi_audio = checked;
            cx.notify();
        });
    }

    fn toggle_audio_lang(&mut self, lang: String, cx: &mut Context<Self>) {
        self.state.update(cx, |s, cx| {
            if let Some(pos) = s.current_options.selected_audio_langs.iter().position(|l| l == &lang) {
                s.current_options.selected_audio_langs.remove(pos);
            } else {
                s.current_options.selected_audio_langs.push(lang);
            }
            cx.notify();
        });
    }

    fn toggle_embed_thumbnail(&mut self, checked: bool, cx: &mut Context<Self>) {
        self.state.update(cx, |s, cx| {
            s.current_options.embed_thumbnail = checked;
            cx.notify();
        });
    }

    fn toggle_embed_metadata(&mut self, checked: bool, cx: &mut Context<Self>) {
        self.state.update(cx, |s, cx| {
            s.current_options.embed_metadata = checked;
            cx.notify();
        });
    }

    fn toggle_format_explorer(&mut self, cx: &mut Context<Self>) {
        self.show_format_explorer = !self.show_format_explorer;
        cx.notify();
    }

    fn select_raw_stream(&mut self, format_id: String, is_video: bool, cx: &mut Context<Self>) {
        self.state.update(cx, |s, cx| {
            if is_video {
                if s.current_options.video_format_id.as_deref() == Some(&format_id) {
                    s.current_options.video_format_id = None;
                } else {
                    s.current_options.video_format_id = Some(format_id);
                }
            } else {
                if s.current_options.audio_format_id.as_deref() == Some(&format_id) {
                    s.current_options.audio_format_id = None;
                } else {
                    s.current_options.audio_format_id = Some(format_id);
                }
            }
            cx.notify();
        });
    }

    fn reset_custom_formats(&mut self, cx: &mut Context<Self>) {
        self.state.update(cx, |s, cx| {
            s.current_options.video_format_id = None;
            s.current_options.audio_format_id = None;
            cx.notify();
        });
    }

    fn toggle_playlist_entry(&mut self, index: usize, cx: &mut Context<Self>) {
        self.state.update(cx, |s, cx| {
            s.toggle_playlist_entry(index);
            cx.notify();
        });
    }

    fn select_all_playlist(&mut self, selected: bool, cx: &mut Context<Self>) {
        self.state.update(cx, |s, cx| {
            s.select_all_playlist(selected);
            cx.notify();
        });
    }

    fn estimated_size_label(&self, cx: &Context<Self>) -> String {
        let s = self.state.read(cx);
        match &s.current_media {
            CurrentMedia::Single(media) => {
                let size = FormatBuilder::compile(media, &s.current_options).estimated_size;
                FormatBuilder::format_size(size)
            }
            _ => "—".into(),
        }
    }

    fn start_download(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let mut count = 0usize;
        self.state.update(cx, |s, cx| {
            match &s.current_media {
                CurrentMedia::Single(media) => {
                    let url = media
                        .webpage_url
                        .clone()
                        .unwrap_or_else(|| format!("https://www.youtube.com/watch?v={}", media.id));
                    s.downloads
                        .enqueue(url, s.current_options.clone(), Some(media));
                    count = 1;
                }
                CurrentMedia::Playlist(pl) => {
                    for entry in pl.entries.iter().filter(|e| e.selected) {
                        s.downloads.enqueue_playlist_item(
                            entry.url.clone(),
                            entry.title.clone(),
                            Some(pl.title.clone()),
                            s.current_options.clone(),
                        );
                        count += 1;
                    }
                }
                CurrentMedia::None => {}
            }
            if count > 0 {
                s.active_route = crate::core::AppRoute::Downloads;
            }
            cx.notify();
        });
        if count > 0 {
            window.push_notification(
                format!(
                    "Queued {count} download{}",
                    if count == 1 { "" } else { "s" }
                ),
                cx,
            );
        }
    }
}

impl Render for ConfigView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let s = self.state.read(cx);
        let opts = s.current_options.clone();
        let media = s.current_media.clone();

        let has_media = !matches!(media, CurrentMedia::None);
        let size_label = self.estimated_size_label(cx);
        let show_explorer = self.show_format_explorer;

        v_flex()
            .size_full()
            .p_6()
            .gap_5()
            .bg(cx.theme().background)
            .overflow_y_scrollbar()
            // Header
            .child(
                v_flex()
                    .gap_1()
                    .child(
                        div()
                            .text_2xl()
                            .font_semibold()
                            .text_color(cx.theme().foreground)
                            .child("Configure Download"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child("Select output formats, container, resolution, and streams"),
                    ),
            )
            // Media Card (Single or Playlist)
            .child(match &media {
                CurrentMedia::Single(m) => {
                    let duration = FormatBuilder::format_duration(m.duration);
                    let uploader = m.uploader.as_deref().unwrap_or("Unknown channel");
                    let formats_count = m.formats.len();
                    let subs_count = m.subtitles.len();
                    let audio_count = m.audio_tracks.len();

                    h_flex()
                        .w_full()
                        .p_4()
                        .gap_4()
                        .items_center()
                        .rounded(cx.theme().radius)
                        .border_1()
                        .border_color(cx.theme().border)
                        .bg(cx.theme().secondary)
                        // Thumbnail or placeholder
                        .child(
                            div()
                                .w(px(140.))
                                .h(px(80.))
                                .rounded(px(4.))
                                .overflow_hidden()
                                .bg(cx.theme().muted)
                                .items_center()
                                .justify_center()
                                .child(Icon::new(IconName::Film).large().text_color(cx.theme().muted_foreground)),
                        )
                        // Media Info Details
                        .child(
                            v_flex()
                                .flex_1()
                                .gap_1()
                                .child(
                                    div()
                                        .text_base()
                                        .font_semibold()
                                        .text_color(cx.theme().foreground)
                                        .child(m.title.clone()),
                                )
                                .child(
                                    h_flex()
                                        .gap_3()
                                        .text_sm()
                                        .child(div().text_color(cx.theme().muted_foreground).child(uploader.to_string()))
                                        .child(div().text_color(cx.theme().muted_foreground).child("·"))
                                        .child(div().text_color(cx.theme().muted_foreground).child(format!("Duration: {duration}"))),
                                )
                                .child(
                                    h_flex()
                                        .gap_2()
                                        .pt_1()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child(div().p_1().rounded(px(3.)).bg(cx.theme().background).child(format!("{formats_count} streams")))
                                        .child(div().p_1().rounded(px(3.)).bg(cx.theme().background).child(format!("{audio_count} audio tracks")))
                                        .child(div().p_1().rounded(px(3.)).bg(cx.theme().background).child(format!("{subs_count} subtitles"))),
                                ),
                        )
                        .into_any_element()
                }
                CurrentMedia::Playlist(p) => {
                    let sel = p.entries.iter().filter(|e| e.selected).count();
                    let total = p.entries.len();
                    let playlist_entries: Vec<(usize, String, bool)> = p
                        .entries
                        .iter()
                        .enumerate()
                        .map(|(i, e)| (i, e.title.clone(), e.selected))
                        .collect();

                    v_flex()
                        .w_full()
                        .p_4()
                        .gap_3()
                        .rounded(cx.theme().radius)
                        .border_1()
                        .border_color(cx.theme().border)
                        .bg(cx.theme().secondary)
                        .child(
                            h_flex()
                                .justify_between()
                                .items_center()
                                .child(
                                    v_flex()
                                        .gap_1()
                                        .child(
                                            div()
                                                .font_semibold()
                                                .text_color(cx.theme().foreground)
                                                .child(p.title.clone()),
                                        )
                                        .child(
                                            div()
                                                .text_sm()
                                                .text_color(cx.theme().muted_foreground)
                                                .child(format!("Playlist · {sel}/{total} items selected")),
                                        ),
                                )
                                .child(
                                    h_flex()
                                        .gap_2()
                                        .child(
                                            Button::new("pl-all")
                                                .ghost()
                                                .small()
                                                .label("Select All")
                                                .on_click(cx.listener(|this, _, _, cx| {
                                                    this.select_all_playlist(true, cx);
                                                })),
                                        )
                                        .child(
                                            Button::new("pl-none")
                                                .ghost()
                                                .small()
                                                .label("Deselect All")
                                                .on_click(cx.listener(|this, _, _, cx| {
                                                    this.select_all_playlist(false, cx);
                                                })),
                                        ),
                                ),
                        )
                        .child(
                            v_flex()
                                .gap_1()
                                .max_h(px(160.))
                                .overflow_y_scrollbar()
                                .children(playlist_entries.into_iter().map(|(idx, title, selected)| {
                                    Checkbox::new(format!("pl-{idx}"))
                                        .label(title)
                                        .checked(selected)
                                        .on_click(cx.listener(move |this, _, _, cx| {
                                            this.toggle_playlist_entry(idx, cx);
                                        }))
                                })),
                        )
                        .into_any_element()
                }
                CurrentMedia::None => {
                    div()
                        .p_4()
                        .rounded(cx.theme().radius)
                        .border_1()
                        .border_color(cx.theme().border)
                        .bg(cx.theme().secondary)
                        .text_color(cx.theme().muted_foreground)
                        .child("No media loaded yet. Go to Dashboard, paste a URL, and click Analyze.")
                        .into_any_element()
                }
            })
            // Mode Section
            .child(
                v_flex()
                    .gap_2()
                    .child(
                        div()
                            .text_sm()
                            .font_semibold()
                            .text_color(cx.theme().foreground)
                            .child("Download Mode"),
                    )
                    .child(
                        h_flex()
                            .gap_3()
                            .child(render_mode_card(
                                "mode-va",
                                "Video + Audio",
                                "Original video and audio",
                                IconName::Tv,
                                opts.mode == DownloadMode::VideoAudio,
                                DownloadMode::VideoAudio,
                                cx,
                            ))
                            .child(render_mode_card(
                                "mode-ao",
                                "Audio Only",
                                "Extract high quality audio track",
                                IconName::Music,
                                opts.mode == DownloadMode::AudioOnly,
                                DownloadMode::AudioOnly,
                                cx,
                            ))
                            .child(render_mode_card(
                                "mode-vo",
                                "Video Only",
                                "No audio track",
                                IconName::Video,
                                opts.mode == DownloadMode::VideoOnly,
                                DownloadMode::VideoOnly,
                                cx,
                            )),
                    ),
            )
            // Options for Video modes (Container, Resolution Ladder, FPS)
            .when(opts.mode != DownloadMode::AudioOnly, |this| {
                this.child(
                    v_flex()
                        .gap_3()
                        // Container chips
                        .child(
                            v_flex()
                                .gap_2()
                                .child(
                                    div()
                                        .text_sm()
                                        .font_semibold()
                                        .text_color(cx.theme().foreground)
                                        .child("Video Container"),
                                )
                                .child(
                                    h_flex()
                                        .gap_2()
                                        .children(Container::ALL_VIDEO.iter().map(|&cont| {
                                            let selected = opts.container == cont;
                                            Button::new(format!("container-{}", cont.extension()))
                                                .label(cont.label())
                                                .small()
                                                .when(selected, |b| b.primary())
                                                .on_click(cx.listener(move |this, _, _, cx| {
                                                    this.set_container(cont, cx);
                                                }))
                                        })),
                                ),
                        )
                        // Resolution ladder
                        .child(
                            v_flex()
                                .gap_2()
                                .child(
                                    div()
                                        .text_sm()
                                        .font_semibold()
                                        .text_color(cx.theme().foreground)
                                        .child("Resolution Preset"),
                                )
                                .child(
                                    h_flex()
                                        .gap_2()
                                        .flex_wrap()
                                        .children(ResolutionPreset::ALL.iter().map(|&res| {
                                            let selected = opts.resolution == res && opts.video_format_id.is_none();
                                            Button::new(format!("res-{}", res.label()))
                                                .label(res.label())
                                                .small()
                                                .when(selected, |b| b.primary())
                                                .on_click(cx.listener(move |this, _, _, cx| {
                                                    this.set_resolution(res, cx);
                                                }))
                                        })),
                                ),
                        )
                        // FPS Presets
                        .child(
                            v_flex()
                                .gap_2()
                                .child(
                                    div()
                                        .text_sm()
                                        .font_semibold()
                                        .text_color(cx.theme().foreground)
                                        .child("Frame Rate (FPS)"),
                                )
                                .child(
                                    h_flex()
                                        .gap_2()
                                        .child(
                                            Button::new("fps-any")
                                                .label("Any FPS")
                                                .small()
                                                .when(opts.fps_filter.is_none(), |b| b.primary())
                                                .on_click(cx.listener(|this, _, _, cx| {
                                                    this.set_fps(None, cx);
                                                })),
                                        )
                                        .child(
                                            Button::new("fps-60")
                                                .label("60 FPS")
                                                .small()
                                                .when(opts.fps_filter == Some(60), |b| b.primary())
                                                .on_click(cx.listener(|this, _, _, cx| {
                                                    this.set_fps(Some(60), cx);
                                                })),
                                        )
                                        .child(
                                            Button::new("fps-30")
                                                .label("30 FPS")
                                                .small()
                                                .when(opts.fps_filter == Some(30), |b| b.primary())
                                                .on_click(cx.listener(|this, _, _, cx| {
                                                    this.set_fps(Some(30), cx);
                                                })),
                                        ),
                                ),
                        ),
                )
            })
            // Options for Audio Only mode (Format chips & Quality presets)
            .when(opts.mode == DownloadMode::AudioOnly, |this| {
                this.child(
                    v_flex()
                        .gap_3()
                        // Audio format chips
                        .child(
                            v_flex()
                                .gap_2()
                                .child(
                                    div()
                                        .text_sm()
                                        .font_semibold()
                                        .text_color(cx.theme().foreground)
                                        .child("Audio Format"),
                                )
                                .child(
                                    h_flex()
                                        .gap_2()
                                        .flex_wrap()
                                        .children(AudioFormat::ALL.iter().map(|&fmt| {
                                            let selected = opts.audio_format == fmt;
                                            Button::new(format!("afmt-{}", fmt.extension()))
                                                .label(fmt.label())
                                                .small()
                                                .when(selected, |b| b.primary())
                                                .on_click(cx.listener(move |this, _, _, cx| {
                                                    this.set_audio_format(fmt, cx);
                                                }))
                                        })),
                                ),
                        )
                        // Audio quality / bitrate
                        .child(
                            v_flex()
                                .gap_2()
                                .child(
                                    div()
                                        .text_sm()
                                        .font_semibold()
                                        .text_color(cx.theme().foreground)
                                        .child("Audio Quality / Bitrate"),
                                )
                                .child(
                                    h_flex()
                                        .gap_2()
                                        .flex_wrap()
                                        .children(AudioQualityPreset::ALL.iter().map(|&qual| {
                                            let selected = opts.audio_quality == qual;
                                            Button::new(format!("aqual-{}", qual.value()))
                                                .label(qual.label())
                                                .small()
                                                .when(selected, |b| b.primary())
                                                .on_click(cx.listener(move |this, _, _, cx| {
                                                    this.set_audio_quality(qual, cx);
                                                }))
                                        })),
                                ),
                        ),
                )
            })
            // Subtitles Section
            .child(
                v_flex()
                    .gap_2()
                    .child(
                        div()
                            .text_sm()
                            .font_semibold()
                            .text_color(cx.theme().foreground)
                            .child("Subtitles & Captions"),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .child(
                                Button::new("sub-none")
                                    .label("None")
                                    .small()
                                    .when(opts.subtitle_mode == SubtitleMode::None, |b| b.primary())
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.set_subtitle_mode(SubtitleMode::None, cx);
                                    })),
                            )
                            .child(
                                Button::new("sub-all")
                                    .label("All Available")
                                    .small()
                                    .icon(IconName::Captions)
                                    .when(opts.subtitle_mode == SubtitleMode::All, |b| b.primary())
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.set_subtitle_mode(SubtitleMode::All, cx);
                                    })),
                            )
                            .child(
                                Button::new("sub-custom")
                                    .label("Select Languages")
                                    .small()
                                    .when(opts.subtitle_mode == SubtitleMode::Custom, |b| b.primary())
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.set_subtitle_mode(SubtitleMode::Custom, cx);
                                    })),
                            ),
                    )
                    // Subtitle languages picker when Custom mode is active
                    .when(opts.subtitle_mode == SubtitleMode::Custom, |this| {
                        let available_subs = match &media {
                            CurrentMedia::Single(m) => m.subtitles.clone(),
                            _ => Vec::new(),
                        };
                        let all_codes: Vec<String> = available_subs.iter().map(|s| s.language.clone()).collect();
                        let all_codes_clone = all_codes.clone();

                        this.child(
                            v_flex()
                                .gap_2()
                                .p_3()
                                .rounded(cx.theme().radius)
                                .bg(cx.theme().secondary)
                                .border_1()
                                .border_color(cx.theme().border)
                                .child(
                                    h_flex()
                                        .justify_between()
                                        .items_center()
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(cx.theme().muted_foreground)
                                                .child(if available_subs.is_empty() {
                                                    "No uploaded subtitles detected for this video.".into()
                                                } else {
                                                    format!("Select from {} uploaded subtitle(s):", available_subs.len())
                                                }),
                                        )
                                        .when(!available_subs.is_empty(), |h| {
                                            h.child(
                                                h_flex()
                                                    .gap_1()
                                                    .child(
                                                        Button::new("sub-sel-all")
                                                            .label("Select All")
                                                            .xsmall()
                                                            .ghost()
                                                            .on_click(cx.listener(move |this, _, _, cx| {
                                                                this.select_all_subtitles(all_codes_clone.clone(), cx);
                                                            })),
                                                    )
                                                    .child(
                                                        Button::new("sub-clear")
                                                            .label("Clear")
                                                            .xsmall()
                                                            .ghost()
                                                            .on_click(cx.listener(|this, _, _, cx| {
                                                                this.clear_subtitles(cx);
                                                            })),
                                                    ),
                                            )
                                        }),
                                )
                                .when(!available_subs.is_empty(), |v| {
                                    v.child(
                                        div()
                                            .relative()
                                            .rounded(cx.theme().radius)
                                            .bg(cx.theme().background)
                                            .border_1()
                                            .border_color(cx.theme().border)
                                            .child(
                                                v_flex()
                                                    .id("subtitles-scroll-box")
                                                    .max_h(px(180.))
                                                    .overflow_y_scroll()
                                                    .track_scroll(&self.subtitles_scroll_handle)
                                                    .on_scroll_wheel(|_, _, cx| cx.stop_propagation())
                                                    .gap_2()
                                                    .p_2()
                                                    .pr_6()
                                                    .children(available_subs.into_iter().map(|sub| {
                                                        let code = sub.language.clone();
                                                        let label = sub.display_label();
                                                        let is_checked = opts.subtitle_langs.contains(&code);
                                                        let code_clone = code.clone();
                                                        Checkbox::new(format!("sub-lang-{}", code))
                                                            .label(label)
                                                            .checked(is_checked)
                                                            .on_click(cx.listener(move |this, _, _, cx| {
                                                                this.toggle_subtitle_lang(code_clone.clone(), cx);
                                                            }))
                                                    })),
                                            )
                                            .child(
                                                Scrollbar::vertical(&self.subtitles_scroll_handle)
                                                    .mode(ScrollbarMode::Always)
                                            ),
                                    )
                                }),
                        )
                    }),
            )
            // Audio Tracks & Metadata Options
            .child(
                v_flex()
                    .gap_2()
                    .child(
                        div()
                            .text_sm()
                            .font_semibold()
                            .text_color(cx.theme().foreground)
                            .child("Audio Tracks & Streams"),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .child(
                                Button::new("audio-mode-default")
                                    .label("Default Audio")
                                    .small()
                                    .when(opts.audio_tracks_mode == AudioTracksMode::Default, |b| b.primary())
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.set_audio_tracks_mode(AudioTracksMode::Default, cx);
                                    })),
                            )
                            .child(
                                Button::new("audio-mode-all")
                                    .label("All Audio Tracks")
                                    .small()
                                    .when(opts.audio_tracks_mode == AudioTracksMode::All, |b| b.primary())
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.set_audio_tracks_mode(AudioTracksMode::All, cx);
                                    })),
                            )
                            .child(
                                Button::new("audio-mode-custom")
                                    .label("Select Tracks")
                                    .small()
                                    .when(opts.audio_tracks_mode == AudioTracksMode::Custom, |b| b.primary())
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.set_audio_tracks_mode(AudioTracksMode::Custom, cx);
                                    })),
                            ),
                    )
                    // Audio tracks picker when Select Tracks is active
                    .when(opts.audio_tracks_mode == AudioTracksMode::Custom, |this| {
                        let available_audio = match &media {
                            CurrentMedia::Single(m) => m.audio_tracks.clone(),
                            _ => Vec::new(),
                        };
                        let all_track_keys: Vec<String> = available_audio
                            .iter()
                            .map(|t| t.language.clone().unwrap_or_else(|| t.format_id.clone()))
                            .collect();
                        let all_track_keys_clone = all_track_keys.clone();

                        this.child(
                            v_flex()
                                .gap_2()
                                .p_3()
                                .rounded(cx.theme().radius)
                                .bg(cx.theme().secondary)
                                .border_1()
                                .border_color(cx.theme().border)
                                .child(
                                    h_flex()
                                        .justify_between()
                                        .items_center()
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(cx.theme().muted_foreground)
                                                .child(if available_audio.is_empty() {
                                                    "No discrete audio tracks detected for this media.".into()
                                                } else {
                                                    format!("Select from {} detected audio track(s):", available_audio.len())
                                                }),
                                        )
                                        .when(!available_audio.is_empty(), |h| {
                                            h.child(
                                                h_flex()
                                                    .gap_1()
                                                    .child(
                                                        Button::new("audio-sel-all")
                                                            .label("Select All")
                                                            .xsmall()
                                                            .ghost()
                                                            .on_click(cx.listener(move |this, _, _, cx| {
                                                                this.select_all_audio_tracks(all_track_keys_clone.clone(), cx);
                                                            })),
                                                    )
                                                    .child(
                                                        Button::new("audio-clear")
                                                            .label("Clear")
                                                            .xsmall()
                                                            .ghost()
                                                            .on_click(cx.listener(|this, _, _, cx| {
                                                                this.clear_audio_tracks(cx);
                                                            })),
                                                    ),
                                            )
                                        }),
                                )
                                .when(!available_audio.is_empty(), |v| {
                                    v.child(
                                        div()
                                            .relative()
                                            .rounded(cx.theme().radius)
                                            .bg(cx.theme().background)
                                            .border_1()
                                            .border_color(cx.theme().border)
                                            .child(
                                                v_flex()
                                                    .id("audio-tracks-scroll-box")
                                                    .max_h(px(180.))
                                                    .overflow_y_scroll()
                                                    .track_scroll(&self.audio_scroll_handle)
                                                    .on_scroll_wheel(|_, _, cx| cx.stop_propagation())
                                                    .gap_2()
                                                    .p_2()
                                                    .pr_6()
                                                    .children(available_audio.into_iter().map(|track| {
                                                        let key = track.language.clone().unwrap_or_else(|| track.format_id.clone());
                                                        let label = track.display_label();
                                                        let is_checked = opts.selected_audio_langs.contains(&key)
                                                            || opts.selected_audio_langs.contains(&track.format_id);
                                                        let key_clone = key.clone();
                                                        Checkbox::new(format!("audio-track-{}", track.format_id))
                                                            .label(label)
                                                            .checked(is_checked)
                                                            .on_click(cx.listener(move |this, _, _, cx| {
                                                                this.toggle_audio_lang(key_clone.clone(), cx);
                                                            }))
                                                    })),
                                            )
                                            .child(
                                                Scrollbar::vertical(&self.audio_scroll_handle)
                                                    .mode(ScrollbarMode::Always)
                                            ),
                                    )
                                }),
                        )
                    })
                    // Embedding options
                    .child(
                        v_flex()
                            .gap_2()
                            .pt_1()
                            .child(
                                Checkbox::new("embed-thumb")
                                    .label("Embed video thumbnail into output file")
                                    .checked(opts.embed_thumbnail)
                                    .on_click(cx.listener(|this, checked, _, cx| {
                                        this.toggle_embed_thumbnail(*checked, cx);
                                    })),
                            )
                            .child(
                                Checkbox::new("embed-meta")
                                    .label("Embed metadata tags & description")
                                    .checked(opts.embed_metadata)
                                    .on_click(cx.listener(|this, checked, _, cx| {
                                        this.toggle_embed_metadata(*checked, cx);
                                    })),
                            ),
                    ),
            )
            // Format Explorer Toggle (Phase 2 feature integrated)
            .child(
                v_flex()
                    .gap_2()
                    .child(
                        h_flex()
                            .justify_between()
                            .items_center()
                            .child(
                                h_flex()
                                    .gap_2()
                                    .items_center()
                                    .child(
                                        Button::new("toggle-explorer")
                                            .ghost()
                                            .small()
                                            .icon(IconName::SlidersHorizontal)
                                            .label(if show_explorer { "Hide Format Explorer" } else { "Show Raw Format Explorer" })
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.toggle_format_explorer(cx);
                                            })),
                                    )
                                    .when(opts.video_format_id.is_some() || opts.audio_format_id.is_some(), |this| {
                                        let v = opts.video_format_id.as_deref().unwrap_or("auto");
                                        let a = opts.audio_format_id.as_deref().unwrap_or("auto");
                                        this.child(
                                            div()
                                                .text_xs()
                                                .font_semibold()
                                                .text_color(cx.theme().primary)
                                                .child(format!("Manual Override: Video #{v} · Audio #{a}")),
                                        )
                                        .child(
                                            Button::new("reset-explorer")
                                                .ghost()
                                                .xsmall()
                                                .label("Reset to Presets")
                                                .on_click(cx.listener(|this, _, _, cx| {
                                                    this.reset_custom_formats(cx);
                                                })),
                                        )
                                    }),
                            ),
                    )
                    .when(show_explorer, |this| {
                        let formats = match &media {
                            CurrentMedia::Single(m) => m.formats.clone(),
                            _ => Vec::new(),
                        };

                        this.child(
                            v_flex()
                                .gap_1()
                                .p_3()
                                .rounded(cx.theme().radius)
                                .border_1()
                                .border_color(cx.theme().border)
                                .bg(cx.theme().secondary)
                                .child(
                                    div()
                                        .text_xs()
                                        .font_semibold()
                                        .text_color(cx.theme().foreground)
                                        .child("Available Streams (Click row to select Video or Audio stream):"),
                                )
                                .child(
                                    v_flex()
                                        .gap_1()
                                        .max_h(px(220.))
                                        .overflow_y_scrollbar()
                                        .children(formats.into_iter().map(|fmt| {
                                            let fid = fmt.format_id.clone();
                                            let is_vid = fmt.stream_type == StreamType::VideoOnly || fmt.stream_type == StreamType::Muxed;
                                            let is_aud = fmt.stream_type == StreamType::AudioOnly;
                                            let is_selected_v = opts.video_format_id.as_deref() == Some(&fid);
                                            let is_selected_a = opts.audio_format_id.as_deref() == Some(&fid);
                                            let is_selected = is_selected_v || is_selected_a;

                                            let res_str = fmt.resolution.clone().unwrap_or_else(|| {
                                                fmt.height.map(|h| format!("{h}p")).unwrap_or_else(|| "audio".into())
                                            });
                                            let fps_str = fmt.fps.map(|f| format!("{f:.0}fps")).unwrap_or_default();
                                            let vcodec = fmt.vcodec.clone().unwrap_or_else(|| "none".into());
                                            let acodec = fmt.acodec.clone().unwrap_or_else(|| "none".into());
                                            let size_str = FormatBuilder::format_size(fmt.filesize);

                                            let fid_clone = fid.clone();
                                            h_flex()
                                                .id(format!("fmt-row-{}", fid))
                                                .p_1()
                                                .gap_2()
                                                .rounded(px(3.))
                                                .text_xs()
                                                .items_center()
                                                .justify_between()
                                                .when(is_selected, |el| el.bg(cx.theme().primary.alpha(0.15)))
                                                .child(
                                                    h_flex()
                                                        .gap_2()
                                                        .child(div().w(px(40.)).font_semibold().text_color(cx.theme().foreground).child(format!("#{fid}")))
                                                        .child(div().w(px(45.)).text_color(cx.theme().muted_foreground).child(fmt.ext.clone()))
                                                        .child(div().w(px(70.)).text_color(cx.theme().foreground).child(res_str))
                                                        .child(div().w(px(45.)).text_color(cx.theme().muted_foreground).child(fps_str))
                                                        .child(div().w(px(90.)).text_color(cx.theme().muted_foreground).child(format!("v:{vcodec}")))
                                                        .child(div().w(px(90.)).text_color(cx.theme().muted_foreground).child(format!("a:{acodec}")))
                                                        .child(div().w(px(60.)).text_color(cx.theme().foreground).child(size_str)),
                                                )
                                                .child(
                                                    h_flex()
                                                        .gap_1()
                                                        .when(is_vid, |h| {
                                                            let fid_c = fid_clone.clone();
                                                            h.child(
                                                                Button::new(format!("sel-v-{}", fid_c))
                                                                    .ghost()
                                                                    .xsmall()
                                                                    .label(if is_selected_v { "Selected (V)" } else { "Set Video" })
                                                                    .when(is_selected_v, |b| b.primary())
                                                                    .on_click(cx.listener(move |this, _, _, cx| {
                                                                        this.select_raw_stream(fid_c.clone(), true, cx);
                                                                    })),
                                                            )
                                                        })
                                                        .when(is_aud, |h| {
                                                            let fid_c = fid_clone.clone();
                                                            h.child(
                                                                Button::new(format!("sel-a-{}", fid_c))
                                                                    .ghost()
                                                                    .xsmall()
                                                                    .label(if is_selected_a { "Selected (A)" } else { "Set Audio" })
                                                                    .when(is_selected_a, |b| b.primary())
                                                                    .on_click(cx.listener(move |this, _, _, cx| {
                                                                        this.select_raw_stream(fid_c.clone(), false, cx);
                                                                    })),
                                                            )
                                                        }),
                                                )
                                        })),
                                ),
                        )
                    }),
            )
            // Footer Action Bar
            .child(
                h_flex()
                    .w_full()
                    .justify_between()
                    .items_center()
                    .pt_3()
                    .border_t_1()
                    .border_color(cx.theme().border)
                    .child(
                        v_flex()
                            .child(
                                div()
                                    .text_sm()
                                    .font_semibold()
                                    .text_color(cx.theme().foreground)
                                    .child(format!("Estimated Size: {size_label}")),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("Exact size may vary depending on compression and post-processing"),
                            ),
                    )
                    .child(
                        Button::new("download-btn")
                            .primary()
                            .icon(IconName::Download)
                            .label("Start Download")
                            .disabled(!has_media)
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.start_download(window, cx);
                            })),
                    ),
            )
    }
}

fn render_mode_card(
    id: &'static str,
    title: &'static str,
    desc: &'static str,
    icon: IconName,
    selected: bool,
    mode: DownloadMode,
    cx: &mut Context<ConfigView>,
) -> impl IntoElement {
    let mut bg = if selected {
        crate::ui::theme::CORAL_CONTAINER
    } else {
        cx.theme().secondary
    };
    if selected {
        bg.a = 0.15;
    }
    let border_color = if selected {
        crate::ui::theme::CORAL
    } else {
        cx.theme().border
    };
    let icon_color = if selected {
        crate::ui::theme::CORAL
    } else {
        cx.theme().muted_foreground
    };

    h_flex()
        .id(id)
        .flex_1()
        .p_4()
        .rounded(crate::ui::theme::RADIUS_CARD)
        .bg(bg)
        .border_1()
        .border_color(border_color)
        .gap_3()
        .items_center()
        .cursor_pointer()
        .hover(|s| s.bg(cx.theme().muted))
        .on_mouse_down(
            gpui_kit::MouseButton::Left,
            cx.listener(move |this, _, _, cx| {
                this.set_mode(mode, cx);
            }),
        )
        .child(Icon::new(icon).size(px(24.)).text_color(icon_color))
        .child(
            v_flex()
                .gap(px(2.))
                .child(
                    div()
                        .font_semibold()
                        .text_sm()
                        .text_color(cx.theme().foreground)
                        .child(title),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(desc),
                ),
        )
}
