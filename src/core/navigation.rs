//! Application navigation routes.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AppRoute {
    #[default]
    Dashboard,
    Config,
    Downloads,
    Converter,
    History,
    Settings,
    About,
}

impl AppRoute {
    pub fn label(self) -> &'static str {
        match self {
            Self::Dashboard => "Dashboard",
            Self::Config => "Configure",
            Self::Downloads => "Downloads",
            Self::Converter => "Converter",
            Self::History => "History",
            Self::Settings => "Settings",
            Self::About => "About",
        }
    }
}
