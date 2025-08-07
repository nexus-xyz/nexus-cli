//! Modular dashboard implementation
//!
//! Split into logical modules for better maintainability

pub mod components;
pub mod renderer;
pub mod state;
pub mod updaters;
pub mod utils;

// Re-export main types and functions for external use
pub use renderer::render_dashboard;
pub use state::DashboardState;
