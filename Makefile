.PHONY: dev test build build-native native check clean fmt clippy

# Default target
all: check build

# Development: Install as dev extension in Zed
dev:
	@echo "To install as dev extension:"
	@echo "  1. Open Zed"
	@echo "  2. Press Cmd+Shift+P (Ctrl+Shift+P on Linux/Windows)"
	@echo "  3. Type 'zed: install dev extension'"
	@echo "  4. Select this directory: $(CURDIR)"

# Run tests
test:
	cargo test --all

# Build release WASM for Zed
build:
	cargo build --release --target wasm32-unknown-unknown

# Build native binary (for local testing)
build-native:
	cargo build --release

# Build LemMinX native binary using GraalVM (requires GraalVM in PATH)
native:
	@echo "Building LemMinX native binary..."
	@echo "This requires GraalVM with native-image installed."
	@TMPDIR=$$(mktemp -d) && \
	cd $$TMPDIR && \
	git clone --depth 1 --branch 0.11.0 https://github.com/eclipse-lemminx/lemminx && \
	cd lemminx && \
	./mvnw -Pnative -DskipTests package && \
	echo "Native binary built at: org.eclipse.lemminx/target/lemminx-*" && \
	ls -la org.eclipse.lemminx/target/lemminx-*

# Quality checks: format, clippy, test
check: fmt clippy test

# Format check
fmt:
	cargo fmt --all -- --check

# Clippy lints
clippy:
	cargo clippy --all-targets -- -D warnings

# Clean build artifacts
clean:
	cargo clean
	rm -rf target/wasm32-unknown-unknown

# Update Cargo.lock
update:
	cargo update

# Generate Cargo.lock with checksums for Nix
cargo-sha256:
	@echo "Run this after successful build to get cargoSha256 for flake.nix:"
	@echo "  nix-prefetch-github --rev <commit> impure0xntk zed-xml-language-server"
	@echo "Or use cargo2nix to generate proper derivations."