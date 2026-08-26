CARGO ?= cargo

.DEFAULT_GOAL := help

.PHONY: help
help: ## show available targets
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "\033[36m%-16s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

.PHONY: run build
run: ## run the terminal application
	$(CARGO) run

build: ## build all workspace targets
	$(CARGO) build --workspace --all-targets

.PHONY: fmt fmt-check lint test
fmt: ## format Rust source files
	$(CARGO) fmt --all

fmt-check: ## verify Rust formatting
	$(CARGO) fmt --check

lint: ## run Clippy with warnings denied
	$(CARGO) clippy --all-targets --all-features -- -D warnings

test: ## run all workspace tests
	$(CARGO) test --workspace

.PHONY: check ci
check: fmt-check lint test ## run all local quality checks

ci: check build ## run the deterministic CI checks
