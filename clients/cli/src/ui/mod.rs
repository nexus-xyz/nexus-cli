// Module declarations
pub mod app;
pub mod dashboard;
mod login;
mod metrics;
pub mod splash;
pub mod syn_recruit;
// Re-exports for external use
pub use app::{App, UIConfig, run};
