#!/bin/bash
set -e

cargo publish -p rustbridge-core
sleep 30

cargo publish -p rustbridge-bundle
sleep 30

cargo publish -p rustbridge-transport
sleep 30

cargo publish -p rustbridge-consumer
sleep 30

cargo publish -p rustbridge-logging
sleep 30

cargo publish -p rustbridge-macros
sleep 30

cargo publish -p rustbridge-runtime
sleep 30

cargo publish -p rustbridge-ffi
sleep 30

cargo publish -p rustbridge-cli
sleep 30

cargo publish -p rustbridge

echo "All crates published successfully!"
