"""Minisign signature verification using Ed25519."""

from __future__ import annotations

import base64
import hashlib

from nacl.signing import VerifyKey
from nacl.exceptions import BadSignatureError


# Minisign format constants
_ED25519_PUBLIC_KEY_BYTES = 32
_ED25519_SIGNATURE_BYTES = 64
_KEY_ID_BYTES = 8
_ALGORITHM_ID_BYTES = 2

# Algorithm ID for Ed25519 public key ("Ed" = 0x45, 0x64)
_ED25519_PUBKEY_ALGORITHM_ID = bytes([0x45, 0x64])

# Algorithm ID for Ed25519 signature ("ED" = 0x45, 0x44)
_ED25519_SIG_ALGORITHM_ID = bytes([0x45, 0x44])


class MinisignVerifier:
    """
    Minisign signature verification using Ed25519.

    This class parses and verifies minisign signatures.
    Minisign uses Ed25519 signatures with a specific file format.

    Minisign Format:
    - Public key: Base64-encoded 42 bytes (2-byte algorithm ID "Ed" + 8-byte key ID + 32-byte key)
    - Signature: Base64-encoded 74 bytes (2-byte algorithm ID "ED" + 8-byte key ID + 64-byte signature)

    Example:
        verifier = MinisignVerifier("RWS...")
        if verifier.verify(data, signature_content):
            print("Signature valid")
    """

    def __init__(self, public_key_base64: str) -> None:
        """
        Create a verifier from a minisign public key string.

        Args:
            public_key_base64: Minisign public key in base64 format (e.g., "RWS...").

        Raises:
            ValueError: If the key format is invalid.
        """
        public_key_bytes, key_id = self._parse_public_key(public_key_base64)
        self._key_id = key_id
        self._verify_key = VerifyKey(public_key_bytes)

    @staticmethod
    def _parse_public_key(public_key_base64: str) -> tuple[bytes, bytes]:
        """
        Parse a minisign public key from base64 format.

        Format: 2 bytes algorithm ID + 8 bytes key ID + 32 bytes public key

        Returns:
            Tuple of (public_key_bytes, key_id).

        Raises:
            ValueError: If the key format is invalid.
        """
        try:
            decoded = base64.b64decode(public_key_base64.strip())
        except Exception as e:
            raise ValueError(f"Invalid base64 encoding in public key: {e}") from e

        expected_length = _ALGORITHM_ID_BYTES + _KEY_ID_BYTES + _ED25519_PUBLIC_KEY_BYTES
        if len(decoded) != expected_length:
            raise ValueError(
                f"Invalid public key length: expected {expected_length}, got {len(decoded)}"
            )

        # Verify algorithm ID
        algorithm_id = decoded[:_ALGORITHM_ID_BYTES]
        if algorithm_id != _ED25519_PUBKEY_ALGORITHM_ID:
            raise ValueError(
                f"Invalid algorithm ID: expected Ed25519, got {algorithm_id.hex()}"
            )

        # Extract the 8-byte key ID (right after algorithm ID)
        key_id = decoded[_ALGORITHM_ID_BYTES : _ALGORITHM_ID_BYTES + _KEY_ID_BYTES]

        # Extract the 32-byte Ed25519 public key (after key ID)
        public_key = decoded[_ALGORITHM_ID_BYTES + _KEY_ID_BYTES :]

        return public_key, key_id

    @staticmethod
    def _parse_signature(signature_string: str) -> tuple[bytes, bytes, bool]:
        """
        Parse a minisign signature from the multi-line format.

        Format:
            untrusted comment: <comment>
            <base64-encoded signature>
            trusted comment: <comment>
            <base64-encoded global signature>

        We only use the second line (the signature itself).

        Returns:
            Tuple of (key_id, signature, is_prehashed).

        Raises:
            ValueError: If signature parsing fails.
        """
        lines = signature_string.strip().split("\n")
        if len(lines) < 2:
            raise ValueError("Invalid signature format: expected at least 2 lines")

        # The signature is on the second line
        signature_base64 = lines[1].strip()

        try:
            decoded = base64.b64decode(signature_base64)
        except Exception as e:
            raise ValueError(f"Invalid base64 encoding in signature: {e}") from e

        expected_length = _ALGORITHM_ID_BYTES + _KEY_ID_BYTES + _ED25519_SIGNATURE_BYTES
        if len(decoded) != expected_length:
            raise ValueError(
                f"Invalid signature length: expected {expected_length}, got {len(decoded)}"
            )

        # Check algorithm ID - "ED" = prehashed, "Ed" = legacy non-prehashed
        algorithm_id = decoded[:_ALGORITHM_ID_BYTES]
        if algorithm_id == _ED25519_SIG_ALGORITHM_ID:
            is_prehashed = True  # "ED" - prehashed with BLAKE2b
        elif algorithm_id == _ED25519_PUBKEY_ALGORITHM_ID:
            is_prehashed = False  # "Ed" - legacy non-prehashed
        else:
            raise ValueError(
                f"Invalid algorithm ID in signature: expected Ed25519, got {algorithm_id.hex()}"
            )

        # Extract key ID
        key_id = decoded[_ALGORITHM_ID_BYTES : _ALGORITHM_ID_BYTES + _KEY_ID_BYTES]

        # Extract signature
        signature = decoded[_ALGORITHM_ID_BYTES + _KEY_ID_BYTES :]

        return key_id, signature, is_prehashed

    def verify(self, data: bytes, signature_string: str) -> bool:
        """
        Verify a minisign signature against data.

        Args:
            data: The data that was signed.
            signature_string: The minisign signature (multi-line format).

        Returns:
            True if the signature is valid, False otherwise.

        Raises:
            ValueError: If signature parsing fails.
        """
        sig_key_id, signature, is_prehashed = self._parse_signature(signature_string)

        # Verify key ID matches
        if sig_key_id != self._key_id:
            return False

        # Minisign "ED" signatures are prehashed - compute BLAKE2b-512 hash first
        # This matches SIGALG_PREHASHED in the minisign crate
        if is_prehashed:
            data_to_verify = hashlib.blake2b(data, digest_size=64).digest()
        else:
            data_to_verify = data

        # Verify the signature using Ed25519
        try:
            self._verify_key.verify(data_to_verify, signature)
            return True
        except BadSignatureError:
            return False
