package com.rustbridge;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.CsvSource;
import org.junit.jupiter.params.provider.ValueSource;

import static org.junit.jupiter.api.Assertions.*;

class LifecycleStateTest {

    @ParameterizedTest
    @CsvSource({
            "0, INSTALLED",
            "1, STARTING",
            "2, ACTIVE",
            "3, STOPPING",
            "4, STOPPED",
            "5, FAILED"
    })
    void fromCode___valid_code___returns_correct_state(int code, LifecycleState expected) {
        LifecycleState state = LifecycleState.fromCode(code);

        assertEquals(expected, state);
    }

    @ParameterizedTest
    @ValueSource(ints = {-1, 6, 100, 255})
    void fromCode___invalid_code___throws_exception(int code) {
        assertThrows(IllegalArgumentException.class, () -> {
            LifecycleState.fromCode(code);
        });
    }

    @Test
    void getCode___returns_correct_code() {
        assertEquals(0, LifecycleState.INSTALLED.getCode());
        assertEquals(1, LifecycleState.STARTING.getCode());
        assertEquals(2, LifecycleState.ACTIVE.getCode());
        assertEquals(3, LifecycleState.STOPPING.getCode());
        assertEquals(4, LifecycleState.STOPPED.getCode());
        assertEquals(5, LifecycleState.FAILED.getCode());
    }

    @Test
    void canHandleRequests___only_active___returns_true() {
        assertFalse(LifecycleState.INSTALLED.canHandleRequests());
        assertFalse(LifecycleState.STARTING.canHandleRequests());
        assertTrue(LifecycleState.ACTIVE.canHandleRequests());
        assertFalse(LifecycleState.STOPPING.canHandleRequests());
        assertFalse(LifecycleState.STOPPED.canHandleRequests());
        assertFalse(LifecycleState.FAILED.canHandleRequests());
    }

    @Test
    void isTerminal___stopped_and_failed___returns_true() {
        assertFalse(LifecycleState.INSTALLED.isTerminal());
        assertFalse(LifecycleState.STARTING.isTerminal());
        assertFalse(LifecycleState.ACTIVE.isTerminal());
        assertFalse(LifecycleState.STOPPING.isTerminal());
        assertTrue(LifecycleState.STOPPED.isTerminal());
        assertTrue(LifecycleState.FAILED.isTerminal());
    }

    @Test
    void roundTrip___code_to_state_and_back() {
        for (LifecycleState state : LifecycleState.values()) {
            int code = state.getCode();
            LifecycleState recovered = LifecycleState.fromCode(code);

            assertEquals(state, recovered);
        }
    }
}
