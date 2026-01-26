# Tasks

Future work and feature ideas for rustbridge.

## Planned

### Plugin Diagnostics API

Implement comprehensive diagnostics that host languages can call to observe plugin characteristics.

**Currently implemented:**
- `getRejectedRequestCount()` - requests rejected due to concurrency limits
- `getState()` - lifecycle state

**Proposed metrics:**
- Memory/heap consumption
- CPU usage
- Total successful calls
- Total failed/error calls
- Call latency (min/max/avg/p99)
- Active concurrent request count
- Throughput (requests/sec)
- Health check endpoint

**Considerations:**
- Some metrics (CPU, memory) may require platform-specific implementations
- Consider a structured `PluginDiagnostics` response object vs individual getters
- Latency tracking adds overhead - consider making it opt-in via config
- May want time-windowed metrics (last 1m, 5m, 15m) vs cumulative
