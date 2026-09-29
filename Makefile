# =============================================================================
# Quire Contract IR Makefile
#
# Native orchestration. Every target calls the toolchain that owns the job:
# cargo for the crate, the contract conformance runner for the domain corpus,
# and quire for specifications.
# =============================================================================

CARGO ?= cargo
PYTHON ?= python3
QUIRE ?= quire
RUSTUP ?= rustup

SUPPORTED_RUST_MINIMUM := 1.98.1
QUALIFICATION_RUST := 1.98.1

.PHONY: help
help:
	@echo "Available targets:"
	@echo "  make fmt              - Format with rustfmt"
	@echo "  make fmt-check        - Verify formatting (CI gate)"
	@echo "  make lint             - Clippy with -D warnings"
	@echo "  make unit             - Run the Python test suite"
	@echo "  make corpus           - Run the native published conformance corpus"
	@echo "  make check-corpus     - Alias for corpus (ecosystem-compatible name)"
	@echo "  make corpus-repro     - Regenerate the corpus in scratch space and compare bytes"
	@echo "  make spec             - Validate and cover all Quire artifacts"
	@echo "  make release-check    - Run every local release gate"
	@echo "  make test             - Run the Python suite and cargo test"
	@echo "  make build            - Release build"
	@echo "  make supported-rust   - Check all targets with the exact supported minimum"
	@echo "  make qualification-rust - Test all targets with the exact qualification compiler"
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

.PHONY: unit
unit:
	$(PYTHON) -m unittest discover -s tests -p '*.py'

.PHONY: corpus
corpus:
	$(CARGO) run --locked --quiet --bin quire-contract-conformance -- run --corpus corpus/contract-v0.1

.PHONY: check-corpus
check-corpus: corpus

.PHONY: corpus-repro
corpus-repro:
	$(CARGO) build --locked --quiet --bin quire-contract-conformance
	$(PYTHON) scripts/generate_conformance_corpus.py --check

.PHONY: spec
spec:
	$(QUIRE) validate --scope . 'spec/**/*.md' 'plan/**/*.md' 'reviews/**/*.md' --summary
	$(QUIRE) coverage --scope . --strict
	$(PYTHON) scripts/validate_matrix_status.py

.PHONY: test
test: unit
	$(CARGO) test --locked --workspace --all-targets -- --include-ignored
	# The model's own doctests with its test-only fault-injection feature off:
	# they prove a default build does not export that surface (FR-019-AC-4).
	$(CARGO) test --locked -p quire-contract-model --doc

.PHONY: build
build:
	$(CARGO) build --locked --workspace --release

.PHONY: supported-rust
supported-rust:
	$(RUSTUP) run $(SUPPORTED_RUST_MINIMUM) $(CARGO) check --locked --workspace --all-targets

.PHONY: qualification-rust
qualification-rust:
	$(RUSTUP) run $(QUALIFICATION_RUST) $(CARGO) test --locked --workspace --all-targets

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
ci: fmt-check lint test corpus corpus-repro spec supported-rust qualification-rust deny cargo-audit audit-unsafe

.PHONY: release-check
release-check: ci
