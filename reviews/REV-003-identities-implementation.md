---
id: REV-003
title: "Issue 6 identities implementation review and gap analysis"
type: Review
relationships:
  - target: ix://agent-ix/quire-contract-ir/issues/6
    type: reviews
---
# Issue 6 identities implementation review and gap analysis

Date: 2026-08-30

Four independent read-only review passes examined the issue #6 implementation
and tests. They reported 24 actionable findings. The reviewer performed static
analysis only and did not attest to local test execution; local gate results are
recorded separately. Every finding below was fixed before candidate evidence was
minted.

| ID | Severity | Finding | Disposition |
|---|---|---|---|
| ID-R01 | high | Validating deserializers had only a happy-path round-trip test. | TC-015 now mutates every issue #6 identity, revision, span, dependency, clause, and package invariant through untrusted JSON. |
| ID-R02 | high | Serde errors reduced structured diagnostic codes to message text. | `ContractPackage::from_json_str` is the sole package decoder and returns `Vec<Diagnostic>`; wire syntax/shape failures use registered `invalid_wire_format`. |
| ID-R03 | medium | Duplicate requirements related the later rather than earlier identity. | Validation retains the first requirement in a map and cites its exact revision; TC-015 pins the earlier revision. |
| ID-R04 | medium | Duplicate clauses omitted the earlier related identity. | Requirement construction receives package identity and cites the first exact clause identity. |
| ID-R05 | medium | Package/source mismatch diagnostics omitted available spans. | Requirement and clause mismatch branches attach their validated spans and TC-015 asserts them. |
| ID-R06 | medium | Identifier paths exposed Rust type names. | Identifier constructors use explicit language-neutral semantic paths, also covered by vocabulary scans. |
| ID-R07 | medium | Anchor, clause, and source identity negatives were incomplete. | TC-015 covers every public identifier family and malformed anchor names. |
| ID-R08 | medium | Revision invalidation did not prove unrelated identities remain stable. | A two-requirement fixture compares the unaffected requirement, clause, and dependency identities across a sibling revision advance. |
| ID-R09 | medium | The two declared diagnostic precedence examples were untested. | Empty package and clause identities have direct precedence assertions. |
| ID-R10 | low | Dependency derivation compared a method with its own implementation. | The test now compares against an independently constructed expected identity set. |
| ID-R11 | low | Package namespaces permitted empty, dot, and traversal segments. | Segment-wise validation rejects empty, `.` and `..` segments, with negative cases. |
| ID-R12 | medium | Malformed wire dependencies discarded an available clause span. | Wire clause validation attaches its span to body/reference diagnostics. |
| ID-R13 | medium | Four reference failure classes lacked span regression assertions. | Cross-package, stale, orphaned-requirement, orphaned-clause, and malformed-reference spans are all asserted. |
| ID-R14 | medium | Language-neutral vocabulary inspection sampled too few variants. | TC-015 serializes every clause, anchor, dependency, semantic-identity, and diagnostic-code variant against one forbidden vocabulary. |
| ID-R15 | low | Diagnostic enum, wire spelling, and registry enumeration could drift. | One macro emits variants, exact wire spellings, and `ALL`; tests round-trip every code and compare registry cardinality. |
| ID-R16 | low | Generic package deserialization still offered a lossy message-only path. | The lossy `Deserialize` implementation was removed in favor of the structured package decoder. |
| ID-R17 | low | The schema-version integration test was untraced. | It is now requirement-tagged as TC-015 / FR-011-AC-1. |
| ID-R18 | medium | Diagnostic serialization spelling and `as_str` literals had separate sources. | Per-variant Serde names come from the same macro literal and every code round-trips exactly. |
| ID-R19 | medium | Clause/anchor tests covered only selected accepted and rejected pairs. | TC-015 executes the complete six-clause-by-five-anchor-state matrix. |
| ID-R20 | medium | Malformed dependency path segments lacked explicit criterion and test ownership. | FR-011-AC-3 and STD-001 own the failure; TC-015 asserts code and path. |
| ID-R21 | low | Wire clause validation could report a bad span before an invalid clause/anchor identity. | Clause and anchor identities validate before source spans, with a two-fault precedence test. |
| ID-R22 | low | Task and traceability states still said implementing after acceptance was met. | TASK-006 and all issue #6-owned matrix rows now say implemented/done; later criteria remain planned. |
| ID-R23 | low | Invalid package references reported the package root rather than reference path. | `RequirementRef::parse` reports `reference.package`, pinned by TC-015. |
| ID-R24 | low | The StR-001 matrix summary still described completed issue #6 work as implementing. | The row now says issue #6 is implemented while leaving later children planned. |

No review finding is waived. FR-012-AC-5 remains explicitly assigned to issue
#8 and is not represented as issue #6 coverage. The full Wave 1 strict coverage
gate therefore remains open for planned issues #8 through #10.

## Local Verification Boundary

The independent reviewer did not execute repository commands. The implementer
must separately retain exact local results for formatting, Clippy, governance
mutation probes, unit/integration tests, licenses, unsafe checks, Quire document
validation, and issue-scoped coverage reconciliation. No local result is an
independent approval or an automated source-release decision.
