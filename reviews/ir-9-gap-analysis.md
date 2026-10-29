---
id: SR-16634
title: "Gap analysis — IR-9 PR 555 string-edge traceability"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; PR #555 origin/main...HEAD; spec/core/functional/FR-056-consume-authoritative-string-edge-scan.md, Makefile, Cargo.toml, string-edge-rules.json, changed src/ and crates/ sources, tests/it/string_edge_gate.rs"
review_set: subset
relationships:
  - { target: "ix://agent-ix/quire-contract-ir/FR-056", type: references }
---

## Summary

Planless audit of FR-056 in the computed repository Test Matrix, changed test bindings and the changed code. Ticket: IR-9. Plan completion: not assessed.

## Verdict

**FAIL** — FR-056-AC-7's source inspection fails because marked converters mask post-intake dispatch. Seven Test criteria have trace bindings, but trace tags alone do not establish final executable evidence.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-056-AC-7 is not discharged: marked post-intake operation and term-name dispatch yields a clean scan while bypassing the required typed intake boundary. | crates/quire-contract-model/src/checked_package/v2/mod.rs:189 |

## Coverage

- FR-056-AC-1 (examined): The normal `make lint` dependency graph invokes the real external command with the repository base, exactly the two nonoverlapping production roots and the checked-in strict rules. A clean controlled consumer completes the invocation; the ordinary `ci` target retains `lint`.
- FR-056-AC-2 (examined): A real unmarked post-intake `match` on a closed node-tag wire spelling, introduced in a controlled consumer production file, makes that same gate entry point fail with the command's exit-one violation and expected file/item diagnostic. Restoring the enum-based control makes the scan pass.
- FR-056-AC-3 (examined): A genuine intake function marked with `ix_trace_rs::string_edge` compiles against the authoritative neutral crate and preserves its conversion/refusal behavior. The command accepts this marked edge while still reporting an adjacent unmarked closed-string decision. A look-alike qualified marker and comment-only marker do not exempt the adverse decision.
- FR-056-AC-4 (examined): An unchanged comparison of two supplied identity/selection keys passes the controlled scan, and an allowed detector-visible non-branching open-text comparison passes only with its valid rule. Removing that rule reports it; attempting to allow-list a branch-gating wire decision is a rule error rather than a clean scan. These controls state the detector's variable-to-variable limitation.
- FR-056-AC-5 (examined): A missing executable, malformed rules, missing source root or malformed Rust source makes the normal scan prerequisite fail. Request/rule/read/parse errors preserve the real command's exit-two diagnostic, and no path reports clean after failure or partial execution.
- FR-056-AC-6 (examined): Every production package that marks an intake function resolves the neutral marker from `ix-trace-rs`, and Cargo metadata confirms neither IR package adds a dependency in any kind on QSL or the scanner. Existing cycle-free metadata and intake conformance controls remain intact.
- FR-056-AC-7 (examined): Each marked function is an actual intake conversion; the checked-in rules contain no branch-dispatch suppression, file/crate exemption or copied scanner logic. Existing reported post-intake decisions retain their owning remediation until removed from the real report.
- FR-056-AC-8 (examined): The actual repository command completes with no unresolved occurrences across both roots, without removing a root, filtering diagnostics, introducing a local marker or weakening FR-064's rule checks. A fixture-only success cannot satisfy this criterion.

- Reconciliation: `quoin matrix --repo . --json` at PR, Quoin 0.29.1 and Quire 0.36.2; static tags and no bound run evidence.
- Plan completion: not assessed
- Repository criteria: 570 tagged, 86 untagged, 1 tagged by ignored test, 15 method without symbol. These include inherited and other-lane gaps, not new defects attributed to this PR.
- FR-056: 7 Test criteria tagged; AC-7 is Inspection and `method-without-symbol`. Verified the test symbols in `tests/it/string_edge_gate.rs` and the actual `Makefile` target. The installed scanner and external QSpec timed-model fixtures were unavailable to this independent pass.
- Untraced changed behavior: no separate changed public surface found beyond the FR-056 lint consumer and conversion work. Changed-source and test stub inspection found no new placeholder implementation.
- Semantic review: not requested; source behavior was inspected only to judge AC-7 marker placement.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | medium | The two new fix-round unit tests for retained catalog and binding-name kinds have no trace tags, so the computed matrix cannot associate their behavioral assertions with TC-460 or another owning obligation. | crates/quire-contract-model/src/checked_package/v2/typed_terms.rs:770 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The formerly post-intake operation and binding-name decisions now consume retained typed classes assigned at catalog and term intake. |
| FND-002 | still-open | The two new fix-round unit tests for retained catalog and binding-name kinds have no trace tags, so the computed matrix cannot associate their behavioral assertions with TC-460 or another owning obligation. |

## Current review state

**CONDITIONAL** — original high findings are fixed; the untagged new tests need a fix round. The installed scanner and full aggregate gate remain unqualified here.

## Dispositions (round 2)

| FND | outcome | reason |
| --- | --- | --- |
| FND-002 | fixed | Both new retained-intake unit tests carry `#[trace("TC-460")]`, linked to the consumer controls test case; no executable source behavior changed in this fix round. |

## Current review state (round 2)

**PASS for the PR diff** — all recorded findings have latest disposition `fixed`. Repository-wide pre-existing matrix gaps, installed scanner availability, and aggregate gate qualification are separate gates and remain unclaimed by this review.
