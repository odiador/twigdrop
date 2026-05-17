.PHONY: setup dev build format lint check install release help

# Default target
all: help

help:
	@echo "🧹 Twigdrop Makefile"
	@echo ""
	@echo "Usage:"
	@echo "  make setup    - Verifies dependencies (cargo, gh)"
	@echo "  make dev      - Runs the application (Pass path as: make dev ~/.gh/repo)"
	@echo "  make build    - Builds the release binary"
	@echo "  make format   - Formats the Rust code"
	@echo "  make lint     - Runs Clippy to catch common mistakes"
	@echo "  make check    - Checks compilation without generating a binary"
	@echo "  make install  - Installs the binary locally via Cargo"
	@echo "  make release VERSION=v0.1.0 - Creates a GitHub release and tag using the gh cli"

setup:
	@echo "Checking Rust installation..."
	@cargo --version || (echo "Cargo not found. Please install Rust."; exit 1)
	@echo "Checking GitHub CLI installation..."
	@gh --version || (echo "GitHub CLI (gh) not found. Please install it."; exit 1)
	@echo "All dependencies look good!"

# Support for passing arguments to 'make dev'
# Usage: make dev /path/to/repo
ifeq (dev,$(firstword $(MAKECMDGOALS)))
  DEV_ARGS := $(wordlist 2,$(words $(MAKECMDGOALS)),$(MAKECMDGOALS))
  $(eval $(DEV_ARGS):;@:)
endif

dev:
	cargo run -- $(DEV_ARGS)

build:
	cargo build --release

format:
	cargo fmt

lint:
	cargo clippy -- -D warnings

check:
	cargo check

install: build
	cargo install --path .

# Creates a tag and release on GitHub
# Usage: make release VERSION=v0.1.0
release:
	@if [ -z "$(VERSION)" ]; then \
		echo "Error: VERSION is not set. Use 'make release VERSION=vX.Y.Z'"; \
		exit 1; \
	fi
	@echo "Creating tag and GitHub release for $(VERSION)..."
	gh release create $(VERSION) --generate-notes --title "Release $(VERSION)"
	@echo "Release $(VERSION) created successfully!"
