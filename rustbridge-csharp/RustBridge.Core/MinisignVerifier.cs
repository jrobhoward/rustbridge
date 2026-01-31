using System.Security.Cryptography;
using NSec.Cryptography;

namespace RustBridge;

/// <summary>
/// Minisign signature verification using Ed25519.
/// <para>
/// This class parses and verifies minisign signatures.
/// Minisign uses Ed25519 signatures with a specific file format.
/// </para>
/// <para>
/// <b>Minisign Format:</b>
/// <list type="bullet">
/// <item>Public key: Base64-encoded 42 bytes (2-byte algorithm ID "Ed" + 8-byte key ID + 32-byte key)</item>
/// <item>Signature: Base64-encoded 74 bytes (2-byte algorithm ID "ED" + 8-byte key ID + 64-byte signature)</item>
/// </list>
/// </para>
/// </summary>
/// <seealso href="https://jedisct1.github.io/minisign/">Minisign</seealso>
public class MinisignVerifier
{
    private const int Ed25519PublicKeyBytes = 32;
    private const int Ed25519SignatureBytes = 64;
    private const int KeyIdBytes = 8;
    private const int AlgorithmIdBytes = 2;

    // Algorithm ID for Ed25519 public key ("Ed" = 0x45, 0x64)
    private static readonly byte[] Ed25519PubkeyAlgorithmId = { 0x45, 0x64 };

    // Algorithm ID for Ed25519 signature ("ED" = 0x45, 0x44)
    private static readonly byte[] Ed25519SigAlgorithmId = { 0x45, 0x44 };

    private readonly PublicKey _publicKey;
    private readonly byte[] _keyId;

    /// <summary>
    /// Create a verifier from a minisign public key string.
    /// </summary>
    /// <param name="publicKeyBase64">Minisign public key in base64 format (e.g., "RWS...").</param>
    /// <exception cref="ArgumentNullException">If publicKeyBase64 is null.</exception>
    /// <exception cref="ArgumentException">If the key format is invalid.</exception>
    public MinisignVerifier(string publicKeyBase64)
    {
        ArgumentNullException.ThrowIfNull(publicKeyBase64);
        if (string.IsNullOrWhiteSpace(publicKeyBase64))
        {
            throw new ArgumentException("Public key cannot be empty or whitespace", nameof(publicKeyBase64));
        }

        var (publicKeyBytes, keyId) = ParsePublicKey(publicKeyBase64);
        _keyId = keyId;

        // Create NSec public key from raw bytes
        if (!PublicKey.TryImport(SignatureAlgorithm.Ed25519, publicKeyBytes, KeyBlobFormat.RawPublicKey, out var key) || key == null)
        {
            throw new ArgumentException("Failed to import Ed25519 public key");
        }
        _publicKey = key;
    }

    /// <summary>
    /// Parse a minisign public key from base64 format.
    /// Format: 2 bytes algorithm ID + 8 bytes key ID + 32 bytes public key
    /// </summary>
    private static (byte[] publicKey, byte[] keyId) ParsePublicKey(string publicKeyBase64)
    {
        byte[] decoded;
        try
        {
            decoded = Convert.FromBase64String(publicKeyBase64.Trim());
        }
        catch (FormatException ex)
        {
            throw new ArgumentException("Invalid base64 encoding in public key", ex);
        }

        var expectedLength = AlgorithmIdBytes + KeyIdBytes + Ed25519PublicKeyBytes;
        if (decoded.Length != expectedLength)
        {
            throw new ArgumentException(
                $"Invalid public key length: expected {expectedLength}, got {decoded.Length}");
        }

        // Verify algorithm ID
        var algorithmId = decoded.AsSpan(0, AlgorithmIdBytes);
        if (!algorithmId.SequenceEqual(Ed25519PubkeyAlgorithmId))
        {
            throw new ArgumentException(
                $"Invalid algorithm ID: expected Ed25519, got {Convert.ToHexString(algorithmId)}");
        }

        // Extract the 8-byte key ID (right after algorithm ID)
        var keyId = decoded.AsSpan(AlgorithmIdBytes, KeyIdBytes).ToArray();

        // Extract the 32-byte Ed25519 public key (after key ID)
        var publicKey = decoded.AsSpan(AlgorithmIdBytes + KeyIdBytes, Ed25519PublicKeyBytes).ToArray();

        return (publicKey, keyId);
    }

    /// <summary>
    /// Parse a minisign signature from the multi-line format.
    /// <para>
    /// Format:
    /// <code>
    /// untrusted comment: &lt;comment&gt;
    /// &lt;base64-encoded signature&gt;
    /// trusted comment: &lt;comment&gt;
    /// &lt;base64-encoded global signature&gt;
    /// </code>
    /// </para>
    /// We only use the second line (the signature itself).
    /// </summary>
    private static (byte[] keyId, byte[] signature, bool isPrehashed) ParseSignature(string signatureString)
    {
        var lines = signatureString.Split('\n');
        if (lines.Length < 2)
        {
            throw new CryptographicException("Invalid signature format: expected at least 2 lines");
        }

        // The signature is on the second line
        var signatureBase64 = lines[1].Trim();

        byte[] decoded;
        try
        {
            decoded = Convert.FromBase64String(signatureBase64);
        }
        catch (FormatException ex)
        {
            throw new CryptographicException("Invalid base64 encoding in signature", ex);
        }

        var expectedLength = AlgorithmIdBytes + KeyIdBytes + Ed25519SignatureBytes;
        if (decoded.Length != expectedLength)
        {
            throw new CryptographicException(
                $"Invalid signature length: expected {expectedLength}, got {decoded.Length}");
        }

        // Check algorithm ID - "ED" = prehashed, "Ed" = legacy non-prehashed
        var algorithmId = decoded.AsSpan(0, AlgorithmIdBytes);
        bool isPrehashed;
        if (algorithmId.SequenceEqual(Ed25519SigAlgorithmId))
        {
            isPrehashed = true;  // "ED" - prehashed with BLAKE2b
        }
        else if (algorithmId.SequenceEqual(Ed25519PubkeyAlgorithmId))
        {
            isPrehashed = false; // "Ed" - legacy non-prehashed
        }
        else
        {
            throw new CryptographicException(
                $"Invalid algorithm ID in signature: expected Ed25519, got {Convert.ToHexString(algorithmId)}");
        }

        // Extract key ID
        var keyId = decoded.AsSpan(AlgorithmIdBytes, KeyIdBytes).ToArray();

        // Extract signature
        var signature = decoded.AsSpan(AlgorithmIdBytes + KeyIdBytes, Ed25519SignatureBytes).ToArray();

        return (keyId, signature, isPrehashed);
    }

    /// <summary>
    /// Verify a minisign signature against data.
    /// </summary>
    /// <param name="data">The data that was signed.</param>
    /// <param name="signatureString">The minisign signature (multi-line format).</param>
    /// <returns>True if the signature is valid, false otherwise.</returns>
    /// <exception cref="CryptographicException">If signature parsing fails.</exception>
    public bool Verify(byte[] data, string signatureString)
    {
        return Verify((ReadOnlySpan<byte>)data, signatureString);
    }

    /// <summary>
    /// Verify a minisign signature against data.
    /// </summary>
    /// <param name="data">The data that was signed.</param>
    /// <param name="signatureString">The minisign signature (multi-line format).</param>
    /// <returns>True if the signature is valid, false otherwise.</returns>
    /// <exception cref="CryptographicException">If signature parsing fails.</exception>
    public bool Verify(ReadOnlySpan<byte> data, string signatureString)
    {
        var (sigKeyId, signature, isPrehashed) = ParseSignature(signatureString);

        // Verify key ID matches
        if (!sigKeyId.AsSpan().SequenceEqual(_keyId))
        {
            return false;
        }

        // Minisign "ED" signatures are prehashed - compute BLAKE2b-512 hash first
        // This matches SIGALG_PREHASHED in the minisign crate
        if (isPrehashed)
        {
            var hash = NSec.Cryptography.HashAlgorithm.Blake2b_512.Hash(data);
            return SignatureAlgorithm.Ed25519.Verify(_publicKey, hash, signature);
        }

        // Non-prehashed: verify directly without allocation
        return SignatureAlgorithm.Ed25519.Verify(_publicKey, data, signature);
    }
}
