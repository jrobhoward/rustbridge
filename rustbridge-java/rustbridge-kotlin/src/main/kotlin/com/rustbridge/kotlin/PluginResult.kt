package com.rustbridge.kotlin

import com.rustbridge.PluginException

/**
 * A sealed class representing the result of a plugin call.
 *
 * This provides exhaustive pattern matching for plugin call results,
 * making it impossible to forget error handling.
 *
 * ```kotlin
 * when (val result = plugin.callSafe<Req, Resp>("echo", request)) {
 *     is PluginResult.Success -> handleSuccess(result.value)
 *     is PluginResult.Error -> handleError(result.code, result.message)
 * }
 * ```
 *
 * @param T the success value type
 */
sealed class PluginResult<out T> {

    /**
     * Represents a successful plugin call.
     *
     * @property value the response value
     */
    data class Success<T>(val value: T) : PluginResult<T>()

    /**
     * Represents a failed plugin call.
     *
     * @property code the error code (from PluginException, or 0 for other exceptions)
     * @property message the error message
     * @property cause the underlying exception, if any
     */
    data class Error(
        val code: Int,
        val message: String,
        val cause: Throwable? = null
    ) : PluginResult<Nothing>()

    /**
     * Returns true if this is a [Success].
     */
    val isSuccess: Boolean get() = this is Success

    /**
     * Returns true if this is an [Error].
     */
    val isError: Boolean get() = this is Error

    /**
     * Returns the success value, or null if this is an error.
     */
    fun getOrNull(): T? = when (this) {
        is Success -> value
        is Error -> null
    }

    /**
     * Returns the success value, or throws the error cause (or a PluginException).
     */
    fun getOrThrow(): T = when (this) {
        is Success -> value
        is Error -> throw cause ?: PluginException(code, message)
    }

    /**
     * Returns the success value, or the result of [default] if this is an error.
     */
    inline fun getOrElse(default: (Error) -> @UnsafeVariance T): T = when (this) {
        is Success -> value
        is Error -> default(this)
    }

    /**
     * Returns the success value, or [defaultValue] if this is an error.
     */
    fun getOrDefault(defaultValue: @UnsafeVariance T): T = when (this) {
        is Success -> value
        is Error -> defaultValue
    }

    /**
     * Transforms the success value using [transform].
     */
    inline fun <R> map(transform: (T) -> R): PluginResult<R> = when (this) {
        is Success -> Success(transform(value))
        is Error -> this
    }

    /**
     * Transforms the success value using [transform], which itself returns a [PluginResult].
     */
    inline fun <R> flatMap(transform: (T) -> PluginResult<R>): PluginResult<R> = when (this) {
        is Success -> transform(value)
        is Error -> this
    }

    /**
     * Transforms the error using [transform].
     */
    inline fun mapError(transform: (Error) -> Error): PluginResult<T> = when (this) {
        is Success -> this
        is Error -> transform(this)
    }

    /**
     * Recovers from an error by providing an alternative value.
     */
    inline fun recover(transform: (Error) -> @UnsafeVariance T): PluginResult<T> = when (this) {
        is Success -> this
        is Error -> Success(transform(this))
    }

    /**
     * Performs [action] if this is a [Success].
     */
    inline fun onSuccess(action: (T) -> Unit): PluginResult<T> {
        if (this is Success) action(value)
        return this
    }

    /**
     * Performs [action] if this is an [Error].
     */
    inline fun onError(action: (Error) -> Unit): PluginResult<T> {
        if (this is Error) action(this)
        return this
    }

    /**
     * Converts this [PluginResult] to a Kotlin [Result].
     */
    fun toResult(): Result<T> = when (this) {
        is Success -> Result.success(value)
        is Error -> Result.failure(cause ?: PluginException(code, message))
    }

    companion object {
        /**
         * Creates a [Success] result.
         */
        fun <T> success(value: T): PluginResult<T> = Success(value)

        /**
         * Creates an [Error] result.
         */
        fun error(code: Int, message: String, cause: Throwable? = null): PluginResult<Nothing> =
            Error(code, message, cause)

        /**
         * Creates a [PluginResult] from a [PluginException].
         */
        fun fromException(e: PluginException): PluginResult<Nothing> =
            Error(e.errorCode, e.message ?: "Unknown error", e)

        /**
         * Runs [block] and wraps the result in a [PluginResult].
         */
        inline fun <T> catching(block: () -> T): PluginResult<T> = try {
            Success(block())
        } catch (e: PluginException) {
            Error(e.errorCode, e.message ?: "Unknown error", e)
        } catch (e: Exception) {
            Error(0, e.message ?: "Unknown error", e)
        }
    }
}

/**
 * Converts a Kotlin [Result] to a [PluginResult].
 */
fun <T> Result<T>.toPluginResult(): PluginResult<T> = fold(
    onSuccess = { PluginResult.Success(it) },
    onFailure = { e ->
        when (e) {
            is PluginException -> PluginResult.Error(e.errorCode, e.message ?: "Unknown error", e)
            else -> PluginResult.Error(0, e.message ?: "Unknown error", e)
        }
    }
)
