//! Built-in media converter (local files via FFmpeg).

use std::path::{Path, PathBuf};
use gpui_kit::component::button::Button;
use gpui_kit::component::checkbox::Checkbox;
use gpui_kit::component::input::{Input, InputState};
use gpui_kit::component::{h_flex, v_flex, Icon};
use crate::ui::prelude::*;

use crate::core::AppState;
use crate::models::{ConversionId, ConversionProfile, ConversionStatus, ConversionTask};

pub struct ConverterView {
    state: Entity<AppState>,
    path_input: Entity<InputState>,
    profile: ConversionProfile,
    keep_original: bool,
}

impl ConverterView {
    pub fn new(window: &mut Window, state: Entity<AppState>, cx: &mut Context<Self>) -> Self {
        let path_input = cx.new(|cx| {
            InputState::new(window, cx).placeholder("Path to local media file (e.g. C:\\Videos\\sample.mkv)…")
        });
        Self {
            state,
            path_input,
            profile: ConversionProfile::RemuxMp4,
            keep_original: true,
        }
    }

    fn set_profile(&mut self, profile: ConversionProfile, cx: &mut Context<Self>) {
        self.profile = profile;
        cx.notify();
    }

    fn toggle_keep_original(&mut self, keep: bool, cx: &mut Context<Self>) {
        self.keep_original = keep;
        cx.notify();
    }

    fn browse_file(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        #[cfg(windows)]
        {
            let script = r#"
                Add-Type -AssemblyName System.Windows.Forms
                $f = New-Object System.Windows.Forms.OpenFileDialog
                $f.Filter = "Media Files (*.mp4;*.mkv;*.webm;*.avi;*.mov;*.flv;*.ts;*.mp3;*.m4a;*.wav;*.flac;*.opus)|*.mp4;*.mkv;*.webm;*.avi;*.mov;*.flv;*.ts;*.mp3;*.m4a;*.wav;*.flac;*.opus|All Files (*.*)|*.*"
                $f.Title = "Select Media File to Convert"
                if ($f.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK) {
                    Write-Output $f.FileName
                }
            "#;
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            let out = std::process::Command::new("powershell")
                .args(["-NoProfile", "-Command", script])
                .creation_flags(CREATE_NO_WINDOW)
                .output();

            if let Ok(output) = out {
                let selected = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !selected.is_empty() {
                    self.path_input.update(cx, |input, cx| {
                        input.set_value(selected, window, cx);
                    });
                    cx.notify();
                }
            }
        }
    }

    fn enqueue(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let path = self.path_input.read(cx).value().trim().to_string();
        if path.is_empty() {
            window.push_notification("Please select or enter a media file path.", cx);
            return;
        }

        let profile = self.profile;
        let keep = self.keep_original;
        self.state.update(cx, |s, cx| {
            s.conversions.enqueue_with_options(path.clone(), profile, keep);
            cx.notify();
        });

        window.push_notification(format!("Added conversion job for {}", profile.label()), cx);
    }

    fn cancel(&mut self, id: ConversionId, cx: &mut Context<Self>) {
        self.state.update(cx, |s, cx| {
            s.conversions.cancel(id);
            cx.notify();
        });
    }

    fn retry(&mut self, id: ConversionId, cx: &mut Context<Self>) {
        self.state.update(cx, |s, cx| {
            s.conversions.retry(id);
            cx.notify();
        });
    }

    fn remove(&mut self, id: ConversionId, cx: &mut Context<Self>) {
        self.state.update(cx, |s, cx| {
            s.conversions.remove(id);
            cx.notify();
        });
    }

    fn clear_completed(&mut self, cx: &mut Context<Self>) {
        self.state.update(cx, |s, cx| {
            s.conversions.clear_completed();
            cx.notify();
        });
    }

    fn reveal_file(path_opt: Option<&str>, default_dir: &Path) {
        if let Some(path_str) = path_opt {
            let p = Path::new(path_str);
            if p.exists() {
                #[cfg(windows)]
                {
                    use std::os::windows::process::CommandExt;
                    const CREATE_NO_WINDOW: u32 = 0x08000000;
                    let _ = std::process::Command::new("explorer")
                        .arg(format!("/select,\"{}\"", p.to_string_lossy()))
                        .creation_flags(CREATE_NO_WINDOW)
                        .spawn();
                }
                #[cfg(not(windows))]
                {
                    if let Some(parent) = p.parent() {
                        let _ = std::process::Command::new("xdg-open").arg(parent).spawn();
                    }
                }
                return;
            }
        }

        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            let _ = std::process::Command::new("explorer")
                .arg(default_dir.to_string_lossy().as_ref())
                .creation_flags(CREATE_NO_WINDOW)
                .spawn();
        }
        #[cfg(not(windows))]
        {
            let _ = std::process::Command::new("xdg-open").arg(default_dir).spawn();
        }
    }
}

impl Render for ConverterView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let tasks = self.state.read(cx).conversions.list();
        let output_dir = self.state.read(cx).conversions.output_dir().clone();
        let profile = self.profile;
        let keep_original = self.keep_original;

        let completed_count = tasks.iter().filter(|t| t.status == ConversionStatus::Completed).count();
        let cards: Vec<AnyElement> = tasks
            .into_iter()
            .map(|task| render_conversion_card(task, &output_dir, cx))
            .collect();

        v_flex()
            .size_full()
            .p_6()
            .gap_4()
            .bg(cx.theme().background)
            // Header bar
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
                                    .text_xl()
                                    .font_bold()
                                    .text_color(cx.theme().foreground)
                                    .child("Media Converter"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("Lossless remuxing, video transcoding, and audio extraction via FFmpeg"),
                            ),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .when(completed_count > 0, |this| {
                                this.child(
                                    Button::new("conv-clear-completed")
                                        .outline()
                                        .icon(IconName::Trash)
                                        .label("Clear Finished")
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.clear_completed(cx);
                                        })),
                                )
                            }),
                    ),
            )
            // Input & Action Row
            .child(
                v_flex()
                    .p_4()
                    .gap_3()
                    .rounded(cx.theme().radius)
                    .bg(cx.theme().secondary)
                    .border_1()
                    .border_color(cx.theme().border)
                    .child(
                        h_flex()
                            .w_full()
                            .gap_2()
                            .items_center()
                            .child(div().flex_1().child(Input::new(&self.path_input)))
                            .child(
                                Button::new("conv-browse")
                                    .outline()
                                    .icon(IconName::FolderOpen)
                                    .label("Browse…")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.browse_file(window, cx);
                                    })),
                            )
                            .child(
                                Button::new("convert-btn")
                                    .primary()
                                    .icon(IconName::RefreshCw)
                                    .label("Convert")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.enqueue(window, cx);
                                    })),
                            ),
                    )
                    .child(
                        h_flex()
                            .gap_4()
                            .items_center()
                            .child(
                                Checkbox::new("keep-original-cb")
                                    .label("Keep original input file after conversion")
                                    .checked(keep_original)
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.toggle_keep_original(!keep_original, cx);
                                    })),
                            ),
                    ),
            )
            // Profiles Category Section
            .child(
                v_flex()
                    .gap_3()
                    .p_4()
                    .rounded(cx.theme().radius)
                    .bg(cx.theme().secondary)
                    .border_1()
                    .border_color(cx.theme().border)
                    .child(
                        div()
                            .text_xs()
                            .font_semibold()
                            .text_color(cx.theme().foreground)
                            .child("VIDEO CONTAINERS & CODECS"),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .flex_wrap()
                            .children(ConversionProfile::ALL_VIDEO.iter().copied().map(|p| {
                                let selected = profile == p;
                                Button::new(format!("prof-vid-{}", p.label()))
                                    .label(p.label())
                                    .when(selected, |b| b.primary())
                                    .when(!selected, |b| b.ghost())
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.set_profile(p, cx);
                                    }))
                            })),
                    )
                    .child(
                        div()
                            .text_xs()
                            .font_semibold()
                            .text_color(cx.theme().foreground)
                            .child("AUDIO EXTRACTION"),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .flex_wrap()
                            .children(ConversionProfile::ALL_AUDIO.iter().copied().map(|p| {
                                let selected = profile == p;
                                Button::new(format!("prof-aud-{}", p.label()))
                                    .label(p.label())
                                    .when(selected, |b| b.primary())
                                    .when(!selected, |b| b.ghost())
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.set_profile(p, cx);
                                    }))
                            })),
                    ),
            )
            // Tasks List or Empty State
            .child(
                if cards.is_empty() {
                    crate::ui::components::empty_state::empty_state(
                        IconName::RefreshCw,
                        "No conversion jobs yet",
                        "Select a file above, pick a target container or codec profile, then press Convert.",
                    )
                    .into_any_element()
                } else {
                    v_flex()
                        .flex_1()
                        .gap_2()
                        .overflow_y_scrollbar()
                        .children(cards)
                        .into_any_element()
                }
            )
    }
}

fn render_conversion_card(
    task: ConversionTask,
    output_dir: &PathBuf,
    cx: &mut Context<ConverterView>,
) -> AnyElement {
    let id = task.id;
    let is_running = task.status == ConversionStatus::Running;
    let is_completed = task.status == ConversionStatus::Completed;
    let is_failed = task.status == ConversionStatus::Failed || task.status == ConversionStatus::Cancelled;

    let (status_color, status_icon) = match task.status {
        ConversionStatus::Completed => (cx.theme().success, IconName::Check),
        ConversionStatus::Failed => (cx.theme().danger, IconName::CircleAlert),
        ConversionStatus::Cancelled => (cx.theme().muted_foreground, IconName::CircleSlash),
        ConversionStatus::Running => (cx.theme().primary, IconName::RefreshCw),
        ConversionStatus::Queued => (cx.theme().muted_foreground, IconName::Clock),
    };

    let filename = Path::new(&task.input_path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(&task.input_path)
        .to_string();

    let output_clone = task.output_path.clone();
    let dir_clone = output_dir.clone();

    v_flex()
        .id(format!("conv-card-{}", id))
        .w_full()
        .p_3()
        .gap_2()
        .rounded(cx.theme().radius)
        .border_1()
        .border_color(cx.theme().border)
        .bg(cx.theme().secondary)
        .child(
            h_flex()
                .w_full()
                .justify_between()
                .items_start()
                .gap_4()
                .child(
                    v_flex()
                        .flex_1()
                        .gap_1()
                        .child(
                            div()
                                .font_semibold()
                                .text_sm()
                                .text_color(cx.theme().foreground)
                                .child(filename),
                        )
                        .child(
                            h_flex()
                                .gap_2()
                                .items_center()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child(
                                    h_flex()
                                        .gap_1()
                                        .items_center()
                                        .text_color(status_color)
                                        .font_semibold()
                                        .child(Icon::new(status_icon).size(px(12.)))
                                        .child(task.status.label()),
                                )
                                .child(div().child("·"))
                                .child(
                                    div()
                                        .text_color(cx.theme().primary)
                                        .child(task.profile.label()),
                                )
                                .when_some(task.output_path.as_ref(), |this, out| {
                                    this.child(div().child("·"))
                                        .child(
                                            div()
                                                .max_w(px(320.))
                                                .text_ellipsis()
                                                .text_color(cx.theme().muted_foreground)
                                                .child(out.clone()),
                                        )
                                }),
                        ),
                )
                .child(
                    h_flex()
                        .gap_1p5()
                        .items_center()
                        .when(is_completed, |this| {
                            let out = output_clone.clone();
                            let d = dir_clone.clone();
                            this.child(
                                Button::new(format!("conv-reveal-{id}"))
                                    .outline()
                                    .icon(IconName::FolderOpen)
                                    .label("Open")
                                    .on_click(cx.listener(move |_, _, _, _| {
                                        ConverterView::reveal_file(out.as_deref(), &d);
                                    })),
                            )
                        })
                        .when(is_failed, |this| {
                            this.child(
                                Button::new(format!("conv-retry-{id}"))
                                    .outline()
                                    .icon(IconName::RotateCw)
                                    .label("Retry")
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.retry(id, cx);
                                    })),
                            )
                        })
                        .when(is_running, |this| {
                            this.child(
                                Button::new(format!("conv-cancel-{id}"))
                                    .danger()
                                    .icon(IconName::X)
                                    .label("Cancel")
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.cancel(id, cx);
                                    })),
                            )
                        })
                        .when(!is_running, |this| {
                            this.child(
                                Button::new(format!("conv-rm-{id}"))
                                    .ghost()
                                    .icon(IconName::Trash)
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.remove(id, cx);
                                    })),
                            )
                        }),
                ),
        )
        .when_some(task.error.as_ref(), |this, err| {
            this.child(
                h_flex()
                    .gap_2()
                    .p_2()
                    .rounded(cx.theme().radius)
                    .bg(cx.theme().danger.opacity(0.1))
                    .border_1()
                    .border_color(cx.theme().danger.opacity(0.3))
                    .text_xs()
                    .text_color(cx.theme().danger)
                    .child(Icon::new(IconName::CircleAlert).size(px(14.)))
                    .child(err.clone()),
            )
        })
        .into_any_element()
}
