#![allow(dead_code)]

use gpui_kit::*;

mod config;
mod core;
mod models;
mod services;
mod ui;

use ui::app_shell::AppShell;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    gpui_kit::application()
        .with_assets(gpui_kit::assets::AllAssets)
        .run(|cx| {
            // MUST be called before any component-backed views
            gpui_kit::init(cx);
            ui::theme::init_nocturnal_theme(cx);

            // Default system window decorations (minimize / maximize / close).
            // Do NOT use TitleBar::window_options() — that enables custom traffic lights.
            gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
                cx.new(|cx| AppShell::new(window, cx))
            })
            .expect("failed to open window");
        });
}
