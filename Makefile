# =============================================================================
# Makefile — Developer tooling for kiddoo-platform
# =============================================================================

.PHONY: setup hooks fmt lint test build check clean env up down logs rebuild ps

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
# Env: Switch .env to a specific environment
# Usage: make env ENV=development|test|staging|production
# -----------------------------------------------------------------------------
ENV ?= development

env:
	@if [ ! -f .env.$(ENV) ]; then \
		echo "✗ File .env.$(ENV) does not exist."; \
		echo "  Available: development, test, staging, production"; \
		exit 1; \
	fi
	@cp .env.$(ENV) .env
	@echo "✓ Switched to $(ENV) environment (.env.$(ENV) → .env)"

# -----------------------------------------------------------------------------
# Docker: Run all services via Docker Compose
# -----------------------------------------------------------------------------
up:
	@if [ ! -f .env ]; then \
		echo "No .env found, copying .env.example..."; \
		cp .env.example .env; \
	fi
	docker compose up -d
	@echo ""
	@echo "✓ Services starting..."
	@echo "  API Gateway:    http://localhost:8000"
	@echo "  Swagger UI:     http://localhost:8000/swagger-ui/"
	@echo "  Identity Proxy: http://localhost:8001"
	@echo "  PostgreSQL:     localhost:5433"
	@echo ""
	@echo "  Logs: make logs"

down:
	docker compose down

down-v:
	docker compose down -v
	@echo "✓ Stopped and removed volumes"

rebuild:
	docker compose up -d --build

ps:
	docker compose ps

logs:
	docker compose logs -f --tail=50

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
	@echo ""
	@echo "  Setup & Config:"
	@echo "    make setup        — Install git hooks and configure dev environment"
	@echo "    make hooks        — Install git hooks only"
	@echo "    make env ENV=xxx  — Switch .env (development|test|staging|production)"
	@echo ""
	@echo "  Rust Development:"
	@echo "    make fmt          — Auto-format code"
	@echo "    make lint         — Run clippy linter"
	@echo "    make test         — Run all tests"
	@echo "    make build        — Full release build"
	@echo "    make check        — Run ALL checks (fmt + lint + test + build)"
	@echo "    make clean        — Remove build artifacts"
	@echo ""
	@echo "  Docker:"
	@echo "    make up           — Start all services (postgres + API)"
	@echo "    make down         — Stop all services"
	@echo "    make down-v       — Stop and remove volumes (reset DB)"
	@echo "    make rebuild      — Rebuild images and restart"
	@echo "    make ps           — Show running containers"
	@echo "    make logs         — Follow service logs"
