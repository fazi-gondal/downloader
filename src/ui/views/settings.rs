//! Full settings editors — path, concurrency, proxy, toggles, theme.

use std::path::PathBuf;
use gpui_kit::component::button::Button;
use gpui_kit::component::input::{Input, InputState};
use gpui_kit::component::switch::Switch;
use gpui_kit::component::{h_flex, v_flex, Theme, ThemeMode as KitThemeMode};
use crate::ui::prelude::*;

use crate::config::{AppSettings, ThemeMode};
use crate::core::AppState;

pub struct SettingsView {
    state: Entity<AppState>,
    download_dir_input: Entity<InputState>,
    proxy_input: Entity<InputState>,
    cookies_input: Entity<InputState>,
}

impl SettingsView {
    pub fn new(window: &mut Window, state: Entity<AppState>, cx: &mut Context<Self>) -> Self {
        let settings = state.read(cx).settings.clone();
        let download_dir_input = cx.new(|cx| {
            InputState::new(window, cx)
                .default_value(settings.download_dir.display().to_string())
                .placeholder("Download directory (e.g. C:\\Downloads)")
        });
        let proxy_input = cx.new(|cx| {
            InputState::new(window, cx)
                .default_value(settings.proxy.clone())
                .placeholder("http://… or socks5://… (optional)")
        });
        let cookies_input = cx.new(|cx| {
            InputState::new(window, cx)
                .default_value(settings.cookies_browser.clone())
                .placeholder("chrome | firefox | edge | brave (optional)")
        });
        Self {
            state,
            download_dir_input,
            proxy_input,
            cookies_input,
        }
    }

    fn browse_download_dir(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        #[cfg(windows)]
        {
            let script = r#"
                Add-Type -AssemblyName System.Windows.Forms
                $f = New-Object System.Windows.Forms.FolderBrowserDialog
                $f.Description = "Select Download Directory"
                if ($f.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK) {
                    Write-Output $f.SelectedPath
                }
            "#;
            let out = std::process::Command::new("powershell")
                .args(["-NoProfile", "-Command", script])
                .output();

            if let Ok(output) = out {
                let selected = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !selected.is_empty() {
                    self.download_dir_input.update(cx, |input, cx| {
                        input.set_value(selected, window, cx);
                    });
                    cx.notify();
                }
            }
        }
    }

    fn save(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let dir = self.download_dir_input.read(cx).value().trim().to_string();
        let proxy = self.proxy_input.read(cx).value().trim().to_string();
        let cookies = self.cookies_input.read(cx).value().trim().to_string();

        self.state.update(cx, |s, cx| {
            if !dir.is_empty() {
                s.settings.download_dir = PathBuf::from(dir);
            }
            s.settings.proxy = proxy;
            s.settings.cookies_browser = cookies;

            if let Err(e) = s.settings.save() {
                log::error!("failed to save settings: {e}");
            }
            s.apply_settings_to_services();
            cx.notify();
        });

        window.push_notification("Settings saved successfully.", cx);
    }

    fn bump_concurrent(&mut self, delta: i32, cx: &mut Context<Self>) {
        self.state.update(cx, |s, cx| {
            let v = s.settings.max_concurrent as i32 + delta;
            s.settings.max_concurrent = v.clamp(1, 16) as u32;
            cx.notify();
        });
    }

    fn bump_fragments(&mut self, delta: i32, cx: &mut Context<Self>) {
        self.state.update(cx, |s, cx| {
            let v = s.settings.concurrent_fragments as i32 + delta;
            s.settings.concurrent_fragments = v.clamp(1, 16) as u32;
            cx.notify();
        });
    }

    fn bump_rate_limit(&mut self, delta: i32, cx: &mut Context<Self>) {
        self.state.update(cx, |s, cx| {
            let v = s.settings.rate_limit_kbps as i32 + delta;
            s.settings.rate_limit_kbps = v.max(0) as u32;
            cx.notify();
        });
    }

    fn set_theme(&mut self, mode: ThemeMode, cx: &mut Context<Self>) {
        self.state.update(cx, |s, cx| {
            s.settings.theme_mode = mode;
            cx.notify();
        });

        match mode {
            ThemeMode::Dark => {
                Theme::change(KitThemeMode::Dark, None, cx);
            }
            ThemeMode::Light => {
                Theme::change(KitThemeMode::Light, None, cx);
            }
            ThemeMode::System => {
                Theme::change(KitThemeMode::Dark, None, cx);
            }
        }
    }

    fn set_keep_originals(&mut self, v: bool, cx: &mut Context<Self>) {
        self.state.update(cx, |s, cx| {
            s.settings.keep_originals = v;
            cx.notify();
        });
    }

    fn set_write_subtitles(&mut self, v: bool, cx: &mut Context<Self>) {
        self.state.update(cx, |s, cx| {
            s.settings.write_subtitles = v;
            cx.notify();
        });
    }

    fn set_multi_audio(&mut self, v: bool, cx: &mut Context<Self>) {
        self.state.update(cx, |s, cx| {
            s.settings.multi_audio = v;
            cx.notify();
        });
    }

    fn set_embed_thumbnail(&mut self, v: bool, cx: &mut Context<Self>) {
        self.state.update(cx, |s, cx| {
            s.settings.embed_thumbnail = v;
            cx.notify();
        });
    }

    fn set_embed_metadata(&mut self, v: bool, cx: &mut Context<Self>) {
        self.state.update(cx, |s, cx| {
            s.settings.embed_metadata = v;
            cx.notify();
        });
    }

    fn set_prefer_vp9(&mut self, v: bool, cx: &mut Context<Self>) {
        self.state.update(cx, |s, cx| {
            s.settings.prefer_vp9_video = v;
            cx.notify();
        });
    }

    fn export_settings(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let settings = self.state.read(cx).settings.clone();
        if let Ok(json) = serde_json::to_string_pretty(&settings) {
            let export_path = dirs::desktop_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("downloader_settings.json");
            if std::fs::write(&export_path, json).is_ok() {
                window.push_notification(format!("Settings exported to {}", export_path.display()), cx);
            } else {
                window.push_notification("Failed to export settings file.", cx);
            }
        }
    }

    fn import_settings(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        #[cfg(windows)]
        {
            let script = r#"
                Add-Type -AssemblyName System.Windows.Forms
                $f = New-Object System.Windows.Forms.OpenFileDialog
                $f.Filter = "JSON Files (*.json)|*.json|All Files (*.*)|*.*"
                $f.Title = "Import Settings JSON"
                if ($f.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK) {
                    Write-Output $f.FileName
                }
            "#;
            let out = std::process::Command::new("powershell")
                .args(["-NoProfile", "-Command", script])
                .output();

            if let Ok(output) = out {
                let file_path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !file_path.is_empty() {
                    if let Ok(content) = std::fs::read_to_string(&file_path) {
                        if let Ok(imported) = serde_json::from_str::<AppSettings>(&content) {
                            self.state.update(cx, |s, cx| {
                                s.settings = imported.clone();
                                let _ = s.settings.save();
                                s.apply_settings_to_services();
                                cx.notify();
                            });
                            self.download_dir_input.update(cx, |input, cx| {
                                input.set_value(imported.download_dir.display().to_string(), window, cx);
                            });
                            self.proxy_input.update(cx, |input, cx| {
                                input.set_value(imported.proxy.clone(), window, cx);
                            });
                            self.cookies_input.update(cx, |input, cx| {
                                input.set_value(imported.cookies_browser.clone(), window, cx);
                            });
                            window.push_notification("Settings imported successfully.", cx);
                            return;
                        }
                    }
                    window.push_notification("Failed to parse settings JSON.", cx);
                }
            }
        }
    }

    fn reset_defaults(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let defaults = AppSettings::default();
        self.state.update(cx, |s, cx| {
            s.settings = defaults.clone();
            let _ = s.settings.save();
            s.apply_settings_to_services();
            cx.notify();
        });
        self.download_dir_input.update(cx, |input, cx| {
            input.set_value(defaults.download_dir.display().to_string(), window, cx);
        });
        self.proxy_input.update(cx, |input, cx| {
            input.set_value(defaults.proxy.clone(), window, cx);
        });
        self.cookies_input.update(cx, |input, cx| {
            input.set_value(defaults.cookies_browser.clone(), window, cx);
        });
        window.push_notification("Settings reset to defaults.", cx);
    }
}

impl Render for SettingsView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let settings = self.state.read(cx).settings.clone();

        v_flex()
            .size_full()
            .p_6()
            .gap_4()
            .bg(cx.theme().background)
            .overflow_y_scrollbar()
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
                                    .child("Settings"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("Configure download destinations, concurrency limits, and defaults"),
                            ),
                    )
                    .child(
                        Button::new("save-settings")
                            .primary()
                            .icon(IconName::Check)
                            .label("Save Changes")
                            .on_click(cx.listener(|this, _, window, cx| this.save(window, cx))),
                    ),
            )
            // Section 1: Paths & Network
            .child(
                v_flex()
                    .gap_3()
                    .p_4()
                    .rounded(cx.theme().radius)
                    .bg(cx.theme().secondary)
                    .border_1()
                    .border_color(cx.theme().border)
                    .child(section_title("PATHS & NETWORK", cx))
                    .child(label("Download Directory", cx))
                    .child(
                        h_flex()
                            .w_full()
                            .gap_2()
                            .items_center()
                            .child(div().flex_1().child(Input::new(&self.download_dir_input)))
                            .child(
                                Button::new("browse-dir")
                                    .outline()
                                    .icon(IconName::FolderOpen)
                                    .label("Browse…")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.browse_download_dir(window, cx);
                                    })),
                            ),
                    )
                    .child(label("Network Proxy (Optional)", cx))
                    .child(Input::new(&self.proxy_input))
                    .child(label("Cookies from Browser (Optional)", cx))
                    .child(Input::new(&self.cookies_input)),
            )
            // Section 2: Concurrency & Performance
            .child(
                v_flex()
                    .gap_3()
                    .p_4()
                    .rounded(cx.theme().radius)
                    .bg(cx.theme().secondary)
                    .border_1()
                    .border_color(cx.theme().border)
                    .child(section_title("CONCURRENCY & LIMITS", cx))
                    .child(
                        h_flex()
                            .w_full()
                            .justify_between()
                            .items_center()
                            .child(
                                v_flex()
                                    .gap_0p5()
                                    .child(label("Max Concurrent Downloads", cx))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(cx.theme().muted_foreground)
                                            .child("Number of simultaneous download tasks running together"),
                                    ),
                            )
                            .child(
                                h_flex()
                                    .gap_2()
                                    .items_center()
                                    .child(
                                        Button::new("conc-dec")
                                            .ghost()
                                            .label("−")
                                            .on_click(cx.listener(|this, _, _, cx| this.bump_concurrent(-1, cx))),
                                    )
                                    .child(
                                        div()
                                            .min_w_10()
                                            .text_center()
                                            .font_semibold()
                                            .text_color(cx.theme().foreground)
                                            .child(settings.max_concurrent.to_string()),
                                    )
                                    .child(
                                        Button::new("conc-inc")
                                            .ghost()
                                            .label("+")
                                            .on_click(cx.listener(|this, _, _, cx| this.bump_concurrent(1, cx))),
                                    ),
                            ),
                    )
                    .child(
                        h_flex()
                            .w_full()
                            .justify_between()
                            .items_center()
                            .child(
                                v_flex()
                                    .gap_0p5()
                                    .child(label("Concurrent Fragments per Download", cx))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(cx.theme().muted_foreground)
                                            .child("Parallel connections per single download item"),
                                    ),
                            )
                            .child(
                                h_flex()
                                    .gap_2()
                                    .items_center()
                                    .child(
                                        Button::new("frag-dec")
                                            .ghost()
                                            .label("−")
                                            .on_click(cx.listener(|this, _, _, cx| this.bump_fragments(-1, cx))),
                                    )
                                    .child(
                                        div()
                                            .min_w_10()
                                            .text_center()
                                            .font_semibold()
                                            .text_color(cx.theme().foreground)
                                            .child(settings.concurrent_fragments.to_string()),
                                    )
                                    .child(
                                        Button::new("frag-inc")
                                            .ghost()
                                            .label("+")
                                            .on_click(cx.listener(|this, _, _, cx| this.bump_fragments(1, cx))),
                                    ),
                            ),
                    )
                    .child(
                        h_flex()
                            .w_full()
                            .justify_between()
                            .items_center()
                            .child(
                                v_flex()
                                    .gap_0p5()
                                    .child(label("Download Rate Limit", cx))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(cx.theme().muted_foreground)
                                            .child("Throttle speed in KB/s (0 = maximum speed)"),
                                    ),
                            )
                            .child(
                                h_flex()
                                    .gap_2()
                                    .items_center()
                                    .child(
                                        Button::new("rate-dec")
                                            .ghost()
                                            .label("−")
                                            .on_click(cx.listener(|this, _, _, cx| this.bump_rate_limit(-100, cx))),
                                    )
                                    .child(
                                        div()
                                            .min_w_16()
                                            .text_center()
                                            .font_semibold()
                                            .text_color(cx.theme().foreground)
                                            .child(if settings.rate_limit_kbps == 0 {
                                                "Unlimited".into()
                                            } else {
                                                format!("{} KB/s", settings.rate_limit_kbps)
                                            }),
                                    )
                                    .child(
                                        Button::new("rate-inc")
                                            .ghost()
                                            .label("+")
                                            .on_click(cx.listener(|this, _, _, cx| this.bump_rate_limit(100, cx))),
                                    ),
                            ),
                    ),
            )
            // Section 3: Appearance & Theme
            .child(
                v_flex()
                    .gap_3()
                    .p_4()
                    .rounded(cx.theme().radius)
                    .bg(cx.theme().secondary)
                    .border_1()
                    .border_color(cx.theme().border)
                    .child(section_title("APPEARANCE & THEME", cx))
                    .child(
                        h_flex()
                            .gap_2()
                            .child(
                                Button::new("theme-system")
                                    .label("System Default")
                                    .when(settings.theme_mode == ThemeMode::System, |b| b.primary())
                                    .when(settings.theme_mode != ThemeMode::System, |b| b.ghost())
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.set_theme(ThemeMode::System, cx);
                                    })),
                            )
                            .child(
                                Button::new("theme-dark")
                                    .label("Dark Theme")
                                    .when(settings.theme_mode == ThemeMode::Dark, |b| b.primary())
                                    .when(settings.theme_mode != ThemeMode::Dark, |b| b.ghost())
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.set_theme(ThemeMode::Dark, cx);
                                    })),
                            )
                            .child(
                                Button::new("theme-light")
                                    .label("Light Theme")
                                    .when(settings.theme_mode == ThemeMode::Light, |b| b.primary())
                                    .when(settings.theme_mode != ThemeMode::Light, |b| b.ghost())
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.set_theme(ThemeMode::Light, cx);
                                    })),
                            ),
                    ),
            )
            // Section 4: Defaults
            .child(
                v_flex()
                    .gap_3()
                    .p_4()
                    .rounded(cx.theme().radius)
                    .bg(cx.theme().secondary)
                    .border_1()
                    .border_color(cx.theme().border)
                    .child(section_title("DOWNLOAD & PROCESSING DEFAULTS", cx))
                    .child(
                        Switch::new("keep-originals")
                            .label("Keep original files after merge or conversion")
                            .checked(settings.keep_originals)
                            .on_click(cx.listener(|this, checked, _, cx| {
                                this.set_keep_originals(*checked, cx);
                            })),
                    )
                    .child(
                        Switch::new("write-subs-default")
                            .label("Download subtitles by default")
                            .checked(settings.write_subtitles)
                            .on_click(cx.listener(|this, checked, _, cx| {
                                this.set_write_subtitles(*checked, cx);
                            })),
                    )
                    .child(
                        Switch::new("multi-audio-default")
                            .label("Preserve all audio streams (multi-audio)")
                            .checked(settings.multi_audio)
                            .on_click(cx.listener(|this, checked, _, cx| {
                                this.set_multi_audio(*checked, cx);
                            })),
                    )
                    .child(
                        Switch::new("embed-thumb-default")
                            .label("Embed cover artwork / thumbnail")
                            .checked(settings.embed_thumbnail)
                            .on_click(cx.listener(|this, checked, _, cx| {
                                this.set_embed_thumbnail(*checked, cx);
                            })),
                    )
                    .child(
                        Switch::new("embed-meta-default")
                            .label("Embed metadata tags")
                            .checked(settings.embed_metadata)
                            .on_click(cx.listener(|this, checked, _, cx| {
                                this.set_embed_metadata(*checked, cx);
                            })),
                    )
                    .child(
                        Switch::new("prefer-vp9")
                            .label("Prefer VP9 / Opus streams when available")
                            .checked(settings.prefer_vp9_video)
                            .on_click(cx.listener(|this, checked, _, cx| {
                                this.set_prefer_vp9(*checked, cx);
                            })),
                    ),
            )
            // Section 5: Backup & Reset
            .child(
                v_flex()
                    .gap_3()
                    .p_4()
                    .rounded(cx.theme().radius)
                    .bg(cx.theme().secondary)
                    .border_1()
                    .border_color(cx.theme().border)
                    .child(section_title("BACKUP & RESTORE", cx))
                    .child(
                        h_flex()
                            .gap_2()
                            .child(
                                Button::new("export-settings-btn")
                                    .outline()
                                    .icon(IconName::Share)
                                    .label("Export Settings JSON")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.export_settings(window, cx);
                                    })),
                            )
                            .child(
                                Button::new("import-settings-btn")
                                    .outline()
                                    .icon(IconName::FolderOpen)
                                    .label("Import Settings JSON")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.import_settings(window, cx);
                                    })),
                            )
                            .child(
                                Button::new("reset-settings-btn")
                                    .ghost()
                                    .icon(IconName::RotateCw)
                                    .label("Reset Defaults")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.reset_defaults(window, cx);
                                    })),
                            ),
                    ),
            )
    }
}

fn section_title(text: &str, cx: &mut Context<SettingsView>) -> impl IntoElement {
    div()
        .text_xs()
        .font_semibold()
        .text_color(cx.theme().muted_foreground)
        .child(text.to_string())
}

fn label(text: &str, cx: &mut Context<SettingsView>) -> impl IntoElement {
    div()
        .text_sm()
        .font_semibold()
        .text_color(cx.theme().foreground)
        .child(text.to_string())
}
