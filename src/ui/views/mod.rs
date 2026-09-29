//! Feature views.
pub mod about;
pub mod config_view;
pub mod converter;
pub mod dashboard;
pub mod downloads;
pub mod history;
pub mod settings;

pub use about::AboutView;
pub use config_view::ConfigView;
pub use converter::ConverterView;
pub use dashboard::DashboardView;
pub use downloads::DownloadsView;
pub use history::HistoryView;
pub use settings::SettingsView;
