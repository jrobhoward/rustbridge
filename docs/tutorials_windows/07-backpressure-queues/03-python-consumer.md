# Section 3: Python Consumer

In this section, you'll implement synchronized plugin access in Python using `queue.Queue` and `concurrent.futures`.

## Prerequisites

Complete the [project setup](./README.md#project-setup) from the chapter introduction.

## Set Up the Python Environment

```powershell
cd $env:USERPROFILE\rustbridge-workspace\sync-demo\consumers\python
python -m venv .venv
.\.venv\Scripts\Activate.ps1
pip install -r requirements.txt
```

## Install rustbridge Python Package

```powershell
pip install -e $env:USERPROFILE\rustbridge-workspace\rustbridge\rustbridge-python
```

## Verify the Generated Consumer

```powershell
Copy-Item ..\..\sync-demo-0.1.0.rbp .
python main.py
```

You should see:

```
Response: Hello from Python!
Length: 18
```

## Add the SynchronizedPlugin Wrapper

Create `synchronized_plugin.py` with a thread-safe wrapper. See the [Linux tutorial](../tutorials/07-backpressure-queues/03-python-consumer.md) for the complete implementation.

## Key Implementation Details

### queue.Queue

```python
self._work_queue: queue.Queue[PluginWorkItem | None] = queue.Queue(
    maxsize=max_queue_size
)
```

- **Bounded**: `maxsize` enables backpressure
- **Blocking**: `put()` blocks when full
- **Thread-safe**: Uses internal locks
- **Sentinel value**: `None` signals shutdown

### concurrent.futures.Future

```python
completion: Future[str] = Future()
# ... later in worker:
completion.set_result(response)
# or
completion.set_exception(e)
```

## Run the Demo

```powershell
python main.py
```

## Summary

You've implemented the synchronized plugin access pattern in three languages:

| Language | Queue Type | Future Type | Key Classes |
|----------|-----------|-------------|-------------|
| C# | `BlockingCollection<T>` | `Task<T>` | `TaskCompletionSource` |
| Java | `ArrayBlockingQueue` | `CompletableFuture<T>` | `Thread` |
| Python | `queue.Queue` | `Future` | `threading.Thread` |

## What's Next?

Continue to [Chapter 8: Binary Transport](../08-binary-transport/README.md) for high-performance data handling.
