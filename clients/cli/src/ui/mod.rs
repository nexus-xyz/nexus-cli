// Module declarations
mod app;
pub mod dashboard;
mod login;
mod metrics;
pub mod splash;
mod stages;

// Re-exports for external use
pub use app::{App, run};
