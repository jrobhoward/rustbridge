package com.rustbridge;

import org.junit.jupiter.api.Nested;
import org.junit.jupiter.api.Test;

import java.nio.charset.StandardCharsets;
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

    // ========================================================================
    // Oracle Test Vectors (generated with minisign crate 0.8)
    // ========================================================================
    // These vectors verify that our MinisignVerifier implementation produces
    // identical results to the reference Rust minisign implementation.

    private static final byte[] ORACLE_TEST_DATA = "Hello, rustbridge!".getBytes(StandardCharsets.UTF_8);

    private static final String ORACLE_PUBLIC_KEY = "RWRX0dXiesomR/SPv8ukThZBrY9f8LrZd0XFz/H5E9jtSR0G9/sRXPu0";

    private static final String ORACLE_SIGNATURE = """
            untrusted comment: untrusted comment for test
            RURX0dXiesomR1yQGGyQgLLAGcsXIj/T/IxgxPjXBuCQ9MD/DjQtm2vxXmuM2OEvRAn36pPO92uTCBiL+na0idTmJIkn9Fnd6g4=
            trusted comment: trusted comment for test
            oqvjCoVOeFtpPv1tQ33i2+BZqHndTlsPLU+/njVMuJw6fjQs+o9O8/MSgMkvG3DqxZVFqEeQYkfuFn3h96rIDQ==""";


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
        // 42 bytes but wrong algorithm ID (should be 0x45 0x64 for "Ed")
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
     * - 2 bytes: algorithm ID (0x45 0x64 = "Ed" for Ed25519)
     * - 8 bytes: key ID
     * - 32 bytes: Ed25519 public key
     */
    @Test
    void key_format___documentation() {
        // Document the expected format
        int algorithmIdBytes = 2;
        int keyIdBytes = 8;
        int publicKeyBytes = 32;
        int totalExpectedBytes = algorithmIdBytes + keyIdBytes + publicKeyBytes;

        assertEquals(42, totalExpectedBytes, "Minisign public key should be 42 bytes");
    }

    /**
     * This test documents the expected signature format.
     * A valid minisign signature line (base64-decoded) is 74 bytes:
     * - 2 bytes: algorithm ID (0x45 0x44 = "ED" for prehashed, or 0x45 0x64 = "Ed" for legacy)
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

    /**
     * Oracle-based tests using reference vectors from Rust minisign crate.
     * These tests verify that our Java MinisignVerifier produces identical
     * results to the reference Rust implementation.
     */
    @Nested
    class OracleVectorTests {

        @Test
        void verify___oracle_valid_signature___returns_true() throws Exception {
            MinisignVerifier verifier = new MinisignVerifier(ORACLE_PUBLIC_KEY);

            boolean result = verifier.verify(ORACLE_TEST_DATA, ORACLE_SIGNATURE);

            assertTrue(result, "Valid oracle signature should verify");
        }

        @Test
        void verify___oracle_wrong_data___returns_false() throws Exception {
            MinisignVerifier verifier = new MinisignVerifier(ORACLE_PUBLIC_KEY);
            byte[] wrongData = "Hello, rustbridge?".getBytes(StandardCharsets.UTF_8); // Changed ! to ?

            boolean result = verifier.verify(wrongData, ORACLE_SIGNATURE);

            assertFalse(result, "Modified data should fail verification");
        }

        @Test
        void verify___oracle_tampered_signature___returns_false() throws Exception {
            MinisignVerifier verifier = new MinisignVerifier(ORACLE_PUBLIC_KEY);
            // Tamper with one character in the signature
            String tamperedSig = ORACLE_SIGNATURE.replace(
                    "RURX0dXiesomR1yQGGyQgLLAGcsXIj",
                    "RURX0dXiesomR1yQGGyQgLLAGcsXIk" // Changed last char
            );

            boolean result = verifier.verify(ORACLE_TEST_DATA, tamperedSig);

            assertFalse(result, "Tampered signature should fail verification");
        }
    }
}
