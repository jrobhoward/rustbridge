# Section 3: Python Consumer

In this section, you'll implement synchronized plugin access in Python using `queue.Queue` and `concurrent.futures`.

## Prerequisites

Complete the [project setup](./README.md#project-setup) from the chapter introduction:

1. Scaffold the project with `rustbridge new sync-demo --all`
2. Add the sleep handler to `src/lib.rs`
3. Build the plugin and create the bundle
4. Copy the bundle to `consumers/python/`

## Set Up the Python Environment

```bash
cd ~/rustbridge-workspace/sync-demo/consumers/python
python3 -m venv .venv
source .venv/bin/activate  # On Windows: .venv\Scripts\activate
pip install -r requirements.txt
```

## Install rustbridge Python Package

```bash
pip install -e ~/rustbridge-workspace/rustbridge/rustbridge-python
```

## Verify the Generated Consumer

Copy the bundle and verify it works:

```bash
cp ../../sync-demo-0.1.0.rbp .
python main.py
```

You should see:

```
Response: Hello from Python!
Length: 18
```

## Understanding the Generated Code

Look at `main.py`:

```python
import json
from rustbridge.core import BundleLoader

bundle_path = "sync-demo-0.1.0.rbp"

loader = BundleLoader(verify_signatures=False)
with loader.load(bundle_path) as plugin:
    request = {"message": "Hello from Python!"}
    response_json = plugin.call("echo", json.dumps(request))
    response = json.loads(response_json)

    print(f"Response: {response['message']}")
    print(f"Length: {response['length']}")
```

Key points:
- `BundleLoader(verify_signatures=False)` creates a loader (disable signature verification for development)
- `loader.load()` extracts the library and returns a plugin in one step
- JSON uses snake_case field names to match Rust

## Add the SynchronizedPlugin Wrapper

Create `synchronized_plugin.py`:

```python
"""Thread-safe wrapper that serializes all plugin calls through a single worker thread."""

from __future__ import annotations

import json
import queue
import threading
from concurrent.futures import Future
from dataclasses import dataclass
from typing import Any, TypeVar

from rustbridge.core.plugin_exception import PluginException
from rustbridge.native.native_plugin import NativePlugin

T = TypeVar("T")


@dataclass
class PluginWorkItem:
    """A work item representing a pending plugin call."""

    type_tag: str
    request: str
    completion: Future[str]


class SynchronizedPlugin:
    """
    Thread-safe wrapper that serializes all plugin calls through a single worker thread.

    This wrapper provides:
    - Serialized access: Only one call executes at a time
    - Backpressure: Callers block when the queue is full
    - Future API: Returns Future that completes when processed
    """

    def __init__(self, plugin: NativePlugin, max_queue_size: int = 100) -> None:
        self._plugin = plugin
        self._max_queue_size = max_queue_size
        self._shutdown = False
        self._lock = threading.Lock()

        # Bounded queue provides backpressure
        self._work_queue: queue.Queue[PluginWorkItem | None] = queue.Queue(
            maxsize=max_queue_size
        )

        # Start the worker thread
        self._worker_thread = threading.Thread(
            target=self._worker_loop,
            name="SynchronizedPlugin-Worker",
            daemon=True,
        )
        self._worker_thread.start()

    @property
    def pending_count(self) -> int:
        """Current number of pending requests in the queue."""
        return self._work_queue.qsize()

    @property
    def max_queue_size(self) -> int:
        """Maximum queue capacity."""
        return self._max_queue_size

    def submit(self, type_tag: str, request: str) -> Future[str]:
        """
        Submit a request asynchronously.

        This method blocks if the queue is full (backpressure).
        """
        with self._lock:
            if self._shutdown:
                future: Future[str] = Future()
                future.set_exception(
                    RuntimeError("SynchronizedPlugin has been shut down")
                )
                return future

        completion: Future[str] = Future()
        work_item = PluginWorkItem(
            type_tag=type_tag,
            request=request,
            completion=completion,
        )

        # This blocks if queue is full - that's the backpressure!
        self._work_queue.put(work_item)

        return completion

    def call(self, type_tag: str, request: str, timeout: float | None = None) -> str:
        """Submit a request and wait for the result (blocking)."""
        future = self.submit(type_tag, request)
        return future.result(timeout=timeout)

    def submit_typed(self, type_tag: str, request: Any) -> Future[Any]:
        """Submit a typed request asynchronously."""
        request_json = json.dumps(request)
        str_future = self.submit(type_tag, request_json)

        result_future: Future[Any] = Future()

        def on_done(f: Future[str]) -> None:
            try:
                response_json = f.result()
                result_future.set_result(json.loads(response_json))
            except Exception as e:
                result_future.set_exception(e)

        str_future.add_done_callback(on_done)
        return result_future

    def call_typed(
        self, type_tag: str, request: Any, timeout: float | None = None
    ) -> Any:
        """Submit a typed request and wait for the result (blocking)."""
        future = self.submit_typed(type_tag, request)
        return future.result(timeout=timeout)

    def _worker_loop(self) -> None:
        """Worker thread main loop."""
        while True:
            try:
                work_item = self._work_queue.get(timeout=0.1)
            except queue.Empty:
                with self._lock:
                    if self._shutdown and self._work_queue.empty():
                        break
                continue

            if work_item is None:
                break

            try:
                response = self._plugin.call(work_item.type_tag, work_item.request)
                work_item.completion.set_result(response)
            except Exception as e:
                work_item.completion.set_exception(e)
            finally:
                self._work_queue.task_done()

        # Fail any remaining items
        while True:
            try:
                work_item = self._work_queue.get_nowait()
                if work_item is not None:
                    work_item.completion.set_exception(
                        RuntimeError("SynchronizedPlugin has been shut down")
                    )
            except queue.Empty:
                break

    def shutdown(self, wait: bool = True, timeout: float = 5.0) -> None:
        """Shutdown the wrapper."""
        with self._lock:
            if self._shutdown:
                return
            self._shutdown = True

        self._work_queue.put(None)

        if wait:
            self._worker_thread.join(timeout=timeout)

        self._plugin.shutdown()

    def close(self) -> None:
        """Close the wrapper (alias for shutdown)."""
        self.shutdown()

    def __enter__(self) -> "SynchronizedPlugin":
        return self

    def __exit__(self, exc_type: Any, exc_val: Any, exc_tb: Any) -> None:
        self.shutdown()
```

## Update main.py

Replace `main.py` with the synchronized demo:

```python
#!/usr/bin/env python3
"""Synchronized plugin access demo."""

from __future__ import annotations

import threading
import time
from concurrent.futures import Future, as_completed

from rustbridge.core import BundleLoader

from synchronized_plugin import SynchronizedPlugin


def main() -> None:
    print("=== Synchronized Plugin Demo (Python) ===\n")

    # Load the plugin from bundle
    loader = BundleLoader(verify_signatures=False)
    plugin = loader.load("sync-demo-0.1.0.rbp")

    # Wrap with synchronized access (queue size = 5 for demo)
    with SynchronizedPlugin(plugin, max_queue_size=5) as sync_plugin:

        # Demo 1: Sequential calls
        print("Demo 1: Sequential calls")
        for i in range(3):
            response = sync_plugin.call_typed("echo", {"message": f"Message {i}"})
            print(f"  Echo: {response['message']} (len={response['length']})")

        # Demo 2: Concurrent calls showing serialization
        print("\nDemo 2: Concurrent calls (observe serialization)")
        start_time = time.time()

        def make_call(call_id: int) -> dict:
            print(f"  [{call_id}] Submitting (queue: {sync_plugin.pending_count})")
            response = sync_plugin.call_typed(
                "sleep", {"duration_ms": 100}  # 100ms sleep
            )
            elapsed = int((time.time() - start_time) * 1000)
            print(f"  [{call_id}] Completed after {elapsed}ms")
            return response

        threads = []
        for i in range(10):
            t = threading.Thread(target=make_call, args=(i,))
            threads.append(t)
            t.start()

        for t in threads:
            t.join()

        total_time = int((time.time() - start_time) * 1000)
        print(f"\nTotal time: {total_time}ms")
        print("  (Expected ~1000ms for 10 x 100ms if serialized)")

        # Demo 3: Backpressure
        print("\nDemo 3: Backpressure (queue size = 5)")
        pressure_start = time.time()

        def pressure_call(call_id: int) -> None:
            submit_time = int((time.time() - pressure_start) * 1000)
            sync_plugin.call_typed("sleep", {"duration_ms": 50})  # 50ms sleep
            complete_time = int((time.time() - pressure_start) * 1000)
            print(f"  [{call_id:02d}] Submit@{submit_time}ms, Complete@{complete_time}ms")

        pressure_threads = []
        for i in range(20):
            t = threading.Thread(target=pressure_call, args=(i,))
            pressure_threads.append(t)
            t.start()

        for t in pressure_threads:
            t.join()

        pressure_time = int((time.time() - pressure_start) * 1000)
        print(f"\nTotal time: {pressure_time}ms")

        # Demo 4: Async with futures
        print("\nDemo 4: Async with futures")
        futures = [
            sync_plugin.submit_typed("echo", {"message": "Alice"}),
            sync_plugin.submit_typed("echo", {"message": "Bob"}),
            sync_plugin.submit_typed("echo", {"message": "Charlie"}),
        ]

        for future in as_completed(futures):
            response = future.result()
            print(f"  Echo: {response['message']}")

    print("\n=== Demo Complete ===")


if __name__ == "__main__":
    main()
```

## Run the Demo

```bash
python main.py
```

Expected output:

```
=== Synchronized Plugin Demo (Python) ===

Demo 1: Sequential calls
  Echo: Message 0 (len=9)
  Echo: Message 1 (len=9)
  Echo: Message 2 (len=9)

Demo 2: Concurrent calls (observe serialization)
  [0] Submitting (queue: 0)
  [1] Submitting (queue: 1)
  ...
  [0] Completed after 103ms
  [1] Completed after 206ms
  ...

Total time: 1015ms
  (Expected ~1000ms for 10 x 100ms if serialized)

Demo 3: Backpressure (queue size = 5)
  [00] Submit@1ms, Complete@52ms
  [01] Submit@1ms, Complete@103ms
  ...

Total time: 1053ms

Demo 4: Async with futures
  Echo: Alice
  Echo: Bob
  Echo: Charlie

=== Demo Complete ===
```

## Understanding the Implementation

### queue.Queue

```python
self._work_queue: queue.Queue[PluginWorkItem | None] = queue.Queue(
    maxsize=max_queue_size
)
```

- **Bounded**: `maxsize` enables backpressure
- **Blocking**: `put()` blocks when full, `get()` blocks when empty
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

- **Async bridge**: Connects caller's `result()` to worker's completion
- **Callbacks**: `add_done_callback()` for async notification
- **Timeout support**: `result(timeout=...)` raises `TimeoutError`

## Error Handling

Errors are propagated through the `Future`:

```python
try:
    response = sync_plugin.call("invalid.tag", "{}")
except PluginException as e:
    print(f"Plugin error: {e}")

# Or with Future:
future = sync_plugin.submit("invalid.tag", "{}")
try:
    result = future.result()
except PluginException as e:
    print(f"Plugin error: {e}")
```

## Summary

You've now implemented the synchronized plugin access pattern in all three languages:

| Language | Queue Type | Future Type | Key Classes |
|----------|-----------|-------------|-------------|
| C# | `BlockingCollection<T>` | `Task<T>` | `TaskCompletionSource` |
| Java | `ArrayBlockingQueue` | `CompletableFuture<T>` | `Thread` |
| Python | `queue.Queue` | `Future` | `threading.Thread` |

The pattern is consistent across languages:
1. Bounded queue for backpressure
2. Single worker thread for serialization
3. Future/Task for async result delivery
4. Graceful shutdown with queue draining

## Next Steps

You now have production-ready patterns for serialized plugin access. Consider:

- Adding metrics and monitoring for production use
- Implementing circuit breakers for failure resilience
- Adding request prioritization if needed
- Combining with connection pooling for multiple plugin instances
