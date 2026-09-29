//! Core: errors, events, navigation, and shared application state.
pub mod app_state;
pub mod errors;
pub mod events;
pub mod navigation;

pub use app_state::{AppState, CurrentMedia};
pub use errors::{AppError, Result};
#[allow(unused_imports)]
pub use events::{AnalysisResult, AppEvent, EventBus, ToastKind};
pub use navigation::AppRoute;
