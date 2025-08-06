pub mod checker;
pub mod manager;
pub mod requirements;

pub use checker::{GitHubRelease, VersionChecker, VersionInfo};
pub use manager::validate_version_requirements;
pub use requirements::{ConstraintType, VersionCheckResult, VersionConstraint, VersionRequirements};