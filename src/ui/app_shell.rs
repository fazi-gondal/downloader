//! Application shell: sidebar navigation + content area.
//!
//! Owns the shared AppState Entity and passes clones to feature views.
//! Uses default system window decorations (minimize / maximize / close).

use gpui_kit::component::sidebar::{
    Sidebar, SidebarFooter, SidebarGroup, SidebarHeader, SidebarMenu, SidebarMenuItem,
};
use gpui_kit::component::button::Button;
use gpui_kit::component::{h_flex, Icon};
use crate::ui::prelude::*;

use crate::core::{AppRoute, AppState};
use crate::ui::views::{
    AboutView, ConfigView, ConverterView, DashboardView, DownloadsView, HistoryView, SettingsView,
};

pub struct AppShell {
    route: AppRoute,
    #[allow(dead_code)]
    state: Entity<AppState>,
    dashboard: Entity<DashboardView>,
    config: Entity<ConfigView>,
    downloads: Entity<DownloadsView>,
    converter: Entity<ConverterView>,
    history: Entity<HistoryView>,
    settings: Entity<SettingsView>,
    about: Entity<AboutView>,
}

impl AppShell {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let state = cx.new(|_| AppState::new());

        Self {
            route: AppRoute::Dashboard,
            state: state.clone(),
            dashboard: cx.new(|cx| DashboardView::new(window, state.clone(), cx)),
            config: cx.new(|cx| ConfigView::new(window, state.clone(), cx)),
            downloads: cx.new(|cx| DownloadsView::new(window, state.clone(), cx)),
            converter: cx.new(|cx| ConverterView::new(window, state.clone(), cx)),
            history: cx.new(|cx| HistoryView::new(window, state.clone(), cx)),
            settings: cx.new(|cx| SettingsView::new(window, state.clone(), cx)),
            about: cx.new(|cx| AboutView::new(window, cx)),
        }
    }

    fn navigate(&mut self, route: AppRoute, cx: &mut Context<Self>) {
        self.state.update(cx, |s, cx| {
            s.active_route = route;
            cx.notify();
        });
        if self.route != route {
            self.route = route;
            cx.notify();
        }
    }

    fn sidebar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let active = self.route;
        let s = self.state.read(cx);
        let active_downloads = s
            .downloads
            .list()
            .iter()
            .filter(|t| !t.status.is_terminal())
            .count();
        let downloads_title = if active_downloads > 0 {
            format!("Downloads ({active_downloads})")
        } else {
            "Downloads".to_string()
        };

        let ffmpeg_ok = crate::services::FfmpegService::default()
            .check_available()
            .is_ok();

        Sidebar::new("app-sidebar")
            .w(px(240.))
            .header(
                SidebarHeader::new().child(
                    h_flex()
                        .gap_3()
                        .items_center()
                        .child(
                            div()
                                .size(px(38.))
                                .rounded(crate::ui::theme::RADIUS_CONTROL)
                                .bg(crate::ui::theme::CORAL)
                                .flex()
                                .items_center()
                                .justify_center()
                                .child(
                                    Icon::new(IconName::Download)
                                        .size(px(20.))
                                        .text_color(crate::ui::theme::TEXT_PRIMARY),
                                ),
                        )
                        .child(
                            gpui_kit::component::v_flex()
                                .gap(px(1.))
                                .child(
                                    div()
                                        .text_sm()
                                        .font_bold()
                                        .text_color(crate::ui::theme::TEXT_PRIMARY)
                                        .child("Video Downloader"),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(crate::ui::theme::TEXT_MUTED)
                                        .child("Pro Studio Suite"),
                                ),
                        ),
                ),
            )
            .child(
                SidebarGroup::new("Library").child(
                    SidebarMenu::new()
                        .child(
                            SidebarMenuItem::new("Home")
                                .icon(IconName::House)
                                .active(active == AppRoute::Dashboard)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.navigate(AppRoute::Dashboard, cx);
                                })),
                        )
                        .child(
                            SidebarMenuItem::new("Configure")
                                .icon(IconName::SlidersHorizontal)
                                .active(active == AppRoute::Config)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.navigate(AppRoute::Config, cx);
                                })),
                        )
                        .child(
                            SidebarMenuItem::new(downloads_title)
                                .icon(IconName::Download)
                                .active(active == AppRoute::Downloads)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.navigate(AppRoute::Downloads, cx);
                                })),
                        )
                        .child(
                            SidebarMenuItem::new("Converter")
                                .icon(IconName::RefreshCw)
                                .active(active == AppRoute::Converter)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.navigate(AppRoute::Converter, cx);
                                })),
                        )
                        .child(
                            SidebarMenuItem::new("History")
                                .icon(IconName::Clock)
                                .active(active == AppRoute::History)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.navigate(AppRoute::History, cx);
                                })),
                        ),
                ),
            )
            .child(
                SidebarGroup::new("System").child(
                    SidebarMenu::new()
                        .child(
                            SidebarMenuItem::new("Settings")
                                .icon(IconName::Settings)
                                .active(active == AppRoute::Settings)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.navigate(AppRoute::Settings, cx);
                                })),
                        )
                        .child(
                            SidebarMenuItem::new("About")
                                .icon(IconName::Info)
                                .active(active == AppRoute::About)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.navigate(AppRoute::About, cx);
                                })),
                        ),
                ),
            )
            .footer(
                SidebarFooter::new().child(
                    gpui_kit::component::v_flex()
                        .w_full()
                        .gap_2()
                        .child(
                            h_flex()
                                .px_3()
                                .py_1p5()
                                .rounded(crate::ui::theme::RADIUS_CONTROL)
                                .bg(crate::ui::theme::SURFACE_INSET)
                                .border_1()
                                .border_color(crate::ui::theme::BORDER_SUBTLE)
                                .items_center()
                                .gap_2()
                                .child(
                                    div()
                                        .size(px(8.))
                                        .rounded_full()
                                        .bg(if ffmpeg_ok {
                                            crate::ui::theme::PILL_GREEN
                                        } else {
                                            crate::ui::theme::PILL_RED
                                        }),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .font_medium()
                                        .text_color(crate::ui::theme::TEXT_MUTED)
                                        .child(if ffmpeg_ok {
                                            "ffmpeg: OK"
                                        } else {
                                            "ffmpeg: missing"
                                        }),
                                ),
                        )
                        .child(
                            Button::new("theme-toggle")
                                .ghost()
                                .w_full()
                                .icon(IconName::SunMedium)
                                .label("Toggle Theme")
                                .on_click(cx.listener(|_this, _, window, cx| {
                                    let is_dark = cx.theme().mode.is_dark();
                                    let new_mode = if is_dark {
                                        gpui_kit::component::ThemeMode::Light
                                    } else {
                                        gpui_kit::component::ThemeMode::Dark
                                    };
                                    gpui_kit::component::Theme::change(new_mode, None, cx);
                                    window.refresh();
                                })),
                        ),
                ),
            )
    }
}

impl Render for AppShell {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let route = self.state.read(cx).active_route;
        if self.route != route {
            self.route = route;
        }

        let content = match self.route {
            AppRoute::Dashboard => self.dashboard.clone().into_any_element(),
            AppRoute::Config => self.config.clone().into_any_element(),
            AppRoute::Downloads => self.downloads.clone().into_any_element(),
            AppRoute::Converter => self.converter.clone().into_any_element(),
            AppRoute::History => self.history.clone().into_any_element(),
            AppRoute::Settings => self.settings.clone().into_any_element(),
            AppRoute::About => self.about.clone().into_any_element(),
        };

        h_flex()
            .size_full()
            .bg(cx.theme().background)
            .child(self.sidebar(cx))
            .child(
                div()
                    .flex_1()
                    .h_full()
                    .overflow_hidden()
                    .child(content),
            )
    }
}
