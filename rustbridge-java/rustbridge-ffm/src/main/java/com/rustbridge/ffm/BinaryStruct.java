package com.rustbridge.ffm;

import java.lang.foreign.*;
import java.nio.charset.StandardCharsets;

/**
 * Base class for binary struct wrappers in FFM.
 * <p>
 * Provides utilities for accessing fixed-size strings and common struct operations.
 * Subclasses define specific struct layouts and accessor methods.
 * <p>
 * Example usage:
 * <pre>{@code
 * public class SmallRequestRaw extends BinaryStruct {
 *     public static final StructLayout LAYOUT = MemoryLayout.structLayout(
 *         ValueLayout.JAVA_BYTE.withName("version"),
 *         MemoryLayout.sequenceLayout(3, ValueLayout.JAVA_BYTE).withName("_reserved"),
 *         MemoryLayout.sequenceLayout(64, ValueLayout.JAVA_BYTE).withName("key"),
 *         ValueLayout.JAVA_INT.withName("key_len"),
 *         ValueLayout.JAVA_INT.withName("flags")
 *     );
 *
 *     public SmallRequestRaw(Arena arena) {
 *         super(arena.allocate(LAYOUT));
 *     }
 *
 *     public void setKey(String key) {
 *         setFixedString(key, 4, 64, 68);  // offset=4, maxLen=64, lenOffset=68
 *     }
 * }
 * }</pre>
 */
public abstract class BinaryStruct {
    protected final MemorySegment segment;

    /**
     * Create a BinaryStruct wrapping an existing memory segment.
     *
     * @param segment the memory segment to wrap
     */
    protected BinaryStruct(MemorySegment segment) {
        this.segment = segment;
    }

    /**
     * Get the underlying memory segment.
     *
     * @return the memory segment
     */
    public MemorySegment segment() {
        return segment;
    }

    /**
     * Get the size of this struct in bytes.
     *
     * @return the byte size
     */
    public abstract long byteSize();

    /**
     * Read a fixed-size string field with a separate length field.
     * <p>
     * Common pattern for FFI structs with fixed-size char arrays:
     * <pre>
     * struct MyStruct {
     *     char key[64];    // Fixed buffer
     *     uint32_t key_len; // Actual length
     * }
     * </pre>
     *
     * @param dataOffset byte offset of the string buffer
     * @param maxLen     maximum length of the buffer
     * @param lenOffset  byte offset of the length field (u32)
     * @return the string value
     */
    protected String getFixedString(long dataOffset, int maxLen, long lenOffset) {
        int len = segment.get(ValueLayout.JAVA_INT, lenOffset);
        if (len <= 0) {
            return "";
        }
        len = Math.min(len, maxLen);
        byte[] bytes = new byte[len];
        MemorySegment.copy(segment, ValueLayout.JAVA_BYTE, dataOffset, bytes, 0, len);
        return new String(bytes, StandardCharsets.UTF_8);
    }

    /**
     * Write a fixed-size string field with a separate length field.
     *
     * @param value      the string value to write
     * @param dataOffset byte offset of the string buffer
     * @param maxLen     maximum length of the buffer
     * @param lenOffset  byte offset of the length field (u32)
     */
    protected void setFixedString(String value, long dataOffset, int maxLen, long lenOffset) {
        byte[] bytes = value.getBytes(StandardCharsets.UTF_8);
        int len = Math.min(bytes.length, maxLen);

        // Zero the buffer first
        for (int i = 0; i < maxLen; i++) {
            segment.set(ValueLayout.JAVA_BYTE, dataOffset + i, (byte) 0);
        }

        // Copy string data
        MemorySegment.copy(bytes, 0, segment, ValueLayout.JAVA_BYTE, dataOffset, len);

        // Set length field
        segment.set(ValueLayout.JAVA_INT, lenOffset, len);
    }

    /**
     * Get a byte field.
     *
     * @param offset byte offset
     * @return the byte value
     */
    protected byte getByte(long offset) {
        return segment.get(ValueLayout.JAVA_BYTE, offset);
    }

    /**
     * Set a byte field.
     *
     * @param offset byte offset
     * @param value  the byte value
     */
    protected void setByte(long offset, byte value) {
        segment.set(ValueLayout.JAVA_BYTE, offset, value);
    }

    /**
     * Get an int (u32/i32) field.
     *
     * @param offset byte offset
     * @return the int value
     */
    protected int getInt(long offset) {
        return segment.get(ValueLayout.JAVA_INT, offset);
    }

    /**
     * Set an int (u32/i32) field.
     *
     * @param offset byte offset
     * @param value  the int value
     */
    protected void setInt(long offset, int value) {
        segment.set(ValueLayout.JAVA_INT, offset, value);
    }

    /**
     * Get a long (u64/i64) field.
     *
     * @param offset byte offset
     * @return the long value
     */
    protected long getLong(long offset) {
        return segment.get(ValueLayout.JAVA_LONG, offset);
    }

    /**
     * Set a long (u64/i64) field.
     *
     * @param offset byte offset
     * @param value  the long value
     */
    protected void setLong(long offset, long value) {
        segment.set(ValueLayout.JAVA_LONG, offset, value);
    }
}
