"""Tests for LifecycleState enum."""

import pytest

from rustbridge import LifecycleState


class TestLifecycleState:
    """Tests for LifecycleState enum."""

    def test_from_code___valid_codes___returns_correct_state(self) -> None:
        assert LifecycleState.from_code(0) == LifecycleState.INSTALLED
        assert LifecycleState.from_code(1) == LifecycleState.STARTING
        assert LifecycleState.from_code(2) == LifecycleState.ACTIVE
        assert LifecycleState.from_code(3) == LifecycleState.STOPPING
        assert LifecycleState.from_code(4) == LifecycleState.STOPPED
        assert LifecycleState.from_code(5) == LifecycleState.FAILED

    def test_from_code___invalid_code___raises_value_error(self) -> None:
        with pytest.raises(ValueError, match="Invalid lifecycle state code"):
            LifecycleState.from_code(6)

        with pytest.raises(ValueError, match="Invalid lifecycle state code"):
            LifecycleState.from_code(-1)

    def test_can_handle_requests___active___returns_true(self) -> None:
        assert LifecycleState.ACTIVE.can_handle_requests() is True

    def test_can_handle_requests___other_states___returns_false(self) -> None:
        assert LifecycleState.INSTALLED.can_handle_requests() is False
        assert LifecycleState.STARTING.can_handle_requests() is False
        assert LifecycleState.STOPPING.can_handle_requests() is False
        assert LifecycleState.STOPPED.can_handle_requests() is False
        assert LifecycleState.FAILED.can_handle_requests() is False

    def test_is_terminal___stopped_and_failed___returns_true(self) -> None:
        assert LifecycleState.STOPPED.is_terminal() is True
        assert LifecycleState.FAILED.is_terminal() is True

    def test_is_terminal___non_terminal_states___returns_false(self) -> None:
        assert LifecycleState.INSTALLED.is_terminal() is False
        assert LifecycleState.STARTING.is_terminal() is False
        assert LifecycleState.ACTIVE.is_terminal() is False
        assert LifecycleState.STOPPING.is_terminal() is False

    def test_int_value___matches_ffi_codes(self) -> None:
        assert int(LifecycleState.INSTALLED) == 0
        assert int(LifecycleState.STARTING) == 1
        assert int(LifecycleState.ACTIVE) == 2
        assert int(LifecycleState.STOPPING) == 3
        assert int(LifecycleState.STOPPED) == 4
        assert int(LifecycleState.FAILED) == 5
