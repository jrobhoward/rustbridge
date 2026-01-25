namespace RustBridge;

/// <summary>
/// Interface for binary struct types used with raw FFI transport.
/// <para>
/// Binary structs provide high-performance communication by avoiding JSON serialization.
/// They must have a fixed memory layout matching the Rust <c>#[repr(C)]</c> struct.
/// </para>
/// <example>
/// <code>
/// [StructLayout(LayoutKind.Sequential, Pack = 1)]
/// public struct SmallRequestRaw : IBinaryStruct
/// {
///     public byte Version;
///     private fixed byte _reserved[3];
///     public fixed byte Key[64];
///     public uint KeyLen;
///     public uint Flags;
///
///     public int ByteSize => 76;
/// }
/// </code>
/// </example>
/// </summary>
public interface IBinaryStruct
{
    /// <summary>
    /// Get the size of this struct in bytes.
    /// </summary>
    int ByteSize { get; }
}
