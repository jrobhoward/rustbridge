using System.Security.Cryptography;

namespace RustBridge.Tests;

/// <summary>
/// Tests for <see cref="MinisignVerifier"/>.
/// <para>
/// Note: Full signature verification tests require real minisign key pairs.
/// These tests verify the parsing and error handling logic.
/// </para>
/// </summary>
public class MinisignVerifierTests
{
    [Fact]
    public void Constructor___InvalidBase64___ThrowsArgumentException()
    {
        Assert.Throws<ArgumentException>(() => new MinisignVerifier("not valid base64!!!"));
    }

    [Fact]
    public void Constructor___EmptyString___ThrowsArgumentException()
    {
        Assert.Throws<ArgumentException>(() => new MinisignVerifier(""));
    }

    [Fact]
    public void Constructor___WrongLengthTooShort___ThrowsArgumentException()
    {
        var shortKey = new byte[10];
        var shortKeyBase64 = Convert.ToBase64String(shortKey);

        var exception = Assert.Throws<ArgumentException>(() => new MinisignVerifier(shortKeyBase64));

        Assert.Contains("length", exception.Message, StringComparison.OrdinalIgnoreCase);
    }

    [Fact]
    public void Constructor___WrongLengthTooLong___ThrowsArgumentException()
    {
        var longKey = new byte[50];
        var longKeyBase64 = Convert.ToBase64String(longKey);

        var exception = Assert.Throws<ArgumentException>(() => new MinisignVerifier(longKeyBase64));

        Assert.Contains("length", exception.Message, StringComparison.OrdinalIgnoreCase);
    }

    [Fact]
    public void Constructor___WrongAlgorithmId___ThrowsArgumentException()
    {
        var wrongAlgo = new byte[42];
        wrongAlgo[0] = 0x00;
        wrongAlgo[1] = 0x00;
        var wrongAlgoBase64 = Convert.ToBase64String(wrongAlgo);

        Assert.Throws<ArgumentException>(() => new MinisignVerifier(wrongAlgoBase64));
    }

    [Fact]
    public void Constructor___NullInput___ThrowsException()
    {
        // Currently throws NullReferenceException - could be improved to ArgumentNullException
        Assert.ThrowsAny<Exception>(() => new MinisignVerifier(null!));
    }

    [Fact]
    public void Constructor___ValidKeyFormat___Succeeds()
    {
        var validKey = new byte[42];
        validKey[0] = 0x45; // 'E'
        validKey[1] = 0x44; // 'D'
        // Bytes 2-33: Ed25519 public key (32 bytes) - use zeros for this test
        // Bytes 34-41: Key ID (8 bytes) - use zeros for this test
        var validKeyBase64 = Convert.ToBase64String(validKey);

        // This may fail on the Ed25519 import since zeros aren't a valid public key,
        // but it should get past the format validation
        var exception = Record.Exception(() => new MinisignVerifier(validKeyBase64));

        // If it throws, it should be about the key import, not the format
        if (exception != null)
        {
            Assert.Contains("import", exception.Message, StringComparison.OrdinalIgnoreCase);
        }
    }

    [Fact]
    public void Verify___InvalidSignatureFormat___ThrowsCryptographicException()
    {
        var validKey = CreateValidTestKey();
        var verifier = new MinisignVerifier(validKey);
        var data = "test data"u8.ToArray();

        Assert.Throws<CryptographicException>(() => verifier.Verify(data, "invalid signature"));
    }

    [Fact]
    public void Verify___SignatureTooFewLines___ThrowsCryptographicException()
    {
        var validKey = CreateValidTestKey();
        var verifier = new MinisignVerifier(validKey);
        var data = "test data"u8.ToArray();

        var exception = Assert.Throws<CryptographicException>(() =>
            verifier.Verify(data, "single line only"));

        Assert.Contains("2 lines", exception.Message);
    }

    [Fact]
    public void Verify___SignatureWrongBase64___ThrowsCryptographicException()
    {
        var validKey = CreateValidTestKey();
        var verifier = new MinisignVerifier(validKey);
        var data = "test data"u8.ToArray();
        var signature = "untrusted comment: test\nnot-valid-base64!!!";

        Assert.Throws<CryptographicException>(() => verifier.Verify(data, signature));
    }

    [Fact]
    public void Verify___SignatureWrongLength___ThrowsCryptographicException()
    {
        var validKey = CreateValidTestKey();
        var verifier = new MinisignVerifier(validKey);
        var data = "test data"u8.ToArray();
        var shortSig = Convert.ToBase64String(new byte[10]);
        var signature = $"untrusted comment: test\n{shortSig}";

        var exception = Assert.Throws<CryptographicException>(() =>
            verifier.Verify(data, signature));

        Assert.Contains("length", exception.Message, StringComparison.OrdinalIgnoreCase);
    }

    [Fact]
    public void Verify___SignatureWrongAlgorithmId___ThrowsCryptographicException()
    {
        var validKey = CreateValidTestKey();
        var verifier = new MinisignVerifier(validKey);
        var data = "test data"u8.ToArray();

        var wrongAlgoSig = new byte[74];
        wrongAlgoSig[0] = 0x00;
        wrongAlgoSig[1] = 0x00;
        var signature = $"untrusted comment: test\n{Convert.ToBase64String(wrongAlgoSig)}";

        var exception = Assert.Throws<CryptographicException>(() =>
            verifier.Verify(data, signature));

        Assert.Contains("algorithm", exception.Message, StringComparison.OrdinalIgnoreCase);
    }

    [Fact]
    public void Verify___KeyIdMismatch___ReturnsFalse()
    {
        var validKey = CreateValidTestKey();
        var verifier = new MinisignVerifier(validKey);
        var data = "test data"u8.ToArray();

        // Create a signature with correct format but different key ID
        var sigBytes = new byte[74];
        sigBytes[0] = 0x45; // 'E'
        sigBytes[1] = 0x44; // 'D'
        // Bytes 2-9: Key ID (different from the key we created)
        sigBytes[2] = 0xFF;
        sigBytes[3] = 0xFF;
        // Rest is signature (64 bytes)
        var signature = $"untrusted comment: test\n{Convert.ToBase64String(sigBytes)}";

        var result = verifier.Verify(data, signature);

        Assert.False(result);
    }

    /// <summary>
    /// This test documents the expected key format.
    /// A valid minisign public key is 42 bytes:
    /// - 2 bytes: algorithm ID (0x45 0x44 = "ED" for Ed25519)
    /// - 32 bytes: Ed25519 public key
    /// - 8 bytes: key ID
    /// </summary>
    [Fact]
    public void KeyFormat___Documentation()
    {
        const int algorithmIdBytes = 2;
        const int publicKeyBytes = 32;
        const int keyIdBytes = 8;
        const int totalExpectedBytes = algorithmIdBytes + publicKeyBytes + keyIdBytes;

        Assert.Equal(42, totalExpectedBytes);
    }

    /// <summary>
    /// This test documents the expected signature format.
    /// A valid minisign signature line (base64-decoded) is 74 bytes:
    /// - 2 bytes: algorithm ID (0x45 0x44 = "ED" for Ed25519)
    /// - 8 bytes: key ID
    /// - 64 bytes: Ed25519 signature
    /// </summary>
    [Fact]
    public void SignatureFormat___Documentation()
    {
        const int algorithmIdBytes = 2;
        const int keyIdBytes = 8;
        const int signatureBytes = 64;
        const int totalExpectedBytes = algorithmIdBytes + keyIdBytes + signatureBytes;

        Assert.Equal(74, totalExpectedBytes);
    }

    /// <summary>
    /// Creates a valid test key using a real Ed25519 key pair.
    /// </summary>
    private static string CreateValidTestKey()
    {
        // Use NSec to generate a real Ed25519 key pair
        using var key = NSec.Cryptography.Key.Create(
            NSec.Cryptography.SignatureAlgorithm.Ed25519,
            new NSec.Cryptography.KeyCreationParameters { ExportPolicy = NSec.Cryptography.KeyExportPolicies.AllowPlaintextExport });

        var publicKeyBytes = key.PublicKey.Export(NSec.Cryptography.KeyBlobFormat.RawPublicKey);

        // Build minisign format: 2 bytes algo ID + 32 bytes public key + 8 bytes key ID
        var minisignKey = new byte[42];
        minisignKey[0] = 0x45; // 'E'
        minisignKey[1] = 0x44; // 'D'
        Array.Copy(publicKeyBytes, 0, minisignKey, 2, 32);
        // Key ID bytes 34-41 are left as zeros

        return Convert.ToBase64String(minisignKey);
    }
}
