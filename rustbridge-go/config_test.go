package rustbridge

import (
	"encoding/json"
	"testing"
)

func TestConfig___Defaults___HasExpectedValues(t *testing.T) {
	cfg := defaultConfig()

	if cfg.logLevel != "info" {
		t.Errorf("default logLevel = %q, want \"info\"", cfg.logLevel)
	}
	if cfg.maxConcurrent != 1000 {
		t.Errorf("default maxConcurrent = %d, want 1000", cfg.maxConcurrent)
	}
	if cfg.shutdownTimeout != 5000 {
		t.Errorf("default shutdownTimeout = %d, want 5000", cfg.shutdownTimeout)
	}
	if cfg.workerThreads != nil {
		t.Errorf("default workerThreads should be nil")
	}
}

func TestConfig___ToJSON___DefaultsSerializeCorrectly(t *testing.T) {
	cfg := defaultConfig()

	data, err := cfg.toJSON()

	if err != nil {
		t.Fatalf("toJSON() error: %v", err)
	}

	var m map[string]any
	if err := json.Unmarshal(data, &m); err != nil {
		t.Fatalf("json.Unmarshal error: %v", err)
	}

	if m["log_level"] != "info" {
		t.Errorf("log_level = %v", m["log_level"])
	}
	if m["max_concurrent_ops"] != float64(1000) {
		t.Errorf("max_concurrent_ops = %v", m["max_concurrent_ops"])
	}
	if m["shutdown_timeout_ms"] != float64(5000) {
		t.Errorf("shutdown_timeout_ms = %v", m["shutdown_timeout_ms"])
	}
	if _, ok := m["worker_threads"]; ok {
		t.Error("worker_threads should not be present in defaults")
	}
}

func TestConfig___WithOptions___SerializesAllFields(t *testing.T) {
	cfg := defaultConfig()
	opts := []Option{
		WithLogLevel(LogLevelDebug),
		WithWorkerThreads(4),
		WithMaxConcurrentOps(500),
		WithShutdownTimeout(10000),
		WithData("my_key", "my_value"),
		WithInitParam("seed", true),
	}
	for _, opt := range opts {
		opt(cfg)
	}

	data, err := cfg.toJSON()

	if err != nil {
		t.Fatalf("toJSON() error: %v", err)
	}

	var m map[string]any
	if err := json.Unmarshal(data, &m); err != nil {
		t.Fatalf("json.Unmarshal error: %v", err)
	}

	if m["log_level"] != "debug" {
		t.Errorf("log_level = %v, want \"debug\"", m["log_level"])
	}
	if m["worker_threads"] != float64(4) {
		t.Errorf("worker_threads = %v, want 4", m["worker_threads"])
	}
	if m["max_concurrent_ops"] != float64(500) {
		t.Errorf("max_concurrent_ops = %v, want 500", m["max_concurrent_ops"])
	}
	if m["shutdown_timeout_ms"] != float64(10000) {
		t.Errorf("shutdown_timeout_ms = %v, want 10000", m["shutdown_timeout_ms"])
	}

	dataMap, ok := m["data"].(map[string]any)
	if !ok {
		t.Fatal("data is not a map")
	}
	if dataMap["my_key"] != "my_value" {
		t.Errorf("data.my_key = %v", dataMap["my_key"])
	}

	initMap, ok := m["init_params"].(map[string]any)
	if !ok {
		t.Fatal("init_params is not a map")
	}
	if initMap["seed"] != true {
		t.Errorf("init_params.seed = %v", initMap["seed"])
	}
}
