//! Plugin lifecycle state machine (OSGI-inspired)

use serde::{Deserialize, Serialize};

/// Plugin lifecycle states following OSGI-inspired model
///
/// State transitions:
/// ```text
/// Installed → Starting → Active → Stopping → Stopped
///                ↑                    │
///                └────────────────────┘ (restart)
///            Any state → Failed (on error)
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleState {
    /// Plugin is installed but not yet initialized
    #[default]
    Installed,
    /// Plugin is starting up (initializing runtime, resources)
    Starting,
    /// Plugin is fully operational and ready to handle requests
    Active,
    /// Plugin is shutting down (cleaning up resources)
    Stopping,
    /// Plugin has been stopped
    Stopped,
    /// Plugin encountered a fatal error
    Failed,
}

impl LifecycleState {
    /// Check if this state can transition to the target state
    pub fn can_transition_to(&self, target: LifecycleState) -> bool {
        use LifecycleState::*;
        matches!(
            (self, target),
            // Normal lifecycle transitions
            (Installed, Starting)
                | (Starting, Active)
                | (Active, Stopping)
                | (Stopping, Stopped)
                // Restart from stopped
                | (Stopped, Starting)
                // Any state can fail
                | (Installed, Failed)
                | (Starting, Failed)
                | (Active, Failed)
                | (Stopping, Failed)
        )
    }

    /// Check if the plugin can handle requests in this state
    pub fn can_handle_requests(&self) -> bool {
        matches!(self, LifecycleState::Active)
    }

    /// Check if the plugin is in a terminal state
    pub fn is_terminal(&self) -> bool {
        matches!(self, LifecycleState::Stopped | LifecycleState::Failed)
    }

    /// Get a human-readable description of this state
    pub fn description(&self) -> &'static str {
        match self {
            LifecycleState::Installed => "Plugin is installed but not initialized",
            LifecycleState::Starting => "Plugin is starting up",
            LifecycleState::Active => "Plugin is active and ready",
            LifecycleState::Stopping => "Plugin is shutting down",
            LifecycleState::Stopped => "Plugin has stopped",
            LifecycleState::Failed => "Plugin has failed",
        }
    }
}

impl std::fmt::Display for LifecycleState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LifecycleState::Installed => write!(f, "Installed"),
            LifecycleState::Starting => write!(f, "Starting"),
            LifecycleState::Active => write!(f, "Active"),
            LifecycleState::Stopping => write!(f, "Stopping"),
            LifecycleState::Stopped => write!(f, "Stopped"),
            LifecycleState::Failed => write!(f, "Failed"),
        }
    }
}

#[cfg(test)]
#[path = "lifecycle/lifecycle_tests.rs"]
mod lifecycle_tests;
