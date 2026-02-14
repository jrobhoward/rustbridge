package rustbridge

import (
	"crypto/ed25519"
	"encoding/base64"
	"errors"
	"strings"

	"golang.org/x/crypto/blake2b"
)

const (
	ed25519PublicKeyBytes = 32
	ed25519SignatureBytes = 64
	keyIDBytes            = 8
	algorithmIDBytes      = 2
)

// Algorithm IDs in minisign format.
var (
	// "Ed" = Ed25519 public key algorithm ID (0x45, 0x64)
	ed25519PubkeyAlgorithmID = [2]byte{0x45, 0x64}

	// "ED" = Ed25519 prehashed signature algorithm ID (0x45, 0x44)
	ed25519SigAlgorithmID = [2]byte{0x45, 0x44}
)

// MinisignVerifier verifies minisign Ed25519 signatures.
type MinisignVerifier struct {
	publicKey ed25519.PublicKey
	keyID     [keyIDBytes]byte
}

// NewMinisignVerifier creates a verifier from a minisign public key string (base64).
func NewMinisignVerifier(publicKeyBase64 string) (*MinisignVerifier, error) {
	decoded, err := base64.StdEncoding.DecodeString(strings.TrimSpace(publicKeyBase64))
	if err != nil {
		return nil, errors.New("invalid base64 encoding in public key: " + err.Error())
	}

	expectedLen := algorithmIDBytes + keyIDBytes + ed25519PublicKeyBytes
	if len(decoded) != expectedLen {
		return nil, errors.New("invalid public key length")
	}

	// Verify algorithm ID
	var algID [2]byte
	copy(algID[:], decoded[:algorithmIDBytes])
	if algID != ed25519PubkeyAlgorithmID {
		return nil, errors.New("invalid algorithm ID: expected Ed25519")
	}

	v := &MinisignVerifier{
		publicKey: ed25519.PublicKey(decoded[algorithmIDBytes+keyIDBytes:]),
	}
	copy(v.keyID[:], decoded[algorithmIDBytes:algorithmIDBytes+keyIDBytes])

	return v, nil
}

// Verify checks a minisign signature against data.
// The signatureString should be the multi-line minisign signature format.
func (v *MinisignVerifier) Verify(data []byte, signatureString string) (bool, error) {
	sigKeyID, signature, isPrehashed, err := parseSignature(signatureString)
	if err != nil {
		return false, err
	}

	// Verify key ID matches
	if sigKeyID != v.keyID {
		return false, nil
	}

	// Determine what data to verify
	var dataToVerify []byte
	if isPrehashed {
		// SIGALG_PREHASHED: compute BLAKE2b-512 hash first
		hash := blake2b.Sum512(data)
		dataToVerify = hash[:]
	} else {
		dataToVerify = data
	}

	// Verify the Ed25519 signature
	return ed25519.Verify(v.publicKey, dataToVerify, signature), nil
}

// parseSignature extracts the key ID, signature bytes, and prehash flag from a minisign signature.
//
// Format:
//
//	untrusted comment: <comment>
//	<base64-encoded signature>
//	trusted comment: <comment>
//	<base64-encoded global signature>
func parseSignature(signatureString string) (keyID [keyIDBytes]byte, sig []byte, isPrehashed bool, err error) {
	lines := strings.Split(strings.TrimSpace(signatureString), "\n")
	if len(lines) < 2 {
		return keyID, nil, false, errors.New("invalid signature format: expected at least 2 lines")
	}

	decoded, err := base64.StdEncoding.DecodeString(strings.TrimSpace(lines[1]))
	if err != nil {
		return keyID, nil, false, errors.New("invalid base64 encoding in signature: " + err.Error())
	}

	expectedLen := algorithmIDBytes + keyIDBytes + ed25519SignatureBytes
	if len(decoded) != expectedLen {
		return keyID, nil, false, errors.New("invalid signature length")
	}

	// Check algorithm ID
	var algID [2]byte
	copy(algID[:], decoded[:algorithmIDBytes])
	switch algID {
	case ed25519SigAlgorithmID:
		isPrehashed = true // "ED" - prehashed with BLAKE2b
	case ed25519PubkeyAlgorithmID:
		isPrehashed = false // "Ed" - legacy non-prehashed
	default:
		return keyID, nil, false, errors.New("invalid algorithm ID in signature")
	}

	copy(keyID[:], decoded[algorithmIDBytes:algorithmIDBytes+keyIDBytes])
	sig = decoded[algorithmIDBytes+keyIDBytes:]

	return keyID, sig, isPrehashed, nil
}
