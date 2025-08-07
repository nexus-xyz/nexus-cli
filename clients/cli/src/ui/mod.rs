// Module declarations
mod app;
pub mod dashboard;
mod login;
mod metrics;
pub mod splash;
// Re-exports for external use
pub use app::{App, UIConfig, run};
