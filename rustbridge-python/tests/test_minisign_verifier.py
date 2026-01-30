"""Tests for MinisignVerifier."""

import base64
import pytest

from rustbridge import MinisignVerifier


def _make_test_public_key(key_id: bytes = b"\x00" * 8) -> str:
    """Create a valid test public key with given key ID.

    Minisign public key format (42 bytes):
    - 2 bytes: algorithm ID "Ed" (0x45, 0x64)
    - 8 bytes: key ID
    - 32 bytes: Ed25519 public key
    """
    # Using all zeros for the public key (invalid for real signing but valid format)
    key_bytes = b"Ed" + key_id + b"\x00" * 32
    return base64.b64encode(key_bytes).decode()


# Test public key (base64 encoded)
# Format: 2 bytes algo ID (Ed) + 8 bytes key ID + 32 bytes public key
TEST_PUBLIC_KEY = _make_test_public_key(b"\x01\x02\x03\x04\x05\x06\x07\x08")

# Test data and signature
TEST_DATA = b"Hello, World!"


# ============================================================================
# Oracle Test Vectors (generated with minisign crate 0.8)
# ============================================================================
# These vectors verify that our MinisignVerifier implementation produces
# identical results to the reference Rust minisign implementation.

ORACLE_TEST_DATA = b"Hello, rustbridge!"

ORACLE_PUBLIC_KEY = "RWRX0dXiesomR/SPv8ukThZBrY9f8LrZd0XFz/H5E9jtSR0G9/sRXPu0"

ORACLE_SIGNATURE = """untrusted comment: untrusted comment for test
RURX0dXiesomR1yQGGyQgLLAGcsXIj/T/IxgxPjXBuCQ9MD/DjQtm2vxXmuM2OEvRAn36pPO92uTCBiL+na0idTmJIkn9Fnd6g4=
trusted comment: trusted comment for test
oqvjCoVOeFtpPv1tQ33i2+BZqHndTlsPLU+/njVMuJw6fjQs+o9O8/MSgMkvG3DqxZVFqEeQYkfuFn3h96rIDQ=="""


class TestMinisignVerifier:
    """Tests for MinisignVerifier."""

    def test_init___invalid_base64___raises_value_error(self) -> None:
        with pytest.raises(ValueError, match="Invalid base64"):
            MinisignVerifier("not-valid-base64!!!")

    def test_init___wrong_length___raises_value_error(self) -> None:
        # Too short
        short_key = base64.b64encode(b"short").decode()

        with pytest.raises(ValueError, match="Invalid public key length"):
            MinisignVerifier(short_key)

    def test_init___wrong_algorithm_id___raises_value_error(self) -> None:
        # Create a key with wrong algorithm ID
        # Wrong algo (XX) + 32 bytes key + 8 bytes key ID = 42 bytes
        wrong_algo = b"XX" + b"\x00" * 40
        encoded = base64.b64encode(wrong_algo).decode()

        with pytest.raises(ValueError, match="Invalid algorithm ID"):
            MinisignVerifier(encoded)

    def test_init___valid_key___creates_verifier(self) -> None:
        # Create a valid key format: Ed + 32 bytes + 8 bytes
        # Note: minisign uses "Ed" (0x45, 0x64) as the algorithm ID
        valid_key = b"Ed" + b"\x00" * 32 + b"\x00" * 8
        encoded = base64.b64encode(valid_key).decode()

        verifier = MinisignVerifier(encoded)

        assert verifier is not None

    def test_parse_signature___too_few_lines___raises_value_error(self) -> None:
        verifier = MinisignVerifier(TEST_PUBLIC_KEY)

        with pytest.raises(ValueError, match="expected at least 2 lines"):
            verifier.verify(b"data", "single line")

    def test_parse_signature___invalid_base64___raises_value_error(self) -> None:
        verifier = MinisignVerifier(TEST_PUBLIC_KEY)
        invalid_sig = "untrusted comment: test\nnot-valid-base64!!!"

        with pytest.raises(ValueError, match="Invalid base64"):
            verifier.verify(b"data", invalid_sig)

    def test_parse_signature___wrong_length___raises_value_error(self) -> None:
        verifier = MinisignVerifier(TEST_PUBLIC_KEY)
        # Signature should be 74 bytes, this is too short
        short_sig = base64.b64encode(b"short").decode()
        invalid_sig = f"untrusted comment: test\n{short_sig}"

        with pytest.raises(ValueError, match="Invalid signature length"):
            verifier.verify(b"data", invalid_sig)

    def test_parse_signature___wrong_algorithm_id___raises_value_error(self) -> None:
        verifier = MinisignVerifier(TEST_PUBLIC_KEY)
        # Wrong algo (XX) + 8 bytes key ID + 64 bytes sig = 74 bytes
        wrong_algo = b"XX" + b"\x00" * 72
        encoded_sig = base64.b64encode(wrong_algo).decode()
        invalid_sig = f"untrusted comment: test\n{encoded_sig}"

        with pytest.raises(ValueError, match="Invalid algorithm ID in signature"):
            verifier.verify(b"data", invalid_sig)

    def test_verify___key_id_mismatch___returns_false(self) -> None:
        verifier = MinisignVerifier(TEST_PUBLIC_KEY)

        # Create a valid-format signature with different key ID
        # ED (2) + different key ID (8) + signature (64) = 74 bytes
        # Note: "ED" (0x45, 0x44) indicates prehashed signature
        different_key_id = b"ED" + b"\xff" * 8 + b"\x00" * 64
        encoded_sig = base64.b64encode(different_key_id).decode()
        sig = f"untrusted comment: test\n{encoded_sig}"

        result = verifier.verify(b"data", sig)

        assert result is False

    def test_verify___invalid_signature___returns_false(self) -> None:
        verifier = MinisignVerifier(TEST_PUBLIC_KEY)

        # Create a signature with matching key ID but invalid signature
        # Extract key ID from the public key (at offset 2, length 8)
        decoded_key = base64.b64decode(TEST_PUBLIC_KEY)
        key_id = decoded_key[2 : 2 + 8]

        # ED (2) + matching key ID (8) + invalid signature (64) = 74 bytes
        # Note: "ED" (0x45, 0x44) indicates prehashed signature
        invalid_sig_bytes = b"ED" + key_id + b"\x00" * 64
        encoded_sig = base64.b64encode(invalid_sig_bytes).decode()
        sig = f"untrusted comment: test\n{encoded_sig}"

        result = verifier.verify(b"data", sig)

        assert result is False


class TestMinisignVerifierOracleVectors:
    """Oracle-based tests using reference vectors from Rust minisign crate.

    These tests verify that our Python MinisignVerifier produces identical
    results to the reference Rust implementation.
    """

    def test_verify___oracle_valid_signature___returns_true(self) -> None:
        """Verify a known-good signature from the Rust minisign crate."""
        verifier = MinisignVerifier(ORACLE_PUBLIC_KEY)

        result = verifier.verify(ORACLE_TEST_DATA, ORACLE_SIGNATURE)

        assert result is True

    def test_verify___oracle_wrong_data___returns_false(self) -> None:
        """Verify that modifying the data causes verification to fail."""
        verifier = MinisignVerifier(ORACLE_PUBLIC_KEY)
        wrong_data = b"Hello, rustbridge?"  # Changed ! to ?

        result = verifier.verify(wrong_data, ORACLE_SIGNATURE)

        assert result is False

    def test_verify___oracle_tampered_signature___returns_false(self) -> None:
        """Verify that modifying the signature causes verification to fail."""
        verifier = MinisignVerifier(ORACLE_PUBLIC_KEY)

        # Tamper with one character in the signature
        tampered_sig = ORACLE_SIGNATURE.replace(
            "RURX0dXiesomR1yQGGyQgLLAGcsXIj",
            "RURX0dXiesomR1yQGGyQgLLAGcsXIk",  # Changed last char
        )

        result = verifier.verify(ORACLE_TEST_DATA, tampered_sig)

        assert result is False
