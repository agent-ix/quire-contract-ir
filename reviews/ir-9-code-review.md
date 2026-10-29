---
id: SR-16633
title: "Code review — IR-9 PR 555 string-edge consumer"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; PR #555 origin/main...HEAD; spec/core/functional/FR-056-consume-authoritative-string-edge-scan.md, Makefile, Cargo.toml, string-edge-rules.json, changed src/ and crates/ sources, tests/it/string_edge_gate.rs"
review_set: subset
---

## Summary

Reviewed the frozen PR #555 diff, including Rust semantics, markers, Make wiring and controlled scanner tests. Ticket: IR-9.

## Verdict

**FAIL** — two marker placements exempt post-intake decisions, contrary to FR-056-AC-7. The local installed `qsl-string-edge` executable is absent; the author's owner-source scan log reports zero occurrences, but this review does not treat that as final installed-tool qualification.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Marked KnownOperation::from_wire and TermBindingName::from_wire are called from post-intake operation/owner decisions over admitted values. The scanner excludes those functions as intake edges, making the clean report possible without retaining typed values at intake. Move these decisions to genuine intake or retain their typed results; do not exempt the later dispatch. | crates/quire-contract-model/src/checked_package/v2/mod.rs:189 |
| FND-002 | high | The marker on semantic_ir_relationship exempts the whole relationship validator. That function checks raw direction/category strings and then makes reference, owner and multiplicity decisions after conversion. Split the closed-string intake converters from the validator and leave the post-intake function unmarked. | crates/quire-contract-model/src/checked_package/v2/model_members.rs:2770 |

## Coverage

- FR-056-AC-1 (examined): The normal `make lint` dependency graph invokes the real external command with the repository base, exactly the two nonoverlapping production roots and the checked-in strict rules. A clean controlled consumer completes the invocation; the ordinary `ci` target retains `lint`.
- FR-056-AC-2 (examined): A real unmarked post-intake `match` on a closed node-tag wire spelling, introduced in a controlled consumer production file, makes that same gate entry point fail with the command's exit-one violation and expected file/item diagnostic. Restoring the enum-based control makes the scan pass.
- FR-056-AC-3 (examined): A genuine intake function marked with `ix_trace_rs::string_edge` compiles against the authoritative neutral crate and preserves its conversion/refusal behavior. The command accepts this marked edge while still reporting an adjacent unmarked closed-string decision. A look-alike qualified marker and comment-only marker do not exempt the adverse decision.
- FR-056-AC-4 (examined): An unchanged comparison of two supplied identity/selection keys passes the controlled scan, and an allowed detector-visible non-branching open-text comparison passes only with its valid rule. Removing that rule reports it; attempting to allow-list a branch-gating wire decision is a rule error rather than a clean scan. These controls state the detector's variable-to-variable limitation.
- FR-056-AC-5 (examined): A missing executable, malformed rules, missing source root or malformed Rust source makes the normal scan prerequisite fail. Request/rule/read/parse errors preserve the real command's exit-two diagnostic, and no path reports clean after failure or partial execution.
- FR-056-AC-6 (examined): Every production package that marks an intake function resolves the neutral marker from `ix-trace-rs`, and Cargo metadata confirms neither IR package adds a dependency in any kind on QSL or the scanner. Existing cycle-free metadata and intake conformance controls remain intact.
- FR-056-AC-7 (examined): Each marked function is an actual intake conversion; the checked-in rules contain no branch-dispatch suppression, file/crate exemption or copied scanner logic. Existing reported post-intake decisions retain their owning remediation until removed from the real report.
- FR-056-AC-8 (examined): The actual repository command completes with no unresolved occurrences across both roots, without removing a root, filtering diagnostics, introducing a local marker or weakening FR-064's rule checks. A fixture-only success cannot satisfy this criterion.

- Reviewed `origin/main...HEAD` only; no CI workflow changes, no rules allow-list entries, and exactly two source roots in the Make invocation.
- Controlled tests tag AC-1..6 and AC-8 and use the real Make entry point. AC-7 is an Inspection row and needs this source review.
- QSpec `timed_model` external fixture files and the installed scanner were unavailable locally. Full aggregate gates were not run in this review.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | medium | The fix adds catalog_intake_retains_known_operation_class and admitted_binding_name_retains_its_class_without_changing_wire_text without a `#[trace(...)]` on either test. This repository traces new tests to requirements or test cases; both assertions are invisible to the computed matrix. Add an appropriate trace tag (TC-460 or the owning criterion) to each test. | crates/quire-contract-model/src/checked_package/v2/operation_catalog.rs:362 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Retained `OperationCatalogEntry.known` is assigned once during catalog indexing, and `CheckedBindingName.kind` is assigned at typed-term deserialization; post-intake callers use the retained kinds. |
| FND-002 | fixed | The broad `semantic_ir_relationship` marker was removed. Dedicated `RelationshipMember`, `RelationshipDirection` and `RelationshipCategory` converters carry the marks; reference, owner and multiplicity checks remain unmarked. |
| FND-003 | still-open | The fix adds catalog_intake_retains_known_operation_class and admitted_binding_name_retains_its_class_without_changing_wire_text without a `#[trace(...)]` on either test. This repository traces new tests to requirements or test cases; both assertions are invisible to the computed matrix. Add an appropriate trace tag (TC-460 or the owning criterion) to each test. |

## Current review state

**CONDITIONAL** — original high findings are fixed; the untagged new tests need a fix round. The installed scanner and full aggregate gate remain unqualified here.

## Dispositions (round 2)

| FND | outcome | reason |
| --- | --- | --- |
| FND-003 | fixed | Both new retained-intake unit tests carry `#[trace("TC-460")]`, linked to the consumer controls test case; no executable source behavior changed in this fix round. |

## Current review state (round 2)

**PASS for the PR diff** — all recorded findings have latest disposition `fixed`. Repository-wide pre-existing matrix gaps, installed scanner availability, and aggregate gate qualification are separate gates and remain unclaimed by this review.
