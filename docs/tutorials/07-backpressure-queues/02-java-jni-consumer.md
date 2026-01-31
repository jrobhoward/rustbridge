# Section 2: Java/JNI Consumer

In this section, you'll implement synchronized plugin access in Java using `BlockingQueue` and `CompletableFuture`.

## Prerequisites

Complete the [project setup](./README.md#project-setup) from the chapter introduction:

1. Scaffold the project with `rustbridge new sync-demo --all`
2. Add the sleep handler to `src/lib.rs`
3. Build the plugin and create the bundle
4. Copy the bundle to `consumers/java-jni/`

## Install rustbridge Java Libraries

If you haven't already, install the Java libraries to Maven local:

```bash
cd ~/rustbridge-workspace/rustbridge/rustbridge-java
./gradlew publishToMavenLocal
```

## Verify the Generated Consumer

The scaffolded Java JNI consumer is in `consumers/java-jni/`. First, copy the bundle and verify it works:

```bash
cd ~/rustbridge-workspace/sync-demo/consumers/java-jni
cp ../../sync-demo-1.0.0.rbp .
./gradlew run
```

You should see:

```
Response: Hello from Java JNI!
Length: 20
```

## Understanding the Generated Code

Look at `src/main/java/com/example/Main.java`:

```java
package com.example;

import com.rustbridge.BundleLoader;
import com.rustbridge.Plugin;
import com.rustbridge.jni.JniPluginLoader;
import com.google.gson.Gson;

public class Main {
    static class EchoRequest {
        String message;
        EchoRequest(String message) { this.message = message; }
    }

    static class EchoResponse {
        String message;
        int length;
    }

    private static final Gson gson = new Gson();

    public static void main(String[] args) throws Exception {
        String bundlePath = "sync-demo-1.0.0.rbp";

        BundleLoader bundleLoader = BundleLoader.builder()
            .bundlePath(bundlePath)
            .verifySignatures(false)
            .build();

        String libraryPath = bundleLoader.extractLibrary().toString();

        try (Plugin plugin = JniPluginLoader.load(libraryPath)) {
            EchoRequest request = new EchoRequest("Hello from Java JNI!");
            String requestJson = gson.toJson(request);

            String responseJson = plugin.call("echo", requestJson);
            EchoResponse response = gson.fromJson(responseJson, EchoResponse.class);

            System.out.println("Response: " + response.message);
            System.out.println("Length: " + response.length);
        }

        bundleLoader.close();
    }
}
```

Key points:
- `BundleLoader.builder()` extracts the native library from the bundle
- `JniPluginLoader.load()` loads the plugin using JNI (works with Java 17+)
- Gson handles JSON serialization with snake_case field names

## Add the SynchronizedPlugin Wrapper

Create `src/main/java/com/example/SynchronizedPlugin.java`:

```java
package com.example;

import com.rustbridge.Plugin;
import com.rustbridge.PluginException;
import com.google.gson.FieldNamingPolicy;
import com.google.gson.Gson;
import com.google.gson.GsonBuilder;

import java.io.Closeable;
import java.util.concurrent.*;

/**
 * A work item representing a pending plugin call.
 */
record PluginWorkItem(
    String typeTag,
    String request,
    CompletableFuture<String> completion
) {}

/**
 * Thread-safe wrapper that serializes all plugin calls through a single worker thread.
 */
public class SynchronizedPlugin implements Closeable {
    private static final Gson GSON = new GsonBuilder()
        .setFieldNamingPolicy(FieldNamingPolicy.LOWER_CASE_WITH_UNDERSCORES)
        .create();

    private final Plugin plugin;
    private final BlockingQueue<PluginWorkItem> workQueue;
    private final Thread workerThread;
    private final int maxQueueSize;
    private volatile boolean shutdown = false;

    public SynchronizedPlugin(Plugin plugin, int maxQueueSize) {
        this.plugin = plugin;
        this.maxQueueSize = maxQueueSize;
        this.workQueue = new ArrayBlockingQueue<>(maxQueueSize);

        this.workerThread = new Thread(this::workerLoop, "SynchronizedPlugin-Worker");
        this.workerThread.setDaemon(true);
        this.workerThread.start();
    }

    public SynchronizedPlugin(Plugin plugin) {
        this(plugin, 100);
    }

    public int getPendingCount() {
        return workQueue.size();
    }

    public CompletableFuture<String> callAsync(String typeTag, String request) {
        if (shutdown) {
            return CompletableFuture.failedFuture(
                new IllegalStateException("SynchronizedPlugin has been shut down"));
        }

        var completion = new CompletableFuture<String>();
        var workItem = new PluginWorkItem(typeTag, request, completion);

        try {
            // This blocks if queue is full - that's the backpressure!
            workQueue.put(workItem);
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
            completion.completeExceptionally(e);
        }

        return completion;
    }

    public String call(String typeTag, String request) throws PluginException {
        try {
            return callAsync(typeTag, request).get();
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
            throw new PluginException("Call interrupted", e);
        } catch (ExecutionException e) {
            Throwable cause = e.getCause();
            if (cause instanceof PluginException pe) {
                throw pe;
            }
            throw new PluginException("Call failed: " + cause.getMessage(), cause);
        }
    }

    public <T, R> CompletableFuture<R> callAsync(String typeTag, T request, Class<R> responseType) {
        String requestJson = GSON.toJson(request);
        return callAsync(typeTag, requestJson)
            .thenApply(responseJson -> GSON.fromJson(responseJson, responseType));
    }

    public <T, R> R call(String typeTag, T request, Class<R> responseType) throws PluginException {
        try {
            return callAsync(typeTag, request, responseType).get();
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
            throw new PluginException("Call interrupted", e);
        } catch (ExecutionException e) {
            Throwable cause = e.getCause();
            if (cause instanceof PluginException pe) {
                throw pe;
            }
            throw new PluginException("Call failed: " + cause.getMessage(), cause);
        }
    }

    private void workerLoop() {
        while (!shutdown || !workQueue.isEmpty()) {
            try {
                PluginWorkItem workItem = workQueue.poll(100, TimeUnit.MILLISECONDS);
                if (workItem == null) {
                    continue;
                }

                try {
                    String response = plugin.call(workItem.typeTag(), workItem.request());
                    workItem.completion().complete(response);
                } catch (Exception e) {
                    workItem.completion().completeExceptionally(e);
                }
            } catch (InterruptedException e) {
                Thread.currentThread().interrupt();
                break;
            }
        }

        // Fail any remaining items
        PluginWorkItem remaining;
        while ((remaining = workQueue.poll()) != null) {
            remaining.completion().completeExceptionally(
                new IllegalStateException("SynchronizedPlugin has been shut down"));
        }
    }

    @Override
    public void close() {
        if (shutdown) {
            return;
        }
        shutdown = true;

        try {
            workerThread.join(5000);
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
        }

        plugin.close();
    }
}
```

## Add Message Types

Create `src/main/java/com/example/Messages.java`:

```java
package com.example;

/**
 * Message types for the sync-demo plugin.
 * Field names use snake_case to match Rust's serde conventions.
 */
public class Messages {

    public static class EchoRequest {
        public String message;

        public EchoRequest(String message) {
            this.message = message;
        }
    }

    public static class EchoResponse {
        public String message;
        public int length;
    }

    public static class SleepRequest {
        public long duration_ms;

        public SleepRequest(long durationMs) {
            this.duration_ms = durationMs;
        }
    }

    public static class SleepResponse {
        public long slept_ms;
    }
}
```

## Update Main.java

Replace `src/main/java/com/example/Main.java`:

```java
package com.example;

import com.rustbridge.BundleLoader;
import com.rustbridge.jni.JniPluginLoader;
import com.example.Messages.*;

import java.util.ArrayList;
import java.util.concurrent.CompletableFuture;

public class Main {

    public static void main(String[] args) throws Exception {
        System.out.println("=== Synchronized Plugin Demo (Java/JNI) ===\n");

        String bundlePath = "sync-demo-1.0.0.rbp";

        BundleLoader bundleLoader = BundleLoader.builder()
            .bundlePath(bundlePath)
            .verifySignatures(false)
            .build();

        var plugin = JniPluginLoader.load(bundleLoader.extractLibrary().toString());

        // Wrap with synchronized access (queue size = 5 for demo)
        try (var syncPlugin = new SynchronizedPlugin(plugin, 5)) {

            // Demo 1: Sequential calls
            System.out.println("Demo 1: Sequential calls");
            for (int i = 0; i < 3; i++) {
                var response = syncPlugin.call(
                    "echo",
                    new EchoRequest("Message " + i),
                    EchoResponse.class);
                System.out.printf("  Echo: %s (len=%d)%n",
                    response.message, response.length);
            }

            // Demo 2: Concurrent calls showing serialization
            System.out.println("\nDemo 2: Concurrent calls (observe serialization)");
            var futures = new ArrayList<CompletableFuture<Void>>();
            long startTime = System.currentTimeMillis();

            for (int i = 0; i < 10; i++) {
                final int id = i;
                var future = CompletableFuture.runAsync(() -> {
                    System.out.printf("  [%d] Submitting (queue: %d)%n",
                        id, syncPlugin.getPendingCount());

                    try {
                        syncPlugin.call(
                            "sleep",
                            new SleepRequest(100),  // 100ms sleep
                            SleepResponse.class);

                        System.out.printf("  [%d] Completed after %dms%n",
                            id, System.currentTimeMillis() - startTime);
                    } catch (Exception e) {
                        System.err.printf("  [%d] Error: %s%n", id, e.getMessage());
                    }
                });
                futures.add(future);
            }

            CompletableFuture.allOf(futures.toArray(new CompletableFuture[0])).join();
            System.out.printf("%nTotal time: %dms%n", System.currentTimeMillis() - startTime);
            System.out.println("  (Expected ~1000ms for 10 x 100ms if serialized)");

            // Demo 3: Backpressure
            System.out.println("\nDemo 3: Backpressure (queue size = 5)");
            long pressureStart = System.currentTimeMillis();

            var pressureFutures = new ArrayList<CompletableFuture<Void>>();
            for (int i = 0; i < 20; i++) {
                final int id = i;
                var future = CompletableFuture.runAsync(() -> {
                    long submitTime = System.currentTimeMillis() - pressureStart;

                    try {
                        syncPlugin.call(
                            "sleep",
                            new SleepRequest(50),  // 50ms sleep
                            SleepResponse.class);

                        long completeTime = System.currentTimeMillis() - pressureStart;
                        System.out.printf("  [%02d] Submit@%dms, Complete@%dms%n",
                            id, submitTime, completeTime);
                    } catch (Exception e) {
                        System.err.printf("  [%02d] Error: %s%n", id, e.getMessage());
                    }
                });
                pressureFutures.add(future);
            }

            CompletableFuture.allOf(pressureFutures.toArray(new CompletableFuture[0])).join();
            System.out.printf("%nTotal time: %dms%n",
                System.currentTimeMillis() - pressureStart);
        }

        bundleLoader.close();
        System.out.println("\n=== Demo Complete ===");
    }
}
```

## Run the Demo

```bash
./gradlew run
```

Expected output:

```
=== Synchronized Plugin Demo (Java/JNI) ===

Demo 1: Sequential calls
  Echo: Message 0 (len=9)
  Echo: Message 1 (len=9)
  Echo: Message 2 (len=9)

Demo 2: Concurrent calls (observe serialization)
  [0] Submitting (queue: 0)
  [1] Submitting (queue: 1)
  ...
  [0] Completed after 105ms
  [1] Completed after 207ms
  ...

Total time: 1012ms
  (Expected ~1000ms for 10 x 100ms if serialized)

Demo 3: Backpressure (queue size = 5)
  [00] Submit@2ms, Complete@53ms
  [01] Submit@2ms, Complete@105ms
  ...

Total time: 1055ms

=== Demo Complete ===
```

## Understanding the Implementation

### ArrayBlockingQueue

```java
this.workQueue = new ArrayBlockingQueue<>(maxQueueSize);
```

- **Bounded**: Fixed capacity enables backpressure
- **Blocking**: `put()` blocks when full, `take()` blocks when empty
- **FIFO**: Maintains request ordering

### CompletableFuture

```java
var completion = new CompletableFuture<String>();
// ... later in worker:
completion.complete(response);
// or
completion.completeExceptionally(e);
```

- **Async bridge**: Connects caller's `get()` to worker's completion
- **Composable**: Chain operations with `thenApply`, `thenCompose`
- **Exception handling**: Propagates errors via `completeExceptionally`

### Gson Field Naming

```java
private static final Gson GSON = new GsonBuilder()
    .setFieldNamingPolicy(FieldNamingPolicy.LOWER_CASE_WITH_UNDERSCORES)
    .create();
```

This converts Java's `camelCase` to Rust's `snake_case` automatically.

## Error Handling

Errors are propagated through the `CompletableFuture`:

```java
try {
    var response = syncPlugin.call("invalid.tag", "{}");
} catch (PluginException e) {
    System.out.println("Plugin error: " + e.getMessage());
}
```

## What's Next?

Continue to the Python implementation.

[Continue to Section 3: Python Consumer →](./03-python-consumer.md)
