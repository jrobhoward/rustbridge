package rustbridge

import "encoding/json"

// Option configures a Plugin during loading.
type Option func(*pluginConfig)

type pluginConfig struct {
	logLevel        string
	workerThreads   *int
	maxConcurrent   int
	shutdownTimeout int
	logHandler      LogHandler
	data            map[string]any
	initParams      map[string]any
}

func defaultConfig() *pluginConfig {
	return &pluginConfig{
		logLevel:        "info",
		maxConcurrent:   1000,
		shutdownTimeout: 5000,
	}
}

// WithLogLevel sets the log level for the plugin.
func WithLogLevel(level LogLevel) Option {
	return func(c *pluginConfig) {
		c.logLevel = level.String()
	}
}

// WithWorkerThreads sets the number of Tokio worker threads.
func WithWorkerThreads(n int) Option {
	return func(c *pluginConfig) {
		c.workerThreads = &n
	}
}

// WithMaxConcurrentOps sets the maximum number of concurrent operations.
// Set to 0 for unlimited concurrency.
func WithMaxConcurrentOps(n int) Option {
	return func(c *pluginConfig) {
		c.maxConcurrent = n
	}
}

// WithShutdownTimeout sets the shutdown timeout in milliseconds.
func WithShutdownTimeout(ms int) Option {
	return func(c *pluginConfig) {
		c.shutdownTimeout = ms
	}
}

// WithLogHandler sets a callback for receiving log messages from the plugin.
func WithLogHandler(handler LogHandler) Option {
	return func(c *pluginConfig) {
		c.logHandler = handler
	}
}

// WithData sets a custom configuration key-value pair.
func WithData(key string, value any) Option {
	return func(c *pluginConfig) {
		if c.data == nil {
			c.data = make(map[string]any)
		}
		c.data[key] = value
	}
}

// WithInitParam sets an initialization parameter.
func WithInitParam(key string, value any) Option {
	return func(c *pluginConfig) {
		if c.initParams == nil {
			c.initParams = make(map[string]any)
		}
		c.initParams[key] = value
	}
}

// toJSON serializes the config to JSON bytes for FFI.
func (c *pluginConfig) toJSON() ([]byte, error) {
	m := map[string]any{
		"log_level":          c.logLevel,
		"max_concurrent_ops": c.maxConcurrent,
		"shutdown_timeout_ms": c.shutdownTimeout,
	}

	if c.data != nil {
		m["data"] = c.data
	} else {
		m["data"] = map[string]any{}
	}

	if c.workerThreads != nil {
		m["worker_threads"] = *c.workerThreads
	}

	if c.initParams != nil {
		m["init_params"] = c.initParams
	}

	return json.Marshal(m)
}
