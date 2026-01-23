#![allow(non_snake_case)]

use super::*;

// Valid transitions

#[test]
fn LifecycleState___installed_to_starting___transition_allowed() {
    let state = LifecycleState::Installed;

    let can_transition = state.can_transition_to(LifecycleState::Starting);

    assert!(can_transition);
}

#[test]
fn LifecycleState___starting_to_active___transition_allowed() {
    let state = LifecycleState::Starting;

    let can_transition = state.can_transition_to(LifecycleState::Active);

    assert!(can_transition);
}

#[test]
fn LifecycleState___active_to_stopping___transition_allowed() {
    let state = LifecycleState::Active;

    let can_transition = state.can_transition_to(LifecycleState::Stopping);

    assert!(can_transition);
}

#[test]
fn LifecycleState___stopping_to_stopped___transition_allowed() {
    let state = LifecycleState::Stopping;

    let can_transition = state.can_transition_to(LifecycleState::Stopped);

    assert!(can_transition);
}

#[test]
fn LifecycleState___stopped_to_starting___restart_allowed() {
    let state = LifecycleState::Stopped;

    let can_transition = state.can_transition_to(LifecycleState::Starting);

    assert!(can_transition);
}

#[test]
fn LifecycleState___any_state_to_failed___transition_allowed() {
    let states = [
        LifecycleState::Installed,
        LifecycleState::Starting,
        LifecycleState::Active,
        LifecycleState::Stopping,
    ];

    for state in states {
        let can_transition = state.can_transition_to(LifecycleState::Failed);

        assert!(
            can_transition,
            "{:?} should be able to transition to Failed",
            state
        );
    }
}

// Invalid transitions

#[test]
fn LifecycleState___installed_to_active___skip_not_allowed() {
    let state = LifecycleState::Installed;

    let can_transition = state.can_transition_to(LifecycleState::Active);

    assert!(!can_transition);
}

#[test]
fn LifecycleState___starting_to_stopping___skip_not_allowed() {
    let state = LifecycleState::Starting;

    let can_transition = state.can_transition_to(LifecycleState::Stopping);

    assert!(!can_transition);
}

#[test]
fn LifecycleState___active_to_starting___backwards_not_allowed() {
    let state = LifecycleState::Active;

    let can_transition = state.can_transition_to(LifecycleState::Starting);

    assert!(!can_transition);
}

#[test]
fn LifecycleState___failed_to_starting___recovery_not_allowed() {
    let state = LifecycleState::Failed;

    let can_transition = state.can_transition_to(LifecycleState::Starting);

    assert!(!can_transition);
}

// can_handle_requests

#[test]
fn LifecycleState___active___can_handle_requests() {
    let state = LifecycleState::Active;

    assert!(state.can_handle_requests());
}

#[test]
fn LifecycleState___non_active_states___cannot_handle_requests() {
    let states = [
        LifecycleState::Installed,
        LifecycleState::Starting,
        LifecycleState::Stopping,
        LifecycleState::Stopped,
        LifecycleState::Failed,
    ];

    for state in states {
        assert!(
            !state.can_handle_requests(),
            "{:?} should not handle requests",
            state
        );
    }
}

// is_terminal

#[test]
fn LifecycleState___stopped___is_terminal() {
    let state = LifecycleState::Stopped;

    assert!(state.is_terminal());
}

#[test]
fn LifecycleState___failed___is_terminal() {
    let state = LifecycleState::Failed;

    assert!(state.is_terminal());
}

#[test]
fn LifecycleState___active___not_terminal() {
    let state = LifecycleState::Active;

    assert!(!state.is_terminal());
}

// Default

#[test]
fn LifecycleState___default___returns_installed() {
    let state = LifecycleState::default();

    assert_eq!(state, LifecycleState::Installed);
}

// Display

#[test]
fn LifecycleState___display___shows_name() {
    assert_eq!(LifecycleState::Active.to_string(), "Active");
    assert_eq!(LifecycleState::Stopped.to_string(), "Stopped");
}
