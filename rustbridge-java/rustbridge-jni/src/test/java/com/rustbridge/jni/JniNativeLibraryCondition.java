package com.rustbridge.jni;

import org.junit.jupiter.api.extension.ConditionEvaluationResult;
import org.junit.jupiter.api.extension.ExecutionCondition;
import org.junit.jupiter.api.extension.ExtensionContext;

import java.lang.annotation.*;

/**
 * Condition that checks if the rustbridge_jni native library is available.
 * <p>
 * Tests annotated with {@link RequiresJniLibrary} will be skipped if the
 * native library cannot be loaded.
 */
public class JniNativeLibraryCondition implements ExecutionCondition {

    private static final boolean LIBRARY_AVAILABLE;
    private static final String SKIP_REASON;

    static {
        boolean available = false;
        String reason = "";
        try {
            System.loadLibrary("rustbridge_jni");
            available = true;
        } catch (UnsatisfiedLinkError e) {
            reason = "rustbridge_jni native library not available: " + e.getMessage();
        }
        LIBRARY_AVAILABLE = available;
        SKIP_REASON = reason;
    }

    /**
     * Check if the native library is available.
     *
     * @return true if the library is loaded
     */
    public static boolean isLibraryAvailable() {
        return LIBRARY_AVAILABLE;
    }

    @Override
    public ConditionEvaluationResult evaluateExecutionCondition(ExtensionContext context) {
        if (LIBRARY_AVAILABLE) {
            return ConditionEvaluationResult.enabled("JNI library available");
        }
        return ConditionEvaluationResult.disabled(SKIP_REASON);
    }

    /**
     * Annotation to mark tests that require the JNI native library.
     */
    @Target({ElementType.TYPE, ElementType.METHOD})
    @Retention(RetentionPolicy.RUNTIME)
    @org.junit.jupiter.api.extension.ExtendWith(JniNativeLibraryCondition.class)
    public @interface RequiresJniLibrary {
    }
}
