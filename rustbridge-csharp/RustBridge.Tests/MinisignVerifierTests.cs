using System.Security.Cryptography;
using System.Text;

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
    // ========================================================================
    // Oracle Test Vectors (generated with minisign crate 0.8)
    // ========================================================================
    // These vectors verify that our MinisignVerifier implementation produces
    // identical results to the reference Rust minisign implementation.

    private static readonly byte[] OracleTestData = Encoding.UTF8.GetBytes("Hello, rustbridge!");

    private const string OraclePublicKey = "RWRX0dXiesomR/SPv8ukThZBrY9f8LrZd0XFz/H5E9jtSR0G9/sRXPu0";

    private const string OracleSignature = """
        untrusted comment: untrusted comment for test
        RURX0dXiesomR1yQGGyQgLLAGcsXIj/T/IxgxPjXBuCQ9MD/DjQtm2vxXmuM2OEvRAn36pPO92uTCBiL+na0idTmJIkn9Fnd6g4=
        trusted comment: trusted comment for test
        oqvjCoVOeFtpPv1tQ33i2+BZqHndTlsPLU+/njVMuJw6fjQs+o9O8/MSgMkvG3DqxZVFqEeQYkfuFn3h96rIDQ==
        """;

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
        validKey[1] = 0x64; // 'd' - minisign public keys use "Ed", not "ED"
        // Bytes 2-9: Key ID (8 bytes) - use zeros for this test
        // Bytes 10-41: Ed25519 public key (32 bytes) - use zeros for this test
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
    /// - 2 bytes: algorithm ID (0x45 0x64 = "Ed" for Ed25519)
    /// - 8 bytes: key ID
    /// - 32 bytes: Ed25519 public key
    /// </summary>
    [Fact]
    public void KeyFormat___Documentation()
    {
        const int algorithmIdBytes = 2;
        const int keyIdBytes = 8;
        const int publicKeyBytes = 32;
        const int totalExpectedBytes = algorithmIdBytes + keyIdBytes + publicKeyBytes;

        Assert.Equal(42, totalExpectedBytes);
    }

    /// <summary>
    /// This test documents the expected signature format.
    /// A valid minisign signature line (base64-decoded) is 74 bytes:
    /// - 2 bytes: algorithm ID (0x45 0x44 = "ED" for prehashed, or 0x45 0x64 = "Ed" for legacy)
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

        // Build minisign format: 2 bytes algo ID + 8 bytes key ID + 32 bytes public key
        var minisignKey = new byte[42];
        minisignKey[0] = 0x45; // 'E'
        minisignKey[1] = 0x64; // 'd' - minisign public keys use "Ed", not "ED"
        // Bytes 2-9: Key ID (left as zeros)
        Array.Copy(publicKeyBytes, 0, minisignKey, 10, 32);

        return Convert.ToBase64String(minisignKey);
    }

    #region Oracle Vector Tests

    /// <summary>
    /// Verify a known-good signature from the Rust minisign crate.
    /// </summary>
    [Fact]
    public void Verify___OracleValidSignature___ReturnsTrue()
    {
        var verifier = new MinisignVerifier(OraclePublicKey);

        var result = verifier.Verify(OracleTestData, OracleSignature);

        Assert.True(result, "Valid oracle signature should verify");
    }

    /// <summary>
    /// Verify that modifying the data causes verification to fail.
    /// </summary>
    [Fact]
    public void Verify___OracleWrongData___ReturnsFalse()
    {
        var verifier = new MinisignVerifier(OraclePublicKey);
        var wrongData = Encoding.UTF8.GetBytes("Hello, rustbridge?"); // Changed ! to ?

        var result = verifier.Verify(wrongData, OracleSignature);

        Assert.False(result, "Modified data should fail verification");
    }

    /// <summary>
    /// Verify that modifying the signature causes verification to fail.
    /// </summary>
    [Fact]
    public void Verify___OracleTamperedSignature___ReturnsFalse()
    {
        var verifier = new MinisignVerifier(OraclePublicKey);
        // Tamper with one character in the signature
        var tamperedSig = OracleSignature.Replace(
            "RURX0dXiesomR1yQGGyQgLLAGcsXIj",
            "RURX0dXiesomR1yQGGyQgLLAGcsXIk" // Changed last char
        );

        var result = verifier.Verify(OracleTestData, tamperedSig);

        Assert.False(result, "Tampered signature should fail verification");
    }

    #endregion
}
