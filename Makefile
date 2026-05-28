# =============================================================================
# Makefile — Developer tooling for kiddoo-platform
# =============================================================================

.PHONY: setup hooks fmt lint test build check clean

# -----------------------------------------------------------------------------
# Setup: Install git hooks and configure project for development
# -----------------------------------------------------------------------------
setup: hooks
	@echo "✓ Development environment configured!"
	@echo "  Git hooks installed from .githooks/"
	@echo ""
	@echo "  Recommended: install gitleaks for secret scanning"
	@echo "    https://github.com/gitleaks/gitleaks#installing"

# -----------------------------------------------------------------------------
# Hooks: Configure git to use project hooks
# -----------------------------------------------------------------------------
hooks:
	@echo "Installing git hooks..."
	@git config core.hooksPath .githooks
	@chmod +x .githooks/*
	@echo "✓ Git hooks installed (pre-commit, commit-msg, pre-push)"

# -----------------------------------------------------------------------------
# Format: Auto-fix formatting
# -----------------------------------------------------------------------------
fmt:
	cargo fmt --all

# -----------------------------------------------------------------------------
# Lint: Run clippy
# -----------------------------------------------------------------------------
lint:
	cargo clippy --workspace -- -D warnings

# -----------------------------------------------------------------------------
# Test: Run all tests
# -----------------------------------------------------------------------------
test:
	cargo test --workspace --verbose

# -----------------------------------------------------------------------------
# Build: Full release build
# -----------------------------------------------------------------------------
build:
	cargo build --workspace --release

# -----------------------------------------------------------------------------
# Check: Run all checks (same as pre-commit + pre-push combined)
# -----------------------------------------------------------------------------
check: fmt lint test build
	@echo ""
	@echo "✓ All checks passed — safe to commit and push!"

# -----------------------------------------------------------------------------
# Clean: Remove build artifacts
# -----------------------------------------------------------------------------
clean:
	cargo clean

# -----------------------------------------------------------------------------
# Help
# -----------------------------------------------------------------------------
help:
	@echo "Available targets:"
	@echo "  make setup   — Install git hooks and configure dev environment"
	@echo "  make hooks   — Install git hooks only"
	@echo "  make fmt     — Auto-format code"
	@echo "  make lint    — Run clippy linter"
	@echo "  make test    — Run all tests"
	@echo "  make build   — Full release build"
	@echo "  make check   — Run ALL checks (fmt + lint + test + build)"
	@echo "  make clean   — Remove build artifacts"
