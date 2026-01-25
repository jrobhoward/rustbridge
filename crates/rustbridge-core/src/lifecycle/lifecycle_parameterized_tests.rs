#![allow(non_snake_case)]

use super::*;
use test_case::test_case;

// ============================================================================
// Parameterized valid transitions
// ============================================================================

#[test_case(LifecycleState::Installed, LifecycleState::Starting)]
#[test_case(LifecycleState::Starting, LifecycleState::Active)]
#[test_case(LifecycleState::Active, LifecycleState::Stopping)]
#[test_case(LifecycleState::Stopping, LifecycleState::Stopped)]
#[test_case(LifecycleState::Stopped, LifecycleState::Starting)]
#[test_case(LifecycleState::Installed, LifecycleState::Failed)]
#[test_case(LifecycleState::Starting, LifecycleState::Failed)]
#[test_case(LifecycleState::Active, LifecycleState::Failed)]
#[test_case(LifecycleState::Stopping, LifecycleState::Failed)]
fn LifecycleState___valid_transitions___allowed(from: LifecycleState, to: LifecycleState) {
    assert!(
        from.can_transition_to(to),
        "{:?} should transition to {:?}",
        from,
        to
    );
}

// ============================================================================
// Parameterized invalid transitions
// ============================================================================

#[test_case(LifecycleState::Installed, LifecycleState::Active)]
#[test_case(LifecycleState::Installed, LifecycleState::Stopping)]
#[test_case(LifecycleState::Installed, LifecycleState::Stopped)]
#[test_case(LifecycleState::Starting, LifecycleState::Stopping)]
#[test_case(LifecycleState::Starting, LifecycleState::Stopped)]
#[test_case(LifecycleState::Active, LifecycleState::Starting)]
#[test_case(LifecycleState::Active, LifecycleState::Installed)]
#[test_case(LifecycleState::Stopping, LifecycleState::Starting)]
#[test_case(LifecycleState::Stopping, LifecycleState::Active)]
#[test_case(LifecycleState::Stopped, LifecycleState::Active)]
#[test_case(LifecycleState::Stopped, LifecycleState::Stopping)]
#[test_case(LifecycleState::Failed, LifecycleState::Starting)]
#[test_case(LifecycleState::Failed, LifecycleState::Active)]
#[test_case(LifecycleState::Failed, LifecycleState::Stopping)]
#[test_case(LifecycleState::Stopped, LifecycleState::Failed)]
#[test_case(LifecycleState::Failed, LifecycleState::Failed)]
fn LifecycleState___invalid_transitions___not_allowed(from: LifecycleState, to: LifecycleState) {
    assert!(
        !from.can_transition_to(to),
        "{:?} should not transition to {:?}",
        from,
        to
    );
}

// ============================================================================
// Parameterized can_handle_requests tests
// ============================================================================

#[test_case(LifecycleState::Active, true)]
#[test_case(LifecycleState::Installed, false)]
#[test_case(LifecycleState::Starting, false)]
#[test_case(LifecycleState::Stopping, false)]
#[test_case(LifecycleState::Stopped, false)]
#[test_case(LifecycleState::Failed, false)]
fn LifecycleState___can_handle_requests___correct_state(state: LifecycleState, expected: bool) {
    assert_eq!(
        state.can_handle_requests(),
        expected,
        "State {:?} can_handle_requests should be {}",
        state,
        expected
    );
}

// ============================================================================
// Parameterized is_terminal tests
// ============================================================================

#[test_case(LifecycleState::Stopped, true)]
#[test_case(LifecycleState::Failed, true)]
#[test_case(LifecycleState::Installed, false)]
#[test_case(LifecycleState::Starting, false)]
#[test_case(LifecycleState::Active, false)]
#[test_case(LifecycleState::Stopping, false)]
fn LifecycleState___is_terminal___correct_state(state: LifecycleState, expected: bool) {
    assert_eq!(
        state.is_terminal(),
        expected,
        "State {:?} is_terminal should be {}",
        state,
        expected
    );
}

// ============================================================================
// Parameterized to_string tests (debug display)
// ============================================================================

#[test_case(LifecycleState::Installed, "Installed")]
#[test_case(LifecycleState::Starting, "Starting")]
#[test_case(LifecycleState::Active, "Active")]
#[test_case(LifecycleState::Stopping, "Stopping")]
#[test_case(LifecycleState::Stopped, "Stopped")]
#[test_case(LifecycleState::Failed, "Failed")]
fn LifecycleState___to_string___correct_representation(state: LifecycleState, expected_str: &str) {
    assert_eq!(state.to_string(), expected_str);
}
