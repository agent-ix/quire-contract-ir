# =============================================================================
# Quire Contract IR Makefile
#
# Native orchestration. Every target calls the toolchain that owns the job:
# cargo for the crate, the contract conformance runner for the domain corpus,
# quire for static export, quoin for evidence. Nothing here computes a verdict,
# attests to its own correctness, or retains evidence of its own.
# =============================================================================

CARGO ?= cargo
PYTHON ?= python3
QUIRE ?= quire
QUOIN ?= quoin

# The shared-assurance lane runs in its own interpreter. engineering-assurance
# declares jsonschema>=4.23 and the PGM-01 Draft 7 governance lane pins 3.2.0;
# both are right for their own job, so they get one environment each.
ASSURANCE_VENV ?= .venv-assurance
ASSURANCE_PYTHON ?= $(ASSURANCE_VENV)/bin/python

ASSURANCE_DIR := target/assurance
CONFORMANCE_RESULT := $(ASSURANCE_DIR)/conformance.jsonl
QUIRE_EXPORT := $(ASSURANCE_DIR)/quire-static-export.json
REVISION ?= $(shell git rev-parse HEAD)

.PHONY: help
help:
	@echo "Available targets:"
	@echo "  make fmt              - Format with rustfmt"
	@echo "  make fmt-check        - Verify formatting (CI gate)"
	@echo "  make lint             - Clippy with -D warnings"
	@echo "  make governance       - Validate PGM-01 schema and corpus"
	@echo "  make unit             - Run the Python test suite"
	@echo "  make corpus           - Run and census the published conformance corpus"
	@echo "  make check-corpus     - Alias for corpus (ecosystem-compatible name)"
	@echo "  make corpus-repro     - Regenerate the corpus in scratch space and compare bytes"
	@echo "  make spec             - Validate and cover all Quire artifacts"
	@echo "  make assurance-env    - Create the pinned shared-assurance interpreter"
	@echo "  make assurance-inputs - Run the native producers the shared path consumes"
	@echo "  make pins             - Classify the shared toolchain against the accepted matrix"
	@echo "  make compat-view      - Read immutable PGM-01 history through the shared mapping"
	@echo "  make assurance-chain  - Seal, retain, receipt, and re-verify through Quoin"
	@echo "  make assurance        - pins + compat-view + assurance-chain"
	@echo "  make assurance-record - Transcribe a conformance run into the Quoin evidence store"
	@echo "  make release-check    - Run every local release gate"
	@echo "  make test             - Validate governance and run cargo test"
	@echo "  make build            - Release build"
	@echo "  make msrv             - Check all targets with Rust 1.75"
	@echo "  make clean            - cargo clean and drop the assurance workspace"
	@echo "  make deny             - cargo deny check licenses"
	@echo "  make audit-unsafe     - Enforce // SAFETY: comments on unsafe blocks"
	@echo "  make ci               - All CI gates locally"

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

# The Python suite covers the whole tests/ tree, including the shared-assurance
# gates, and those read producer output. They consume it; they never produce it.
.PHONY: unit
unit: assurance-env assurance-inputs
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

.PHONY: test
test: governance unit
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
	rm -rf $(ASSURANCE_VENV)

# =============================================================================
# Shared assurance
#
# The producers run here. Quire exports static facts and Quoin transcribes,
# retains, and audits what it is handed; neither of them invokes anything.
# =============================================================================

$(ASSURANCE_PYTHON):
	$(PYTHON) -m venv $(ASSURANCE_VENV)
	$(ASSURANCE_VENV)/bin/pip install --quiet --disable-pip-version-check -r requirements-assurance.txt

.PHONY: assurance-env
assurance-env: $(ASSURANCE_PYTHON)

.PHONY: assurance-inputs
assurance-inputs:
	mkdir -p $(ASSURANCE_DIR)
	$(CARGO) run --quiet --bin quire-contract-conformance -- run --manifest corpus/contract-v0.1/manifest.json > $(CONFORMANCE_RESULT)
	$(QUIRE) coverage --scope . --json > $(QUIRE_EXPORT)

.PHONY: pins
pins: assurance-env
	$(ASSURANCE_PYTHON) scripts/check_shared_pins.py

.PHONY: compat-view
compat-view: assurance-env
	$(ASSURANCE_PYTHON) scripts/pgm01_compatibility_view.py
	$(ASSURANCE_PYTHON) scripts/pgm01_compatibility_view.py --mutation-probes

.PHONY: assurance-chain
assurance-chain: assurance-inputs
	$(PYTHON) scripts/assurance_chain.py \
		--candidate-revision $(REVISION) \
		--conformance $(CONFORMANCE_RESULT) \
		--quire-export $(QUIRE_EXPORT)

.PHONY: assurance
assurance: pins compat-view assurance-chain

# Operator target, and the only one that writes outside target/. It transcribes
# a conformance run into the Quoin evidence store under spec/evidence/, keyed by
# the commit it ran at.
#
# That output is deliberately not committed. A record naming a revision, stored
# in a commit that changes the revision, is stale the moment it lands — which is
# exactly the failure that left the deleted verifier red on main. Where the
# store is retained is a deployment decision; a run recorded here is retained by
# whatever runs it. What every gate proves on every invocation is the path
# itself, in `make assurance-chain`.
#
# Note the honest `bound: 0`: this repository declares no suite registry, so the
# run is transcribed and binds no obligation. Transcription working and binding
# nothing are different facts, and quoin reports them separately.
.PHONY: assurance-record
assurance-record: assurance-inputs
	$(QUOIN) evidence record \
		--repo . \
		--suite SUITE-001 \
		--commit $(REVISION) \
		--tool "quire-contract-conformance $(shell $(CARGO) run --quiet --bin quire-contract-conformance -- --version | cut -d' ' -f2)" \
		--adapter contract-conformance \
		--kind Conformance \
		--results $(CONFORMANCE_RESULT)

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
ci: fmt-check lint test corpus corpus-repro deny audit-unsafe assurance

.PHONY: release-check
release-check: ci spec
