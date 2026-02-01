# Section 2: Java/JNI Consumer

In this section, you'll implement synchronized plugin access in Java using `BlockingQueue` and `CompletableFuture`.

## Prerequisites

Complete the [project setup](./README.md#project-setup) from the chapter introduction.

## Install rustbridge Java Libraries

```powershell
cd $env:USERPROFILE\rustbridge-workspace\rustbridge\rustbridge-java
.\gradlew.bat publishToMavenLocal
```

## Verify the Generated Consumer

```powershell
cd $env:USERPROFILE\rustbridge-workspace\sync-demo\consumers\java-jni
Copy-Item ..\..\sync-demo-0.1.0.rbp .
.\gradlew.bat run
```

You should see:

```
Response: Hello from Java JNI!
Length: 20
```

## Add the SynchronizedPlugin Wrapper

Create `SynchronizedPlugin.java` with a thread-safe wrapper. See the [Linux tutorial](../tutorials/07-backpressure-queues/02-java-jni-consumer.md) for the complete implementation.

## Key Implementation Details

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

## Run the Demo

```powershell
.\gradlew.bat run
```

## What's Next?

Continue to the Python implementation.

[Continue to Section 3: Python Consumer →](./03-python-consumer.md)
