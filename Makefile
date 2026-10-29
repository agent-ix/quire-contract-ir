# =============================================================================
# Quire Contract IR Makefile
#
# Native orchestration. Every target calls the toolchain that owns the job:
# cargo for the crate, the contract conformance runner for the domain corpus,
# and quire for specifications.
# =============================================================================

CARGO ?= cargo
# --locked only when no local patch is active: a patch rewrites the resolution.
LOCKED ?= $(if $(wildcard .cargo/config.toml),,--locked)
QUIRE ?= quire
STRING_EDGE ?= qsl-string-edge
QUIRE_SPEC_LANGUAGE_BIN ?= $(QUIRE_SPEC_LANGUAGE_DIR)/target/debug/quire-spec

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
	@echo "  make conformance-qspec-compile - Compile the QSpec conformance target without fixtures (CI gate)"
	@echo "  make conformance-qspec - QSpec's positive fixtures admit, adverse mutations refuse as recorded, selection identity derives (needs QUIRE_SPECIFICATION_DIR; not in ci)"
	@echo "  make conformance-qsl-ir703 - Fresh ConfigVersion V2 producer/IR accessor check (needs QUIRE_SPEC_LANGUAGE_DIR and its current binaries)"
	@echo "  make conformance-qsl-ir769 - Fresh QSL scalar result context through admitted IR (needs QUIRE_SPEC_LANGUAGE_DIR and its current binary)"
	@echo "  make conformance-qsl-ir779 - QSL FR-062 RR-13 path conditions through admitted IR (needs QUIRE_SPEC_LANGUAGE_DIR and its current binary)"
	@echo "  make conformance-qsl-ir760 - QSL-emitted Population<T>[N] through the default-feature model consumer (needs QUIRE_SPEC_LANGUAGE_DIR)"
	@echo "  make build            - Release build"
	@echo "  make clean            - cargo clean"
	@echo "  make deny             - Run all cargo-deny policy checks"
	@echo "  make use-local        - Patch first-party git deps to sibling checkouts (.cargo/config.toml)"
	@echo "  make use-remote       - Remove the local patch file and restore Cargo.lock; build from GitHub"
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

.PHONY: string-edge
string-edge:
	$(STRING_EDGE) --base-dir . --crate-root src --crate-root crates/quire-contract-model/src --rules string-edge-rules.json

.PHONY: lint
lint: string-edge
	$(CARGO) clippy $(LOCKED) --workspace --all-targets --features conformance-cli -- -D warnings
	# The workspace lane turns on the model's test-only fault-injection feature
	# through the root dev-dependency; this lane checks the model as a consumer
	# builds it, with the feature off.
	$(CARGO) clippy $(LOCKED) -p quire-contract-model -- -D warnings

.PHONY: corpus
corpus:
	$(CARGO) run $(LOCKED) --quiet --features conformance-cli --bin quire-contract-conformance -- run --corpus corpus/contract-v0.1 --schemas schemas

.PHONY: check-corpus
check-corpus: corpus

.PHONY: spec
spec:
	$(QUIRE) validate --scope . 'spec/**/*.md' 'plan/**/*.md' 'reviews/**/*.md' --summary
	$(QUIRE) coverage --scope . --strict

.PHONY: test
test:
	$(CARGO) test $(LOCKED) --workspace --all-targets --features conformance-cli -- --include-ignored
	# The model's own doctests with its test-only fault-injection feature off:
	# they prove a default build does not export that surface (FR-019-AC-4).
	$(CARGO) test $(LOCKED) -p quire-contract-model --doc
	$(CARGO) test $(LOCKED) -p quire-contract-model --test source_obligation_public
	# The root crate's doctests, which `--all-targets` leaves out: the
	# `compile_fail` probes of the non-success constructor (FR-030-AC-6).
	$(CARGO) test $(LOCKED) -p quire-contract-ir --doc

# FR-038-AC-107/176: QSpec's CheckedPackage V2 fixtures and recorded model
# declaration keys agree with the production reader and key derivation. Reads the
# checkout named by QUIRE_SPECIFICATION_DIR and fails, never skips, when it is
# unset, empty, or holds no proposals/checked-package-v2/fixtures/, or when a
# fixture does not admit. Fixture execution stays outside `test` and `ci`:
# they have no checkout.
.PHONY: conformance-qspec-compile
conformance-qspec-compile:
	$(CARGO) test $(LOCKED) --workspace --no-run --test conformance_qspec

.PHONY: conformance-qspec
conformance-qspec:
	$(CARGO) test $(LOCKED) --workspace --lib ir1083_
	$(CARGO) test $(LOCKED) --test conformance_qspec
	@output=$$(QSPEC_CONFORMANCE=1 $(CARGO) test $(LOCKED) --workspace --lib checked_package::v2::model_members::tests::tc_048_qspec_model_declaration_keys_use_production_derivation -- --exact 2>&1); \
	status=$$?; printf '%s\n' "$$output"; \
	[ "$$status" -eq 0 ] || exit "$$status"; \
	passed=$$(printf '%s\n' "$$output" | grep -c '^test result: ok\. 1 passed; 0 failed; 0 ignored; 0 measured;' || true); \
	[ "$$passed" -eq 1 ] || { echo 'required QSpec production-key test did not run exactly once' >&2; exit 1; }
	@output=$$(QSPEC_CONFORMANCE=1 $(CARGO) test $(LOCKED) --test it checked_package_v2_owners::tc_229_qspec_ 2>&1); \
	status=$$?; printf '%s\n' "$$output"; \
	[ "$$status" -eq 0 ] || exit "$$status"; \
	passed=$$(printf '%s\n' "$$output" | grep -c '^test result: ok\. 2 passed; 0 failed; 0 ignored; 0 measured;' || true); \
	[ "$$passed" -eq 1 ] || { echo 'required QSpec population fixture tests did not run exactly once' >&2; exit 1; }

# TC-445's producer-origin case is explicit: the QSL checkout supplies its own
# current fixture and compiler executables, and the test records the revision
# and commands. Ordinary IR tests have no external QSL build dependency.
.PHONY: conformance-qsl-ir703
conformance-qsl-ir703:
	@test -n "$(QUIRE_SPEC_LANGUAGE_DIR)" || { echo 'QUIRE_SPEC_LANGUAGE_DIR is required' >&2; exit 1; }
	@test -x "$(QUIRE_SPEC_LANGUAGE_DIR)/target/debug/examples/config_version_fixtures" || { echo 'build the current QSL config_version_fixtures example first' >&2; exit 1; }
	@test -x "$(QUIRE_SPEC_LANGUAGE_DIR)/target/debug/quire-spec" || { echo 'build the current QSL quire-spec binary first' >&2; exit 1; }
	@output=$$(IR703_QSL_CONFORMANCE=1 $(CARGO) test $(LOCKED) --workspace --test it checked_package_v2_comparison_operands::tc_445_fresh_qsl_config_version_has_shared_field_with_post_and_pre_observations -- --exact --nocapture 2>&1); \
	status=$$?; printf '%s\n' "$$output"; \
	[ "$$status" -eq 0 ] || exit "$$status"; \
	passed=$$(printf '%s\n' "$$output" | grep -c '^test result: ok\. 1 passed; 0 failed; 0 ignored; 0 measured;' || true); \
	[ "$$passed" -eq 1 ] || { echo 'required TC-445 QSL producer test did not run exactly once' >&2; exit 1; }

.PHONY: conformance-qsl-ir769
conformance-qsl-ir769:
	@test -n "$(QUIRE_SPEC_LANGUAGE_DIR)" || { echo 'QUIRE_SPEC_LANGUAGE_DIR is required' >&2; exit 1; }
	@test -x "$(QUIRE_SPEC_LANGUAGE_BIN)" || { echo 'build the current QSL quire-spec binary first' >&2; exit 1; }
	@output=$$(QUIRE_SPEC_LANGUAGE_BIN="$(QUIRE_SPEC_LANGUAGE_BIN)" $(CARGO) test $(LOCKED) --test conformance_qsl_ir769 -- --nocapture 2>&1); \
	status=$$?; printf '%s\n' "$$output"; \
	[ "$$status" -eq 0 ] || exit "$$status"; \
	passed=$$(printf '%s\n' "$$output" | grep -c '^test result: ok\. 1 passed; 0 failed; 0 ignored; 0 measured;' || true); \
	[ "$$passed" -eq 1 ] || { echo 'required TC-447 QSL producer test did not run exactly once' >&2; exit 1; }

.PHONY: conformance-qsl-ir779
conformance-qsl-ir779:
	@test -n "$(QUIRE_SPEC_LANGUAGE_DIR)" || { echo 'QUIRE_SPEC_LANGUAGE_DIR is required' >&2; exit 1; }
	@test -x "$(QUIRE_SPEC_LANGUAGE_BIN)" || { echo 'build the current QSL quire-spec binary first' >&2; exit 1; }
	@output=$$(QUIRE_SPEC_LANGUAGE_DIR="$(QUIRE_SPEC_LANGUAGE_DIR)" QUIRE_SPEC_LANGUAGE_BIN="$(QUIRE_SPEC_LANGUAGE_BIN)" QUIRE_SPEC_LANGUAGE_TARGET_DIR="$(QUIRE_SPEC_LANGUAGE_TARGET_DIR)" $(CARGO) test $(LOCKED) --test conformance_qsl_ir779 -- --nocapture 2>&1); \
	status=$$?; printf '%s\n' "$$output"; \
	[ "$$status" -eq 0 ] || exit "$$status"; \
	passed=$$(printf '%s\n' "$$output" | grep -c '^test result: ok\. 1 passed; 0 failed; 0 ignored; 0 measured;' || true); \
	[ "$$passed" -eq 1 ] || { echo 'required TC-450 QSL producer test did not run exactly once' >&2; exit 1; }

.PHONY: conformance-qsl-ir760
conformance-qsl-ir760:
	@test -n "$(QUIRE_SPEC_LANGUAGE_DIR)" || { echo 'QUIRE_SPEC_LANGUAGE_DIR is required' >&2; exit 1; }
	@QUIRE_SPEC_LANGUAGE_DIR="$(QUIRE_SPEC_LANGUAGE_DIR)" QUIRE_SPEC_LANGUAGE_TARGET_DIR="$(CURDIR)/target/qsl-producer" CARGO_TARGET_DIR="$(CURDIR)/target" $(CARGO) test $(LOCKED) --manifest-path crates/quire-contract-model/Cargo.toml --test selected_population_public -- --nocapture
	@QUIRE_SPEC_LANGUAGE_DIR="$(QUIRE_SPEC_LANGUAGE_DIR)" QUIRE_SPEC_LANGUAGE_TARGET_DIR="$(CURDIR)/target/qsl-producer" CARGO_TARGET_DIR="$(CURDIR)/target" $(CARGO) test $(LOCKED) --test conformance_qsl_ir760 -- --nocapture

.PHONY: build
build:
	$(CARGO) build $(LOCKED) --workspace --release
	$(CARGO) build $(LOCKED) -p quire-contract-ir --release --features conformance-cli --bin quire-contract-conformance

.PHONY: clean
clean:
	$(CARGO) clean

# =============================================================================
# Supply chain & safety
# =============================================================================

# The Quire CLI owns first-party Git lock coherence (FR-055).
.PHONY: deny
deny:
	$(CARGO) deny check
	$(QUIRE) check-lock Cargo.lock

.PHONY: cargo-audit
cargo-audit:
	$(CARGO) audit

# =============================================================================
# Local development against sibling checkouts
#
# `use-local` writes a gitignored .cargo/config.toml that patches each
# first-party git dependency to its working tree at $(SIBLINGS)/<repo>, uncommitted
# edits included. `use-local` first snapshots Cargo.lock to the gitignored
# .cargo/Cargo.lock.pre-local; `use-remote` deletes the config and restores the
# lock from that snapshot (and does nothing to the lock if there is none). So the
# lock returns to its state before the first `use-local`; lock changes made while
# a patch is active are discarded, and a `cargo update -p` made without a patch is kept.
# Format: <repo>:<crate>:<crate-dir>; entries are grouped by repo here, in any
# order, so each repo gets exactly one [patch] table.
# SIBLINGS is the directory holding the sibling clones: the parent of the main
# checkout, so it is also right from a linked worktree. Override to relocate.
# =============================================================================

SIBLINGS ?= $(abspath $(shell git rev-parse --path-format=absolute --git-common-dir)/../..)
LOCAL_PATCHES ?= quire-verification-contracts:quire-verification-contracts:. ix-trace-rs:ix-trace-rs:. quire-canonical:quire-canonical:. quire-canonical:quire-canonical-derive:quire-canonical-derive quire-walk:quire-walk:.

.PHONY: use-local
use-local:
	@set -e; mkdir -p .cargo; \
	for spec in $(LOCAL_PATCHES); do \
	  if [ "$$(printf '%s' "$$spec" | tr -cd ':' | wc -c)" != 2 ] || printf '%s' "$$spec" | grep -q '::\|^:\|:$$'; then \
	    echo "use-local: malformed LOCAL_PATCHES entry '$$spec' (want repo:crate:dir)" >&2; exit 1; \
	  fi; \
	  repo=$${spec%%:*}; rest=$${spec#*:}; dir=$${rest#*:}; \
	  if [ ! -f "$(SIBLINGS)/$$repo/$$dir/Cargo.toml" ]; then \
	    echo "use-local: $(SIBLINGS)/$$repo is not cloned (no Cargo.toml at $(SIBLINGS)/$$repo/$$dir); clone agent-ix/$$repo next to this repo" >&2; exit 1; \
	  fi; \
	done; \
	[ -f .cargo/Cargo.lock.pre-local ] || cp Cargo.lock .cargo/Cargo.lock.pre-local; \
	: > .cargo/config.toml; \
	repos=$$(for spec in $(LOCAL_PATCHES); do printf '%s\n' "$${spec%%:*}"; done | awk '!seen[$$0]++'); \
	first=1; \
	for repo in $$repos; do \
	  [ "$$first" = 1 ] || printf '\n' >> .cargo/config.toml; first=0; \
	  printf '[patch."https://github.com/agent-ix/%s"]\n' "$$repo" >> .cargo/config.toml; \
	  for spec in $(LOCAL_PATCHES); do \
	    [ "$${spec%%:*}" = "$$repo" ] || continue; \
	    rest=$${spec#*:}; crate=$${rest%%:*}; dir=$${rest#*:}; \
	    printf '%s = { path = "%s/%s/%s" }\n' "$$crate" "$(SIBLINGS)" "$$repo" "$$dir" >> .cargo/config.toml; \
	  done; \
	done; echo "wrote .cargo/config.toml"; \
	meta=$$(mktemp); \
	if ! $(CARGO) metadata --format-version 1 >/dev/null 2>"$$meta"; then \
	  cat "$$meta" >&2; rm -f "$$meta" .cargo/config.toml; \
	  [ ! -f .cargo/Cargo.lock.pre-local ] || { cp .cargo/Cargo.lock.pre-local Cargo.lock; rm -f .cargo/Cargo.lock.pre-local; }; \
	  echo "use-local: cargo metadata failed under the patch" >&2; exit 1; \
	fi; \
	if grep -q 'patch .* was not used' "$$meta"; then \
	  cat "$$meta" >&2; rm -f "$$meta" .cargo/config.toml; \
	  [ ! -f .cargo/Cargo.lock.pre-local ] || { cp .cargo/Cargo.lock.pre-local Cargo.lock; rm -f .cargo/Cargo.lock.pre-local; }; \
	  echo "use-local: a patch was not used; the sibling's version does not satisfy the requirement" >&2; exit 1; \
	fi; \
	rm -f "$$meta"

.PHONY: use-remote
use-remote:
	rm -f .cargo/config.toml
	@if [ -f .cargo/Cargo.lock.pre-local ]; then mv .cargo/Cargo.lock.pre-local Cargo.lock; echo "restored Cargo.lock from the pre-local snapshot"; fi

# =============================================================================
# Composite
# =============================================================================

.PHONY: ci
ci: fmt-check lint test conformance-qspec-compile corpus spec deny cargo-audit

.PHONY: release-check
release-check: ci

# TC-041 / FR-028-AC-4: explicit native production configurations. Reuses the
# worktree target; this named lane also runs the build control by itself.
.PHONY: model-boundary-builds
model-boundary-builds:
	$(CARGO) test $(LOCKED) --workspace --test it cycle_free_model::production_default_all_feature_and_msrv_builds -- --ignored --exact --nocapture
