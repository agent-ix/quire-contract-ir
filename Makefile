# =============================================================================
# Quire Contract IR Makefile
# =============================================================================

CARGO ?= cargo
PYTHON ?= python3
QUIRE ?= quire

.PHONY: help
help:
	@echo "Available targets:"
	@echo "  make fmt              - Format with rustfmt"
	@echo "  make fmt-check        - Verify formatting (CI gate)"
	@echo "  make lint             - Clippy with -D warnings"
	@echo "  make governance       - Validate PGM-01 schema and corpus"
	@echo "  make corpus          - Run and census the published conformance corpus"
	@echo "  make check-corpus    - Alias for corpus (ecosystem-compatible name)"
	@echo "  make corpus-repro    - Regenerate the corpus in scratch space and compare bytes"
	@echo "  make spec             - Validate and cover all Quire artifacts"
	@echo "  make evidence-verify  - Verify one immutable evidence record"
	@echo "  make verify-evidence  - Alias for evidence-verify"
	@echo "  make release-check    - Run every local release gate, including evidence"
	@echo "  make test             - Validate governance and run cargo test"
	@echo "  make build            - Release build"
	@echo "  make msrv             - Check all targets with Rust 1.75"
	@echo "  make clean            - cargo clean"
	@echo "  make deny             - cargo deny check licenses"
	@echo "  make audit-unsafe     - Enforce // SAFETY: comments on unsafe blocks"
	@echo "  make ci               - All CI gates locally (fmt-check + lint + test + deny + audit-unsafe)"

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
	$(CARGO) clippy --all-targets -- -D warnings

.PHONY: governance
governance:
	$(PYTHON) scripts/validate_governance.py --check-runtime
	$(PYTHON) scripts/validate_governance.py
	$(PYTHON) scripts/validate_governance.py --mutation-probes
	$(PYTHON) -m unittest discover -s tests -p '*.py'

.PHONY: corpus
corpus:
	$(CARGO) run --quiet --bin quire-contract-conformance -- run --manifest corpus/contract-v0.1/manifest.json | $(PYTHON) -c 'import json, sys; manifest = json.load(open("corpus/contract-v0.1/manifest.json", encoding="utf-8")); rows = [json.loads(line) for line in sys.stdin]; assert [row["fixture_id"] for row in rows] == [fixture["id"] for fixture in manifest["fixtures"]]; assert all(row["status"] == "match" for row in rows)'

.PHONY: check-corpus
check-corpus: corpus

.PHONY: corpus-repro
corpus-repro:
	$(CARGO) build --quiet --bin quire-contract-conformance
	$(PYTHON) scripts/generate_conformance_corpus.py --check

.PHONY: spec
spec:
	$(QUIRE) validate --scope . 'spec/**/*.md' 'plan/**/*.md' 'reviews/**/*.md' --summary
	$(QUIRE) coverage --scope . --strict
	$(PYTHON) scripts/validate_matrix_status.py

.PHONY: evidence-verify
evidence-verify:
	$(PYTHON) scripts/verify_evidence.py

.PHONY: verify-evidence
verify-evidence: evidence-verify

.PHONY: test
test: governance
	QUIRE_GOVERNANCE_PYTHON=$(PYTHON) $(CARGO) test -- --include-ignored

.PHONY: build
build:
	$(CARGO) build --release

.PHONY: msrv
msrv:
	rustup run 1.75.0 $(CARGO) check --locked --all-targets

.PHONY: clean
clean:
	$(CARGO) clean

# =============================================================================
# Supply chain & safety
# =============================================================================

.PHONY: deny
deny:
	$(CARGO) deny check licenses

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
ci: fmt-check lint test corpus corpus-repro deny audit-unsafe

.PHONY: release-check
release-check: ci spec evidence-verify
