//! Active and queued downloads with rich progress, status badges, and actions.

use std::path::{Path, PathBuf};
use gpui_kit::component::button::Button;
use gpui_kit::component::{h_flex, v_flex, Icon};
use crate::ui::prelude::*;

use crate::core::AppState;
use crate::models::{DownloadId, DownloadStatus, DownloadTask};
use crate::services::FormatBuilder;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DownloadFilter {
    #[default]
    All,
    Active,
    Completed,
    Failed,
}

impl DownloadFilter {
    pub fn label(self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Active => "Active",
            Self::Completed => "Completed",
            Self::Failed => "Failed / Cancelled",
        }
    }

    pub fn matches(self, status: DownloadStatus) -> bool {
        match self {
            Self::All => true,
            Self::Active => !status.is_terminal(),
            Self::Completed => status == DownloadStatus::Completed,
            Self::Failed => status == DownloadStatus::Failed || status == DownloadStatus::Cancelled,
        }
    }
}

pub struct DownloadsView {
    state: Entity<AppState>,
    filter: DownloadFilter,
}

impl DownloadsView {
    pub fn new(_window: &mut Window, state: Entity<AppState>, _cx: &mut Context<Self>) -> Self {
        Self {
            state,
            filter: DownloadFilter::All,
        }
    }

    fn cancel(&mut self, id: DownloadId, cx: &mut Context<Self>) {
        self.state.update(cx, |s, cx| {
            s.downloads.cancel(id);
            cx.notify();
        });
    }

    fn retry(&mut self, id: DownloadId, cx: &mut Context<Self>) {
        self.state.update(cx, |s, cx| {
            s.downloads.retry(id);
            cx.notify();
        });
    }

    fn remove(&mut self, id: DownloadId, cx: &mut Context<Self>) {
        self.state.update(cx, |s, cx| {
            s.downloads.remove(id);
            cx.notify();
        });
    }

    fn clear_completed(&mut self, cx: &mut Context<Self>) {
        self.state.update(cx, |s, cx| {
            s.downloads.clear_completed();
            cx.notify();
        });
    }

    fn cancel_all(&mut self, cx: &mut Context<Self>) {
        self.state.update(cx, |s, cx| {
            s.downloads.cancel_all();
            cx.notify();
        });
    }

    fn reveal_file(path_opt: Option<&str>, download_dir: &Path) {
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

        // Fallback: open download folder
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            let _ = std::process::Command::new("explorer")
                .arg(download_dir.to_string_lossy().as_ref())
                .creation_flags(CREATE_NO_WINDOW)
                .spawn();
        }
        #[cfg(not(windows))]
        {
            let _ = std::process::Command::new("xdg-open").arg(download_dir).spawn();
        }
    }
}

impl Render for DownloadsView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Keep history in sync with terminal download tasks
        self.state.update(cx, |s, _| {
            s.sync_downloads_to_history();
        });

        let all_tasks = self.state.read(cx).downloads.list();
        let download_dir = self.state.read(cx).downloads.download_dir().clone();
        let total_count = all_tasks.len();
        let active_count = all_tasks.iter().filter(|t| !t.status.is_terminal()).count();
        let completed_count = all_tasks.iter().filter(|t| t.status == DownloadStatus::Completed).count();
        let failed_count = all_tasks.iter().filter(|t| t.status == DownloadStatus::Failed || t.status == DownloadStatus::Cancelled).count();

        let current_filter = self.filter;
        let filtered_tasks: Vec<DownloadTask> = all_tasks
            .into_iter()
            .filter(|t| current_filter.matches(t.status))
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
                                    .child("Downloads Queue"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(format!(
                                        "{total_count} total · {active_count} active · {completed_count} completed"
                                    )),
                            ),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .when(active_count > 0, |this| {
                                this.child(
                                    Button::new("cancel-all-btn")
                                        .danger()
                                        .icon(IconName::CircleSlash)
                                        .label("Cancel All")
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.cancel_all(cx);
                                        })),
                                )
                            })
                            .when(completed_count > 0, |this| {
                                this.child(
                                    Button::new("clear-completed-btn")
                                        .outline()
                                        .icon(IconName::Trash)
                                        .label("Clear Finished")
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.clear_completed(cx);
                                        })),
                                )
                            })
                            .child(
                                Button::new("refresh-downloads")
                                    .ghost()
                                    .icon(IconName::RefreshCw)
                                    .label("Refresh")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.state.update(cx, |s, cx| {
                                            s.sync_downloads_to_history();
                                            cx.notify();
                                        });
                                    })),
                            ),
                    ),
            )
            // Filter Tabs
            .child(
                h_flex()
                    .gap_2()
                    .p_1()
                    .rounded(cx.theme().radius)
                    .bg(cx.theme().secondary)
                    .border_1()
                    .border_color(cx.theme().border)
                    .child(
                        filter_chip("tab-all", format!("All ({total_count})"), current_filter == DownloadFilter::All, cx)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.filter = DownloadFilter::All;
                                cx.notify();
                            })),
                    )
                    .child(
                        filter_chip("tab-active", format!("Active ({active_count})"), current_filter == DownloadFilter::Active, cx)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.filter = DownloadFilter::Active;
                                cx.notify();
                            })),
                    )
                    .child(
                        filter_chip("tab-completed", format!("Completed ({completed_count})"), current_filter == DownloadFilter::Completed, cx)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.filter = DownloadFilter::Completed;
                                cx.notify();
                            })),
                    )
                    .child(
                        filter_chip("tab-failed", format!("Failed / Stopped ({failed_count})"), current_filter == DownloadFilter::Failed, cx)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.filter = DownloadFilter::Failed;
                                cx.notify();
                            })),
                    ),
            )
            // Tasks List or Empty State
            .child(
                if filtered_tasks.is_empty() {
                    crate::ui::components::empty_state::empty_state(
                        IconName::Download,
                        "No downloads found",
                        "Analyze any media or playlist URL from the Home tab to begin downloading.",
                    )
                    .into_any_element()
                } else {
                    let cards: Vec<AnyElement> = filtered_tasks
                        .into_iter()
                        .map(|task| render_download_card(task, &download_dir, cx))
                        .collect();
                    v_flex()
                        .flex_1()
                        .gap_3()
                        .overflow_y_scrollbar()
                        .children(cards)
                        .into_any_element()
                }
            )
    }
}

fn filter_chip(id: &'static str, label: String, active: bool, _cx: &App) -> Button {
    let btn = Button::new(id).label(label);
    if active {
        btn.primary()
    } else {
        btn.ghost()
    }
}

fn render_download_card(task: DownloadTask, download_dir: &PathBuf, cx: &mut Context<DownloadsView>) -> AnyElement {
    let id = task.id;
    let is_active = !task.status.is_terminal();
    let is_completed = task.status == DownloadStatus::Completed;
    let is_failed = task.status == DownloadStatus::Failed || task.status == DownloadStatus::Cancelled;

    let bar_color = match task.status {
        DownloadStatus::Completed => crate::ui::theme::PILL_GREEN,
        DownloadStatus::Failed => crate::ui::theme::PILL_RED,
        DownloadStatus::Cancelled => cx.theme().muted_foreground,
        DownloadStatus::Downloading => crate::ui::theme::CORAL,
        DownloadStatus::Processing | DownloadStatus::Analyzing => crate::ui::theme::TEAL_BRIGHT,
        DownloadStatus::Queued => cx.theme().muted_foreground,
    };

    let size_str = match (task.downloaded_bytes, task.total_bytes) {
        (d, Some(t)) if t > 0 => format!(
            "{} / {}",
            FormatBuilder::format_size(Some(d)),
            FormatBuilder::format_size(Some(t))
        ),
        (d, _) if d > 0 => FormatBuilder::format_size(Some(d)),
        _ => FormatBuilder::format_size(task.total_bytes),
    };

    let speed_eta = match (&task.speed, &task.eta) {
        (Some(s), Some(e)) => format!("{s} · ETA {e}"),
        (Some(s), None) => s.clone(),
        _ => String::new(),
    };

    let pct_clamped = (task.progress * 100.0).clamp(0.0, 100.0);
    let output_path_clone = task.output_path.clone();
    let dir_clone = download_dir.clone();

    v_flex()
        .id(format!("task-{}", id))
        .w_full()
        .p_4()
        .gap_3()
        .rounded(cx.theme().radius)
        .border_1()
        .border_color(cx.theme().border)
        .bg(cx.theme().secondary)
        // Top row: Title, Subtitle, and Action Buttons
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
                                .child(task.title.clone()),
                        )
                        .child(
                            h_flex()
                                .gap_2()
                                .items_center()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child(
                                    div()
                                        .max_w(px(380.))
                                        .text_ellipsis()
                                        .child(task.url.clone()),
                                )
                                .when_some(task.playlist_title.as_ref(), |this, pl| {
                                    this.child(div().child("·"))
                                        .child(div().text_color(cx.theme().primary).child(format!("📁 {pl}")))
                                }),
                        ),
                )
                // Action buttons on top right
                .child(
                    h_flex()
                        .gap_1p5()
                        .items_center()
                        .when(is_completed, |this| {
                            let out = output_path_clone.clone();
                            let d = dir_clone.clone();
                            this.child(
                                Button::new(format!("reveal-{id}"))
                                    .outline()
                                    .icon(IconName::FolderOpen)
                                    .label("Open")
                                    .on_click(cx.listener(move |_, _, _, _| {
                                        DownloadsView::reveal_file(out.as_deref(), &d);
                                    })),
                            )
                        })
                        .when(is_failed, |this| {
                            this.child(
                                Button::new(format!("retry-{id}"))
                                    .outline()
                                    .icon(IconName::RotateCw)
                                    .label("Retry")
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.retry(id, cx);
                                    })),
                            )
                        })
                        .when(is_active, |this| {
                            this.child(
                                Button::new(format!("cancel-{id}"))
                                    .danger()
                                    .icon(IconName::X)
                                    .label("Cancel")
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.cancel(id, cx);
                                    })),
                            )
                        })
                        .when(!is_active, |this| {
                            this.child(
                                Button::new(format!("remove-{id}"))
                                    .ghost()
                                    .icon(IconName::Trash)
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.remove(id, cx);
                                    })),
                            )
                        }),
                ),
        )
        // Middle row: Progress track bar
        .child(
            v_flex()
                .w_full()
                .gap_1p5()
                .child(
                    div()
                        .w_full()
                        .h(px(6.))
                        .rounded_full()
                        .bg(cx.theme().border)
                        .child(
                            div()
                                .h_full()
                                .rounded_full()
                                .bg(bar_color)
                                .w(relative(task.progress.clamp(0.0, 1.0) as f32)),
                        ),
                ),
        )
        // Bottom row: Badges, progress %, size, and speed/ETA
        .child(
            h_flex()
                .w_full()
                .justify_between()
                .items_center()
                .text_xs()
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .child(crate::ui::components::status_pill::download_status_pill(task.status))
                        .child(
                            div()
                                .font_medium()
                                .text_color(cx.theme().foreground)
                                .child(format!("{pct_clamped:.1}%")),
                        )
                        .child(
                            div()
                                .text_color(cx.theme().muted_foreground)
                                .child(size_str),
                        ),
                )
                .child(
                    div()
                        .text_color(cx.theme().muted_foreground)
                        .child(if speed_eta.is_empty() {
                            task.output_path.as_deref().unwrap_or_default().to_string()
                        } else {
                            speed_eta
                        }),
                ),
        )
        // Optional error banner
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
