---
id: SR-028
title: "Code review — shared assurance migration"
type: SpecReview
analysis: code-review
scope: "scripts/check_shared_pins.py; scripts/pgm01_compatibility_view.py; scripts/assurance_chain.py; tests/test_shared_assurance.py; tests/governance_reconciliation.rs; assurance/; Makefile; FR-022; revised FR-009; PGM-01-R09"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/PLAN-004
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/FR-022
    type: references
  - target: ix://agent-ix/quire-contract-ir/issues/39
    type: references
---

# SR-028: Code review — shared assurance migration

## Summary

Reviewed the whole issue #39 diff against `origin/main` at `8e0953e`: three new
scripts, one new Python test module, the revised FR-009/FR-022/PGM-01-R09
specification, the Makefile, and the deletion of the repository-local evidence
verifier. Rust changes are confined to one added assertion in
`tests/governance_reconciliation.rs`, so the Rust lane is scoped to that file and
to the gates, which were run rather than assumed. Eight findings: six fixed
during the review, including both `high`s, and two accepted with rationale.

## Verdict

**CONDITIONAL** — no `high` findings remain. Two `medium` findings are accepted
with stated rationale, both about the shape of a guard rather than a defect in
what it guards.

## Gates run

| Gate | Result |
| --- | --- |
| `make ci` | exit 0 — 40 Rust tests, 17 Python tests, `fmt --check`, `clippy -D warnings`, `cargo deny check licenses`, unsafe audit, corpus, corpus-repro, assurance |
| `make spec` | exit 0 — `quire validate`, `quire coverage --strict`, matrix status census |
| `make assurance` | exit 0 — 4/4 pins compatible, 19/19 census cases, 6/6 mutation probes, 10/10 chain scenarios, 3/3 adapter probes, 4/4 controls |
| `python3 scripts/validate_governance.py --mutation-probes` | 7/7 detected |
| evidence tree vs `origin/main` | `git diff --stat origin/main -- evidence/ schemas/ corpus/ src/` is empty |

## Findings

| ID      | Severity | Summary                                                                       | Refs                                        | Escape Cause                        |
| ------- | -------- | ----------------------------------------------------------------------------- | ------------------------------------------- | ----------------------------------- |
| FND-001 | high     | Retained-intake failures were discarded, so a failed retention surfaced later as a confusing `attestation_missing` | scripts/assurance_chain.py:453              | implementation-bug-despite-evidence |
| FND-002 | high     | `intake_detail["directory"]` indexed a value that is a string on the failure path, raising `TypeError` instead of the stated error | scripts/assurance_chain.py:453              | implementation-bug-despite-evidence |
| FND-003 | medium   | The mirror check exempted its own rule text by substring, so rewording the sentence would have evaded it | scripts/check_shared_pins.py:146            | correct-requirement-no-evidence     |
| FND-004 | medium   | The audit fixture carried a stand-in `report_digest` derived from the proof id — a fabricated identity in the one place identity is the point | scripts/assurance_chain.py:497              | implementation-bug-despite-evidence |
| FND-005 | medium   | Three `f"...{detail[:0]}"` interpolations rendered as nothing and existed only to silence an unused name | scripts/assurance_chain.py:570,723           | correct-requirement-no-evidence      |
| FND-006 | medium   | `Chain.seal_record` writes `configuration_digest` back into dicts it shares with the loaded declaration, so the two stay correct only by aliasing | scripts/assurance_chain.py:180              | correct-requirement-no-evidence      |
| FND-007 | medium   | The no-verdict-from-stdout guard is a lexical scan over `scripts/*.py`, not a semantic one | tests/test_shared_assurance.py:238          | correct-requirement-no-evidence      |
| FND-008 | low      | `TC-024`'s new script census read `__pycache__` bytecode and panicked on invalid UTF-8 | tests/governance_reconciliation.rs:342       | implementation-bug-despite-evidence |

## Dispositions

- **FND-001, FND-002 — FIXED.** Every intake that is meant to succeed now checks
  its status and raises `ChainError` with the tool's own message. The retention
  scenario additionally requires the returned payload to be a mapping before
  indexing it.
- **FND-003 — FIXED.** `mirror_references` no longer greps `assurance/pins.json`.
  It scans requirement files, lockfiles, `.npmrc`, `Cargo.toml`, and
  `package.json` line by line, and reads the pins file as JSON, inspecting only
  the two fields that name something installable. The check was then seen red:
  appending `--registry=https://npm.ix/` to `requirements-assurance.txt` turned
  the gate to `NOT satisfied`.
- **FND-004 — FIXED.** The audit report digest is now the SHA-256 of the
  canonically serialized report object, so it identifies the bytes it claims to.
- **FND-005 — FIXED.** Removed; the names they existed to consume are now `_`.
- **FND-006 — ACCEPTED.** The aliasing is deliberate and local: the attestation
  must name the same configuration digest the record sealed, and re-deriving it
  independently would create two computations that could disagree. It is
  confined to one method with a comment stating the coupling, and a divergence
  would be caught immediately — Quoin refuses an attestation whose
  `configuration_digest` does not match its record's proof obligation. A
  refactor to a single derived map is a readability change, not a correctness
  one, and is not made under a migration diff.
- **FND-007 — ACCEPTED, with the limitation stated.** A lexical scan cannot
  prove no verdict is ever recovered from console text; nothing short of the
  reviewer reading the diff can. What it does do is make the pattern expensive
  to introduce accidentally, and it is deliberately narrow — it names verdict
  words rather than banning every mention of `stdout`, because the broad version
  flagged `SEMVER.search(stdout)` and a rule that noisy gets reworded rather
  than obeyed. The substantive guarantee is structural and is checked elsewhere:
  the adapter binds on the runner's declared protocol
  (`quire.contract.conformance-jsonl/v1`) on every row and refuses a foreign
  one, and every attestation states its `result` rather than having one read
  off a stream.
- **FND-008 — FIXED.** The census filters to `.py`, `.sh`, and `.txt` sources.

## Notes on what was deliberately not changed

- `evidence/`, all five files in `schemas/`, `corpus/`, and `src/` are
  byte-identical to `origin/main`. The domain producer, its corpus, its
  diagnostics, and its result format are untouched, which is what the migration
  contract requires of a migration commit.
- The two generic PGM-01 schemas are frozen rather than deleted. Every retained
  manifest names one of them by path and digest; the family this migration
  removes is the *verifier*, and deleting the schema would break a reference
  inside bytes that must stay readable. `TC-024` now asserts no script
  references either, and that assertion was checked red by adding a reference
  and watching the test fail.
- `scripts/validate_governance.py` and
  `schemas/derivation-evidence-envelope-v1.schema.json` are `KEEP` in the
  accepted decision table and FR-008 already classifies the record as
  producer-owned structured output. They were not touched.
