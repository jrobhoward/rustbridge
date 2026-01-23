package com.rustbridge.examples

import com.rustbridge.LogLevel
import com.rustbridge.PluginConfig
import com.rustbridge.ffm.FfmPluginLoader

/**
 * Basic example demonstrating idiomatic Kotlin usage of rustbridge.
 *
 * This example shows:
 * - Using the `use` block for automatic resource management
 * - Data classes for type-safe JSON serialization
 * - Extension functions for cleaner API
 * - String templates for JSON construction
 */

fun main() {
    println("=== rustbridge Kotlin Basic Example ===\n")

    // Locate the plugin library
    val pluginPath = findPluginPath()
    println("Loading plugin from: $pluginPath\n")

    // Configure the plugin (using fluent API)
    val config = PluginConfig()
        .logLevel(LogLevel.INFO)
        .workerThreads(4)

    // Use block ensures automatic cleanup
    FfmPluginLoader.load(pluginPath, config).use { plugin ->

        // Example 1: Echo message
        println("1. Echo Example:")
        val echoResponse = plugin.callTyped<EchoResponse>(
            "echo",
            EchoRequest(message = "Hello from Kotlin!")
        )
        println("   Response: ${echoResponse.message}")
        println("   Length: ${echoResponse.length}\n")

        // Example 2: Greet user
        println("2. Greet Example:")
        val greetResponse = plugin.callTyped<GreetResponse>(
            "greet",
            GreetRequest(name = "Kotlin Developer")
        )
        println("   ${greetResponse.greeting}\n")

        // Example 3: Create user
        println("3. Create User Example:")
        val userResponse = plugin.callTyped<CreateUserResponse>(
            "user.create",
            CreateUserRequest(
                username = "kotlinuser",
                email = "kotlin@example.com"
            )
        )
        println("   User ID: ${userResponse.user_id}")
        println("   Created at: ${userResponse.created_at}\n")

        // Example 4: Math operation
        println("4. Math Add Example:")
        val mathResponse = plugin.callTyped<AddResponse>(
            "math.add",
            AddRequest(a = 42, b = 58)
        )
        println("   42 + 58 = ${mathResponse.result}\n")

        // Example 5: Multiple calls (plugin is stateful)
        println("5. Multiple User Creation:")
        repeat(3) { i ->
            val response = plugin.callTyped<CreateUserResponse>(
                "user.create",
                CreateUserRequest(
                    username = "user$i",
                    email = "user$i@example.com"
                )
            )
            println("   Created ${response.user_id}")
        }
    }
    // Plugin automatically shutdown when leaving `use` block

    println("\n=== Example Complete ===")
}
