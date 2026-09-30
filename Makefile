# =============================================================================
# Quire Contract IR Makefile
#
# Native orchestration. Every target calls the toolchain that owns the job:
# cargo for the crate, the contract conformance runner for the domain corpus,
# and quire for specifications.
# =============================================================================

CARGO ?= cargo
QUIRE ?= quire

.PHONY: help
help:
	@echo "Available targets:"
	@echo "  make fmt              - Format with rustfmt"
	@echo "  make fmt-check        - Verify formatting (CI gate)"
	@echo "  make lint             - Clippy with -D warnings"
	@echo "  make corpus           - Run the native published conformance corpus"
	@echo "  make check-corpus     - Alias for corpus (ecosystem-compatible name)"
	@echo "  make spec             - Validate and cover all Quire artifacts"
	@echo "  make release-check    - Run every local release gate"
	@echo "  make test             - Run cargo test"
	@echo "  make build            - Release build"
	@echo "  make clean            - cargo clean"
	@echo "  make deny             - Run all cargo-deny policy checks"
	@echo "  make audit-unsafe     - Enforce // SAFETY: comments on unsafe blocks"
	@echo "  make ci               - All local release gates"

# =============================================================================
# Format / Lint / Test
# =============================================================================

.PHONY: fmt
fmt:
	$(CARGO) fmt --all

.PHONY: fmt-check
fmt-check:
	$(CARGO) fmt --all -- --check

.PHONY: lint
lint:
	$(CARGO) clippy --locked --workspace --all-targets -- -D warnings
	# The workspace lane turns on the model's test-only fault-injection feature
	# through the root dev-dependency; this lane checks the model as a consumer
	# builds it, with the feature off.
	$(CARGO) clippy --locked -p quire-contract-model -- -D warnings

.PHONY: corpus
corpus:
	$(CARGO) run --locked --quiet --bin quire-contract-conformance -- run --corpus corpus/contract-v0.1 --schemas schemas

.PHONY: check-corpus
check-corpus: corpus

.PHONY: spec
spec:
	$(QUIRE) validate --scope . 'spec/**/*.md' 'plan/**/*.md' 'reviews/**/*.md' --summary
	$(QUIRE) coverage --scope . --strict

.PHONY: test
test:
	$(CARGO) test --locked --workspace --all-targets -- --include-ignored
	# The model's own doctests with its test-only fault-injection feature off:
	# they prove a default build does not export that surface (FR-019-AC-4).
	$(CARGO) test --locked -p quire-contract-model --doc

.PHONY: build
build:
	$(CARGO) build --locked --workspace --release

.PHONY: clean
clean:
	$(CARGO) clean

# =============================================================================
# Supply chain & safety
# =============================================================================

.PHONY: deny
deny:
	$(CARGO) deny check

.PHONY: cargo-audit
cargo-audit:
	$(CARGO) audit

.PHONY: audit-unsafe
audit-unsafe:
	bash scripts/check_unsafe_comments.sh

# =============================================================================
# Composite
# =============================================================================

.PHONY: ci
ci: fmt-check lint test corpus spec deny cargo-audit audit-unsafe

.PHONY: release-check
release-check: ci
