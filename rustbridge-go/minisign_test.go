package rustbridge

import (
	"crypto/ed25519"
	"encoding/base64"
	"fmt"
	"testing"

	"golang.org/x/crypto/blake2b"
)

// buildMinisignPublicKey encodes an Ed25519 public key in minisign format.
func buildMinisignPublicKey(pub ed25519.PublicKey, keyID [8]byte) string {
	buf := make([]byte, 0, 42)
	buf = append(buf, 0x45, 0x64) // "Ed" algorithm ID
	buf = append(buf, keyID[:]...)
	buf = append(buf, pub...)
	return base64.StdEncoding.EncodeToString(buf)
}

// buildMinisignSignature encodes an Ed25519 signature in minisign format (prehashed).
func buildMinisignSignature(sig []byte, keyID [8]byte, prehashed bool) string {
	buf := make([]byte, 0, 74)
	if prehashed {
		buf = append(buf, 0x45, 0x44) // "ED"
	} else {
		buf = append(buf, 0x45, 0x64) // "Ed"
	}
	buf = append(buf, keyID[:]...)
	buf = append(buf, sig...)
	sigB64 := base64.StdEncoding.EncodeToString(buf)
	return fmt.Sprintf("untrusted comment: test\n%s\ntrusted comment: test\n%s",
		sigB64, base64.StdEncoding.EncodeToString(sig[:ed25519SignatureBytes]))
}

func TestNewMinisignVerifier___ValidKey___Succeeds(t *testing.T) {
	pub, _, _ := ed25519.GenerateKey(nil)
	keyID := [8]byte{1, 2, 3, 4, 5, 6, 7, 8}
	encoded := buildMinisignPublicKey(pub, keyID)

	v, err := NewMinisignVerifier(encoded)

	if err != nil {
		t.Fatalf("NewMinisignVerifier error: %v", err)
	}
	if v == nil {
		t.Fatal("verifier is nil")
	}
}

func TestNewMinisignVerifier___InvalidBase64___ReturnsError(t *testing.T) {
	_, err := NewMinisignVerifier("not-valid-base64!!!")

	if err == nil {
		t.Fatal("expected error for invalid base64")
	}
}

func TestNewMinisignVerifier___WrongLength___ReturnsError(t *testing.T) {
	encoded := base64.StdEncoding.EncodeToString([]byte{0x45, 0x64, 1, 2, 3})

	_, err := NewMinisignVerifier(encoded)

	if err == nil {
		t.Fatal("expected error for wrong length")
	}
}

func TestNewMinisignVerifier___WrongAlgorithmID___ReturnsError(t *testing.T) {
	buf := make([]byte, 42)
	buf[0] = 0xFF // wrong algorithm ID
	buf[1] = 0xFF
	encoded := base64.StdEncoding.EncodeToString(buf)

	_, err := NewMinisignVerifier(encoded)

	if err == nil {
		t.Fatal("expected error for wrong algorithm ID")
	}
}

func TestVerify___ValidPrehashedSignature___ReturnsTrue(t *testing.T) {
	pub, priv, _ := ed25519.GenerateKey(nil)
	keyID := [8]byte{1, 2, 3, 4, 5, 6, 7, 8}

	data := []byte("hello rustbridge")
	hash := blake2b.Sum512(data)
	sig := ed25519.Sign(priv, hash[:])

	pubKey := buildMinisignPublicKey(pub, keyID)
	sigStr := buildMinisignSignature(sig, keyID, true)

	v, err := NewMinisignVerifier(pubKey)
	if err != nil {
		t.Fatalf("NewMinisignVerifier error: %v", err)
	}

	valid, err := v.Verify(data, sigStr)

	if err != nil {
		t.Fatalf("Verify error: %v", err)
	}
	if !valid {
		t.Error("Verify returned false, want true")
	}
}

func TestVerify___ValidLegacySignature___ReturnsTrue(t *testing.T) {
	pub, priv, _ := ed25519.GenerateKey(nil)
	keyID := [8]byte{1, 2, 3, 4, 5, 6, 7, 8}

	data := []byte("hello rustbridge")
	sig := ed25519.Sign(priv, data)

	pubKey := buildMinisignPublicKey(pub, keyID)
	sigStr := buildMinisignSignature(sig, keyID, false)

	v, err := NewMinisignVerifier(pubKey)
	if err != nil {
		t.Fatalf("NewMinisignVerifier error: %v", err)
	}

	valid, err := v.Verify(data, sigStr)

	if err != nil {
		t.Fatalf("Verify error: %v", err)
	}
	if !valid {
		t.Error("Verify returned false, want true")
	}
}

func TestVerify___TamperedData___ReturnsFalse(t *testing.T) {
	pub, priv, _ := ed25519.GenerateKey(nil)
	keyID := [8]byte{1, 2, 3, 4, 5, 6, 7, 8}

	data := []byte("hello rustbridge")
	hash := blake2b.Sum512(data)
	sig := ed25519.Sign(priv, hash[:])

	pubKey := buildMinisignPublicKey(pub, keyID)
	sigStr := buildMinisignSignature(sig, keyID, true)

	v, _ := NewMinisignVerifier(pubKey)

	valid, err := v.Verify([]byte("tampered data"), sigStr)

	if err != nil {
		t.Fatalf("Verify error: %v", err)
	}
	if valid {
		t.Error("Verify returned true for tampered data")
	}
}

func TestVerify___WrongKeyID___ReturnsFalse(t *testing.T) {
	pub, priv, _ := ed25519.GenerateKey(nil)
	keyID := [8]byte{1, 2, 3, 4, 5, 6, 7, 8}
	wrongKeyID := [8]byte{9, 9, 9, 9, 9, 9, 9, 9}

	data := []byte("hello rustbridge")
	hash := blake2b.Sum512(data)
	sig := ed25519.Sign(priv, hash[:])

	pubKey := buildMinisignPublicKey(pub, keyID)
	sigStr := buildMinisignSignature(sig, wrongKeyID, true) // different key ID in signature

	v, _ := NewMinisignVerifier(pubKey)

	valid, err := v.Verify(data, sigStr)

	if err != nil {
		t.Fatalf("Verify error: %v", err)
	}
	if valid {
		t.Error("Verify returned true for wrong key ID")
	}
}
