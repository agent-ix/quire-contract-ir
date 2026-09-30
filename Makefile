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
	@echo "  make use-local        - Patch first-party git deps to sibling checkouts (.cargo/config.toml)"
	@echo "  make use-remote       - Remove the local patch file; build from GitHub"
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

# First-party crates that must appear once in Cargo.lock. cargo-deny's
# `deny-multiple-versions` (deny.toml) only sees copies whose versions differ;
# every first-party crate is 0.1.0-ish, so a second git spec of the same
# version is invisible to it. The lockfile count below catches that case.
FIRST_PARTY ?= quire-contract-model quire-contract-ir quire-verification-contracts ix-trace-rs quire-canonical quire-spec-language quire-exact qsl-attrs qsl-bench qsl-cst qsl-eval qsl-forms qsl-foundation qsl-package qsl-replay qsl-route qsl-semantics qsl-source

.PHONY: deny
deny:
	$(CARGO) deny check
	@set -e; for c in $(FIRST_PARTY); do \
	  n=$$(grep -c "^name = \"$$c\"$$" Cargo.lock || true); \
	  if [ "$$n" -gt 1 ]; then echo "deny: $$c appears $$n times in Cargo.lock" >&2; exit 1; fi; \
	done

.PHONY: cargo-audit
cargo-audit:
	$(CARGO) audit

.PHONY: audit-unsafe
audit-unsafe:
	bash scripts/check_unsafe_comments.sh

# =============================================================================
# Local development against sibling checkouts
#
# `use-local` writes a gitignored .cargo/config.toml that patches each
# first-party git dependency to its working tree at $(SIBLINGS)/<repo>, uncommitted
# edits included. `use-remote` deletes it. Format: <repo>:<crate-dir>:<crate>.
# SIBLINGS is the directory holding the sibling clones: the parent of the main
# checkout, so it is also right from a linked worktree. Override to relocate.
# =============================================================================

SIBLINGS ?= $(abspath $(shell git rev-parse --path-format=absolute --git-common-dir)/../..)
LOCAL_PATCHES ?= quire-verification-contracts:.:quire-verification-contracts ix-trace-rs:.:ix-trace-rs

.PHONY: use-local
use-local:
	@set -e; mkdir -p .cargo; : > .cargo/config.toml; \
	for spec in $(LOCAL_PATCHES); do \
	  repo=$${spec%%:*}; rest=$${spec#*:}; dir=$${rest%%:*}; crate=$${rest#*:}; \
	  if [ ! -f "$(SIBLINGS)/$$repo/$$dir/Cargo.toml" ]; then \
	    rm -f .cargo/config.toml; \
	    echo "use-local: $(SIBLINGS)/$$repo is not cloned (no Cargo.toml at $(SIBLINGS)/$$repo/$$dir); clone agent-ix/$$repo next to this repo" >&2; exit 1; \
	  fi; \
	  printf '[patch."https://github.com/agent-ix/%s"]\n%s = { path = "%s/%s/%s" }\n\n' "$$repo" "$$crate" "$(SIBLINGS)" "$$repo" "$$dir" >> .cargo/config.toml; \
	done; echo "wrote .cargo/config.toml"

.PHONY: use-remote
use-remote:
	rm -f .cargo/config.toml

# =============================================================================
# Composite
# =============================================================================

.PHONY: ci
ci: fmt-check lint test corpus spec deny cargo-audit audit-unsafe

.PHONY: release-check
release-check: ci
