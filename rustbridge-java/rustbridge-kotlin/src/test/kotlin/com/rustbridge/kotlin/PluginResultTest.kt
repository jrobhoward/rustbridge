package com.rustbridge.kotlin

import com.rustbridge.PluginException
import org.junit.jupiter.api.*
import org.junit.jupiter.api.Assertions.*
import java.util.concurrent.TimeUnit

/**
 * Tests for PluginResult sealed class.
 */
@TestMethodOrder(MethodOrderer.OrderAnnotation::class)
@Timeout(value = 30, unit = TimeUnit.SECONDS)
class PluginResultTest {

    // ==================== Success tests ====================

    @Test
    @Order(1)
    @DisplayName("success___isSuccess___returns_true")
    fun success___isSuccess___returns_true() {
        val result: PluginResult<String> = PluginResult.Success("test")

        assertTrue(result.isSuccess)
        assertFalse(result.isError)
    }

    @Test
    @Order(2)
    @DisplayName("success___getOrNull___returns_value")
    fun success___getOrNull___returns_value() {
        val result: PluginResult<String> = PluginResult.Success("test value")

        assertEquals("test value", result.getOrNull())
    }

    @Test
    @Order(3)
    @DisplayName("success___getOrThrow___returns_value")
    fun success___getOrThrow___returns_value() {
        val result: PluginResult<Int> = PluginResult.Success(42)

        assertEquals(42, result.getOrThrow())
    }

    @Test
    @Order(4)
    @DisplayName("success___getOrElse___returns_value")
    fun success___getOrElse___returns_value() {
        val result: PluginResult<String> = PluginResult.Success("original")

        val value = result.getOrElse { "fallback" }

        assertEquals("original", value)
    }

    @Test
    @Order(5)
    @DisplayName("success___getOrDefault___returns_value")
    fun success___getOrDefault___returns_value() {
        val result: PluginResult<String> = PluginResult.Success("original")

        assertEquals("original", result.getOrDefault("default"))
    }

    // ==================== Error tests ====================

    @Test
    @Order(10)
    @DisplayName("error___isError___returns_true")
    fun error___isError___returns_true() {
        val result: PluginResult<String> = PluginResult.Error(1, "test error")

        assertTrue(result.isError)
        assertFalse(result.isSuccess)
    }

    @Test
    @Order(11)
    @DisplayName("error___getOrNull___returns_null")
    fun error___getOrNull___returns_null() {
        val result: PluginResult<String> = PluginResult.Error(1, "test error")

        assertNull(result.getOrNull())
    }

    @Test
    @Order(12)
    @DisplayName("error___getOrThrow___throws_exception")
    fun error___getOrThrow___throws_exception() {
        val result: PluginResult<String> = PluginResult.Error(42, "test error")

        val exception = assertThrows<PluginException> { result.getOrThrow() }
        assertEquals(42, exception.errorCode)
        assertEquals("test error", exception.message)
    }

    @Test
    @Order(13)
    @DisplayName("error___getOrElse___returns_fallback")
    fun error___getOrElse___returns_fallback() {
        val result: PluginResult<String> = PluginResult.Error(1, "error")

        val value = result.getOrElse { "fallback for ${it.code}" }

        assertEquals("fallback for 1", value)
    }

    @Test
    @Order(14)
    @DisplayName("error___getOrDefault___returns_default")
    fun error___getOrDefault___returns_default() {
        val result: PluginResult<String> = PluginResult.Error(1, "error")

        assertEquals("default", result.getOrDefault("default"))
    }

    // ==================== map() tests ====================

    @Test
    @Order(20)
    @DisplayName("success___map___transforms_value")
    fun success___map___transforms_value() {
        val result: PluginResult<Int> = PluginResult.Success(10)

        val mapped = result.map { it * 2 }

        assertEquals(20, mapped.getOrNull())
    }

    @Test
    @Order(21)
    @DisplayName("error___map___preserves_error")
    fun error___map___preserves_error() {
        val result: PluginResult<Int> = PluginResult.Error(1, "error")

        val mapped = result.map { it * 2 }

        assertTrue(mapped.isError)
        assertEquals(1, (mapped as PluginResult.Error).code)
    }

    // ==================== flatMap() tests ====================

    @Test
    @Order(30)
    @DisplayName("success___flatMap___chains_operations")
    fun success___flatMap___chains_operations() {
        val result: PluginResult<Int> = PluginResult.Success(10)

        val chained = result.flatMap { value ->
            if (value > 0) PluginResult.Success(value.toString())
            else PluginResult.Error(1, "negative")
        }

        assertEquals("10", chained.getOrNull())
    }

    @Test
    @Order(31)
    @DisplayName("error___flatMap___preserves_error")
    fun error___flatMap___preserves_error() {
        val result: PluginResult<Int> = PluginResult.Error(1, "original error")

        val chained = result.flatMap { PluginResult.Success(it.toString()) }

        assertTrue(chained.isError)
    }

    // ==================== recover() tests ====================

    @Test
    @Order(40)
    @DisplayName("success___recover___returns_original")
    fun success___recover___returns_original() {
        val result: PluginResult<String> = PluginResult.Success("original")

        val recovered = result.recover { "recovered" }

        assertEquals("original", recovered.getOrNull())
    }

    @Test
    @Order(41)
    @DisplayName("error___recover___returns_recovered_value")
    fun error___recover___returns_recovered_value() {
        val result: PluginResult<String> = PluginResult.Error(1, "error")

        val recovered = result.recover { "recovered from ${it.code}" }

        assertEquals("recovered from 1", recovered.getOrNull())
    }

    // ==================== onSuccess/onError tests ====================

    @Test
    @Order(50)
    @DisplayName("success___onSuccess___executes_action")
    fun success___onSuccess___executes_action() {
        val result: PluginResult<String> = PluginResult.Success("test")
        var executed = false

        result.onSuccess { executed = true }

        assertTrue(executed)
    }

    @Test
    @Order(51)
    @DisplayName("error___onSuccess___does_not_execute")
    fun error___onSuccess___does_not_execute() {
        val result: PluginResult<String> = PluginResult.Error(1, "error")
        var executed = false

        result.onSuccess { executed = true }

        assertFalse(executed)
    }

    @Test
    @Order(52)
    @DisplayName("error___onError___executes_action")
    fun error___onError___executes_action() {
        val result: PluginResult<String> = PluginResult.Error(1, "error")
        var errorCode = 0

        result.onError { errorCode = it.code }

        assertEquals(1, errorCode)
    }

    @Test
    @Order(53)
    @DisplayName("success___onError___does_not_execute")
    fun success___onError___does_not_execute() {
        val result: PluginResult<String> = PluginResult.Success("test")
        var executed = false

        result.onError { executed = true }

        assertFalse(executed)
    }

    // ==================== Companion functions ====================

    @Test
    @Order(60)
    @DisplayName("catching___success___returns_success")
    fun catching___success___returns_success() {
        val result = PluginResult.catching { "computed value" }

        assertTrue(result.isSuccess)
        assertEquals("computed value", result.getOrNull())
    }

    @Test
    @Order(61)
    @DisplayName("catching___plugin_exception___returns_error_with_code")
    fun catching___plugin_exception___returns_error_with_code() {
        val result = PluginResult.catching {
            throw PluginException(42, "plugin error")
        }

        assertTrue(result.isError)
        val error = result as PluginResult.Error
        assertEquals(42, error.code)
        assertEquals("plugin error", error.message)
    }

    @Test
    @Order(62)
    @DisplayName("catching___generic_exception___returns_error_with_zero_code")
    fun catching___generic_exception___returns_error_with_zero_code() {
        val result = PluginResult.catching {
            throw RuntimeException("runtime error")
        }

        assertTrue(result.isError)
        val error = result as PluginResult.Error
        assertEquals(0, error.code)
        assertEquals("runtime error", error.message)
    }

    // ==================== toResult() tests ====================

    @Test
    @Order(70)
    @DisplayName("success___toResult___returns_kotlin_success")
    fun success___toResult___returns_kotlin_success() {
        val pluginResult: PluginResult<String> = PluginResult.Success("test")

        val kotlinResult = pluginResult.toResult()

        assertTrue(kotlinResult.isSuccess)
        assertEquals("test", kotlinResult.getOrNull())
    }

    @Test
    @Order(71)
    @DisplayName("error___toResult___returns_kotlin_failure")
    fun error___toResult___returns_kotlin_failure() {
        val pluginResult: PluginResult<String> = PluginResult.Error(1, "error")

        val kotlinResult = pluginResult.toResult()

        assertTrue(kotlinResult.isFailure)
    }

    // ==================== toPluginResult() extension ====================

    @Test
    @Order(80)
    @DisplayName("kotlin_success___toPluginResult___returns_success")
    fun kotlin_success___toPluginResult___returns_success() {
        val kotlinResult = Result.success("test")

        val pluginResult = kotlinResult.toPluginResult()

        assertTrue(pluginResult.isSuccess)
        assertEquals("test", pluginResult.getOrNull())
    }

    @Test
    @Order(81)
    @DisplayName("kotlin_failure___toPluginResult___returns_error")
    fun kotlin_failure___toPluginResult___returns_error() {
        val kotlinResult = Result.failure<String>(PluginException(42, "error"))

        val pluginResult = kotlinResult.toPluginResult()

        assertTrue(pluginResult.isError)
        assertEquals(42, (pluginResult as PluginResult.Error).code)
    }
}
