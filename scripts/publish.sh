#!/bin/bash
set -e

# Tier 1: No internal dependencies
cargo publish -p rustbridge-core
sleep 30

cargo publish -p rustbridge-bundle
sleep 30

# Tier 1.5: Depends on core + bundle + transport only
cargo publish -p rustbridge-consumer
sleep 30

# Tier 2: Depend on tier 1
cargo publish -p rustbridge-transport
sleep 30

cargo publish -p rustbridge-logging
sleep 30

cargo publish -p rustbridge-macros
sleep 30

cargo publish -p rustbridge-runtime
sleep 30

# Tier 3: Depend on tier 2
cargo publish -p rustbridge-ffi
sleep 30

cargo publish -p rustbridge-cli
sleep 30

# Tier 4: Facade crate (depends on tier 3)
cargo publish -p rustbridge

echo "All crates published successfully!"
