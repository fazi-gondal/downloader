//! Download history (persisted JSON).

use std::path::{Path, PathBuf};
use gpui_kit::component::button::Button;
use gpui_kit::component::{h_flex, v_flex};
use crate::ui::prelude::*;
use uuid::Uuid;

use crate::core::AppState;
use crate::models::HistoryEntry;
use crate::services::FormatBuilder;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HistoryFilter {
    #[default]
    All,
    Successful,
    Failed,
}

pub struct HistoryView {
    state: Entity<AppState>,
    filter: HistoryFilter,
}

impl HistoryView {
    pub fn new(_window: &mut Window, state: Entity<AppState>, _cx: &mut Context<Self>) -> Self {
        Self {
            state,
            filter: HistoryFilter::All,
        }
    }

    fn clear_all(&mut self, cx: &mut Context<Self>) {
        self.state.update(cx, |s, cx| {
            s.history.clear();
            cx.notify();
        });
    }

    fn remove(&mut self, id: Uuid, cx: &mut Context<Self>) {
        self.state.update(cx, |s, cx| {
            s.history.remove(id);
            cx.notify();
        });
    }

    fn redownload(&mut self, url: String, title: String, window: &mut Window, cx: &mut Context<Self>) {
        self.state.update(cx, |s, cx| {
            s.downloads.enqueue(url.clone(), s.current_options.clone(), None);
            s.active_route = crate::core::AppRoute::Downloads;
            cx.notify();
        });
        window.push_notification(format!("Re-enqueued download: {title}"), cx);
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

impl Render for HistoryView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let all_entries: Vec<HistoryEntry> = self.state.read(cx).history.list().to_vec();
        let download_dir = self.state.read(cx).downloads.download_dir().clone();

        let total_count = all_entries.len();
        let success_count = all_entries.iter().filter(|e| e.success).count();
        let failed_count = all_entries.iter().filter(|e| !e.success).count();

        let current_filter = self.filter;
        let filtered: Vec<HistoryEntry> = all_entries
            .into_iter()
            .filter(|e| match current_filter {
                HistoryFilter::All => true,
                HistoryFilter::Successful => e.success,
                HistoryFilter::Failed => !e.success,
            })
            .collect();

        let cards: Vec<AnyElement> = filtered
            .into_iter()
            .map(|entry| render_history_card(entry, &download_dir, cx))
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
                                    .child("Download History"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(format!(
                                        "{total_count} logged · {success_count} successful · {failed_count} failed"
                                    )),
                            ),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .when(total_count > 0, |this| {
                                this.child(
                                    Button::new("clear-history")
                                        .outline()
                                        .icon(IconName::Trash)
                                        .label("Clear History")
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.clear_all(cx);
                                        })),
                                )
                            }),
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
                        Button::new("hist-filter-all")
                            .label(format!("All ({total_count})"))
                            .when(current_filter == HistoryFilter::All, |b| b.primary())
                            .when(current_filter != HistoryFilter::All, |b| b.ghost())
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.filter = HistoryFilter::All;
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("hist-filter-success")
                            .label(format!("Successful ({success_count})"))
                            .when(current_filter == HistoryFilter::Successful, |b| b.primary())
                            .when(current_filter != HistoryFilter::Successful, |b| b.ghost())
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.filter = HistoryFilter::Successful;
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("hist-filter-failed")
                            .label(format!("Failed ({failed_count})"))
                            .when(current_filter == HistoryFilter::Failed, |b| b.primary())
                            .when(current_filter != HistoryFilter::Failed, |b| b.ghost())
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.filter = HistoryFilter::Failed;
                                cx.notify();
                            })),
                    ),
            )
            // History list or Empty State
            .child(
                if cards.is_empty() {
                    crate::ui::components::empty_state::empty_state(
                        IconName::Clock,
                        "No history records",
                        "Finished downloads will automatically be preserved in this log.",
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

fn render_history_card(
    entry: HistoryEntry,
    download_dir: &PathBuf,
    cx: &mut Context<HistoryView>,
) -> AnyElement {
    let id = entry.id;
    let when = entry
        .completed_at
        .format("%Y-%m-%d %H:%M")
        .to_string();
    let size = FormatBuilder::format_size(entry.file_size);

    let url_clone = entry.url.clone();
    let title_clone = entry.title.clone();
    let out_clone = entry.output_path.clone();
    let dir_clone = download_dir.clone();

    h_flex()
        .id(format!("hist-item-{}", id))
        .w_full()
        .p_3()
        .gap_3()
        .items_center()
        .rounded(cx.theme().radius)
        .border_1()
        .border_color(cx.theme().border)
        .bg(cx.theme().secondary)
        .child(
            v_flex()
                .flex_1()
                .gap_1()
                .child(
                    div()
                        .font_semibold()
                        .text_sm()
                        .text_color(cx.theme().foreground)
                        .child(entry.title.clone()),
                )
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .text_xs()
                        .child(crate::ui::components::status_pill::status_pill(
                            if entry.success { "Completed" } else { "Failed" },
                            if entry.success {
                                crate::ui::theme::PILL_GREEN
                            } else {
                                crate::ui::theme::PILL_RED
                            },
                        ))
                        .child(div().child("·"))
                        .child(
                            div()
                                .text_color(cx.theme().muted_foreground)
                                .child(when),
                        )
                        .child(div().child("·"))
                        .child(
                            div()
                                .text_color(cx.theme().muted_foreground)
                                .child(size),
                        ),
                )
                .when_some(entry.output_path.as_ref(), |this, out| {
                    this.child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(out.clone()),
                    )
                })
                .when_some(entry.error.as_ref(), |this, err| {
                    this.child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().danger)
                            .child(err.clone()),
                    )
                }),
        )
        // Actions: Open, Redownload, Remove
        .child(
            h_flex()
                .gap_1p5()
                .items_center()
                .when(entry.output_path.is_some(), |this| {
                    let out = out_clone.clone();
                    let d = dir_clone.clone();
                    this.child(
                        Button::new(format!("hist-open-{id}"))
                            .outline()
                            .icon(IconName::FolderOpen)
                            .label("Open")
                            .on_click(cx.listener(move |_, _, _, _| {
                                HistoryView::reveal_file(out.as_deref(), &d);
                            })),
                    )
                })
                .child(
                    Button::new(format!("hist-redownload-{id}"))
                        .outline()
                        .icon(IconName::RotateCw)
                        .label("Re-download")
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.redownload(url_clone.clone(), title_clone.clone(), window, cx);
                        })),
                )
                .child(
                    Button::new(format!("hist-rm-{id}"))
                        .ghost()
                        .icon(IconName::Trash)
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.remove(id, cx);
                        })),
                ),
        )
        .into_any_element()
}
