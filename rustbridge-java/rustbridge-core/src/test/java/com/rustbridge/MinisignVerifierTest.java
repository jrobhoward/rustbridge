package com.rustbridge;

import org.junit.jupiter.api.Test;

import java.security.InvalidKeyException;
import java.security.SignatureException;
import java.util.Base64;

import static org.junit.jupiter.api.Assertions.*;

/**
 * Tests for MinisignVerifier.
 * <p>
 * Note: Full signature verification tests require real minisign key pairs.
 * These tests verify the parsing and error handling logic.
 */
class MinisignVerifierTest {

    @Test
    void constructor___invalid_base64___throws_exception() {
        assertThrows(InvalidKeyException.class, () -> {
            new MinisignVerifier("not valid base64!!!");
        });
    }

    @Test
    void constructor___empty_string___throws_exception() {
        assertThrows(Exception.class, () -> {
            new MinisignVerifier("");
        });
    }

    @Test
    void constructor___wrong_length_too_short___throws_exception() {
        // Too short (10 bytes, need 42)
        byte[] shortKey = new byte[10];
        String shortKeyBase64 = Base64.getEncoder().encodeToString(shortKey);

        InvalidKeyException exception = assertThrows(InvalidKeyException.class, () -> {
            new MinisignVerifier(shortKeyBase64);
        });

        assertTrue(exception.getMessage().contains("length") ||
                exception.getCause() != null);
    }

    @Test
    void constructor___wrong_length_too_long___throws_exception() {
        // Too long (50 bytes, need 42)
        byte[] longKey = new byte[50];
        String longKeyBase64 = Base64.getEncoder().encodeToString(longKey);

        InvalidKeyException exception = assertThrows(InvalidKeyException.class, () -> {
            new MinisignVerifier(longKeyBase64);
        });

        assertTrue(exception.getMessage().contains("length") ||
                exception.getCause() != null);
    }

    @Test
    void constructor___wrong_algorithm_id___throws_exception() {
        // 42 bytes but wrong algorithm ID (should be 0x45 0x44 for "ED")
        byte[] wrongAlgo = new byte[42];
        wrongAlgo[0] = 0x00;
        wrongAlgo[1] = 0x00;
        String wrongAlgoBase64 = Base64.getEncoder().encodeToString(wrongAlgo);

        InvalidKeyException exception = assertThrows(InvalidKeyException.class, () -> {
            new MinisignVerifier(wrongAlgoBase64);
        });

        // Should fail on algorithm ID check or key parsing
        assertNotNull(exception);
    }

    @Test
    void constructor___null_input___throws_exception() {
        assertThrows(Exception.class, () -> {
            new MinisignVerifier(null);
        });
    }

    /**
     * This test documents the expected key format.
     * A valid minisign public key is 42 bytes:
     * - 2 bytes: algorithm ID (0x45 0x44 = "ED" for Ed25519)
     * - 32 bytes: Ed25519 public key
     * - 8 bytes: key ID
     */
    @Test
    void key_format___documentation() {
        // Document the expected format
        int algorithmIdBytes = 2;
        int publicKeyBytes = 32;
        int keyIdBytes = 8;
        int totalExpectedBytes = algorithmIdBytes + publicKeyBytes + keyIdBytes;

        assertEquals(42, totalExpectedBytes, "Minisign public key should be 42 bytes");
    }

    /**
     * This test documents the expected signature format.
     * A valid minisign signature line (base64-decoded) is 74 bytes:
     * - 2 bytes: algorithm ID (0x45 0x44 = "ED" for Ed25519)
     * - 8 bytes: key ID
     * - 64 bytes: Ed25519 signature
     */
    @Test
    void signature_format___documentation() {
        // Document the expected format
        int algorithmIdBytes = 2;
        int keyIdBytes = 8;
        int signatureBytes = 64;
        int totalExpectedBytes = algorithmIdBytes + keyIdBytes + signatureBytes;

        assertEquals(74, totalExpectedBytes, "Minisign signature should be 74 bytes");
    }
}
