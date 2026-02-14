.PHONY: build-plugin test test-unit test-integration bench fmt vet

WORKSPACE_ROOT := $(shell cd .. && pwd)

# Build the hello-plugin for integration tests
build-plugin:
	cd $(WORKSPACE_ROOT) && cargo build --release -p hello-plugin

# Run all tests
test: build-plugin
	go test -v ./...

# Run unit tests only (no plugin needed)
test-unit:
	go test -run 'Test(LogLevel|ParseLogLevel|LifecycleState|CanHandle|IsTerminal|PluginError|IsPluginError|ErrorCode|Config|ResponseEnvelope)' -v ./...

# Run integration tests only (requires hello-plugin)
test-integration: build-plugin
	go test -run 'Test(Load|Call|Close|SetLogLevel|RejectedRequest|HasBinaryTransport|CallRaw|StructSizes)' -v ./...

# Run benchmarks
bench: build-plugin
	go test -bench=. -benchmem -count=3 ./...

# Format code
fmt:
	gofmt -w .

# Run go vet
vet:
	go vet ./...
