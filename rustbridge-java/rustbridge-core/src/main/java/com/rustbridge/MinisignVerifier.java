package com.rustbridge;

import net.i2p.crypto.eddsa.EdDSAEngine;
import net.i2p.crypto.eddsa.EdDSAPublicKey;
import net.i2p.crypto.eddsa.spec.EdDSANamedCurveTable;
import net.i2p.crypto.eddsa.spec.EdDSAParameterSpec;
import net.i2p.crypto.eddsa.spec.EdDSAPublicKeySpec;

import java.security.*;
import java.util.Arrays;
import java.util.Base64;

/**
 * Minisign signature verification using Ed25519.
 *
 * <p>This class parses and verifies minisign signatures.
 * Minisign uses Ed25519 signatures with a specific file format.
 *
 * <h2>Minisign Format</h2>
 * <ul>
 *   <li>Public key: Base64-encoded 42 bytes (2-byte algorithm ID + 32-byte key + 8-byte key ID)</li>
 *   <li>Signature: Base64-encoded 74 bytes (2-byte algorithm ID + 8-byte key ID + 64-byte signature)</li>
 * </ul>
 *
 * @see <a href="https://jedisct1.github.io/minisign/">Minisign</a>
 */
public class MinisignVerifier {
    private static final int ED25519_PUBLIC_KEY_BYTES = 32;
    private static final int ED25519_SIGNATURE_BYTES = 64;
    private static final int KEY_ID_BYTES = 8;
    private static final int ALGORITHM_ID_BYTES = 2;

    // Expected algorithm ID for Ed25519 (0x4445 = "Ed" in ASCII)
    private static final byte[] ED25519_ALGORITHM_ID = {0x45, 0x44};

    private final PublicKey publicKey;
    private final byte[] keyId;

    /**
     * Create a verifier from a minisign public key string.
     *
     * @param publicKeyBase64 Minisign public key in base64 format (e.g., "RWS...")
     * @throws InvalidKeyException if the key format is invalid
     */
    public MinisignVerifier(String publicKeyBase64) throws InvalidKeyException {
        this.publicKey = parsePublicKey(publicKeyBase64);
        this.keyId = extractKeyId(publicKeyBase64);
    }

    /**
     * Parse a minisign public key from base64 format.
     *
     * <p>Format: 2 bytes algorithm ID + 32 bytes public key + 8 bytes key ID
     *
     * @param publicKeyBase64 base64-encoded public key
     * @return EdDSA public key
     * @throws InvalidKeyException if the key format is invalid
     */
    private static PublicKey parsePublicKey(String publicKeyBase64) throws InvalidKeyException {
        try {
            byte[] decoded = Base64.getDecoder().decode(publicKeyBase64.trim());

            if (decoded.length != ALGORITHM_ID_BYTES + ED25519_PUBLIC_KEY_BYTES + KEY_ID_BYTES) {
                throw new InvalidKeyException(
                        "Invalid public key length: expected " +
                                (ALGORITHM_ID_BYTES + ED25519_PUBLIC_KEY_BYTES + KEY_ID_BYTES) +
                                ", got " + decoded.length
                );
            }

            // Verify algorithm ID
            byte[] algorithmId = Arrays.copyOfRange(decoded, 0, ALGORITHM_ID_BYTES);
            if (!Arrays.equals(algorithmId, ED25519_ALGORITHM_ID)) {
                throw new InvalidKeyException(
                        "Invalid algorithm ID: expected Ed25519, got " +
                                bytesToHex(algorithmId)
                );
            }

            // Extract the 32-byte Ed25519 public key
            byte[] keyBytes = Arrays.copyOfRange(
                    decoded,
                    ALGORITHM_ID_BYTES,
                    ALGORITHM_ID_BYTES + ED25519_PUBLIC_KEY_BYTES
            );

            // Create EdDSA public key using i2p library
            EdDSAParameterSpec spec = EdDSANamedCurveTable.getByName("Ed25519");
            EdDSAPublicKeySpec pubKeySpec = new EdDSAPublicKeySpec(keyBytes, spec);
            return new EdDSAPublicKey(pubKeySpec);
        } catch (Exception e) {
            throw new InvalidKeyException("Failed to parse public key", e);
        }
    }

    /**
     * Extract the key ID from a minisign public key.
     *
     * @param publicKeyBase64 base64-encoded public key
     * @return 8-byte key ID
     * @throws InvalidKeyException if the key format is invalid
     */
    private static byte[] extractKeyId(String publicKeyBase64) throws InvalidKeyException {
        byte[] decoded = Base64.getDecoder().decode(publicKeyBase64.trim());
        if (decoded.length != ALGORITHM_ID_BYTES + ED25519_PUBLIC_KEY_BYTES + KEY_ID_BYTES) {
            throw new InvalidKeyException("Invalid public key length");
        }
        return Arrays.copyOfRange(
                decoded,
                ALGORITHM_ID_BYTES + ED25519_PUBLIC_KEY_BYTES,
                decoded.length
        );
    }

    /**
     * Parse a minisign signature from the multi-line format.
     *
     * <p>Format:
     * <pre>
     * untrusted comment: &lt;comment&gt;
     * &lt;base64-encoded signature&gt;
     * trusted comment: &lt;comment&gt;
     * &lt;base64-encoded global signature&gt;
     * </pre>
     *
     * <p>We only use the second line (the signature itself).
     *
     * @param signatureString multi-line signature string
     * @return parsed signature components
     * @throws SignatureException if parsing fails
     */
    private static ParsedSignature parseSignature(String signatureString)
            throws SignatureException {
        String[] lines = signatureString.split("\n");
        if (lines.length < 2) {
            throw new SignatureException("Invalid signature format: expected at least 2 lines");
        }

        // The signature is on the second line
        String signatureBase64 = lines[1].trim();

        try {
            byte[] decoded = Base64.getDecoder().decode(signatureBase64);

            if (decoded.length != ALGORITHM_ID_BYTES + KEY_ID_BYTES + ED25519_SIGNATURE_BYTES) {
                throw new SignatureException(
                        "Invalid signature length: expected " +
                                (ALGORITHM_ID_BYTES + KEY_ID_BYTES + ED25519_SIGNATURE_BYTES) +
                                ", got " + decoded.length
                );
            }

            // Verify algorithm ID
            byte[] algorithmId = Arrays.copyOfRange(decoded, 0, ALGORITHM_ID_BYTES);
            if (!Arrays.equals(algorithmId, ED25519_ALGORITHM_ID)) {
                throw new SignatureException(
                        "Invalid algorithm ID in signature: expected Ed25519, got " +
                                bytesToHex(algorithmId)
                );
            }

            // Extract key ID
            byte[] keyId = Arrays.copyOfRange(
                    decoded,
                    ALGORITHM_ID_BYTES,
                    ALGORITHM_ID_BYTES + KEY_ID_BYTES
            );

            // Extract signature
            byte[] signature = Arrays.copyOfRange(
                    decoded,
                    ALGORITHM_ID_BYTES + KEY_ID_BYTES,
                    decoded.length
            );

            return new ParsedSignature(keyId, signature);
        } catch (IllegalArgumentException e) {
            throw new SignatureException("Invalid base64 encoding in signature", e);
        }
    }

    /**
     * Convert bytes to hex string for debugging.
     */
    private static String bytesToHex(byte[] bytes) {
        StringBuilder sb = new StringBuilder();
        for (byte b : bytes) {
            sb.append(String.format("%02x", b));
        }
        return sb.toString();
    }

    /**
     * Verify a minisign signature against data.
     *
     * @param data            the data that was signed
     * @param signatureString the minisign signature (multi-line format)
     * @return true if the signature is valid, false otherwise
     * @throws SignatureException if signature verification fails
     */
    public boolean verify(byte[] data, String signatureString) throws SignatureException {
        try {
            // Parse the signature
            ParsedSignature sig = parseSignature(signatureString);

            // Verify key ID matches
            if (!Arrays.equals(this.keyId, sig.keyId)) {
                return false;
            }

            // Verify the signature using Ed25519
            Signature verifier = new EdDSAEngine(MessageDigest.getInstance("SHA-512"));
            verifier.initVerify(publicKey);
            verifier.update(data);
            return verifier.verify(sig.signature);
        } catch (NoSuchAlgorithmException | InvalidKeyException e) {
            throw new SignatureException("Ed25519 verification failed", e);
        }
    }

    /**
     * Parsed signature components.
     */
    private static class ParsedSignature {
        final byte[] keyId;
        final byte[] signature;

        ParsedSignature(byte[] keyId, byte[] signature) {
            this.keyId = keyId;
            this.signature = signature;
        }
    }
}
