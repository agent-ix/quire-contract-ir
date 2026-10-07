---
id: SR-2942
title: "Integrity review of the IR-663 private intake retention amendment"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir PR 321 (IR-663 amendment, specification only); spec/checked_package/functional/FR-038-consume-checked-package-v2.md; spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: references
  - target: ix://agent-ix/quire-contract-ir/TC-048
    type: references
---

## Summary

Ticket: IR-663. Consistency, interpretation and testability of the amended
retention contract against QSpec FR-154, the FCD semantic IR schema (every
`field`, `operation`, `relationship`, `variant`, `state`, `transition` and
`step` carries its own required `identity` and `origin`) and the current
intake code paths (`classify` grouping same-identity type nodes; sequential
`relationships.insert` detection of a repeated relationship identity;
`Defects::first` for nested member defects). The private boundary, the absence
rules and the pre-declaration/Limit/InexactNumber rules are consistent and
testable. Gaps remain where several nodes or nested non-relationship
declarations are involved: the rule admits more than one reading there and
TC-048 does not pin it.

## Examined units

- FR-038 retention section, identity, nested-declaration, grouped-duplicate, pre-declaration, boundary and open-item paragraphs (examined)
- FR-038-AC-168, FR-038-AC-174, FR-038-AC-175 (examined)
- TC-048 private intake steps 1, 3, 4 and 5 (examined)
- QSpec FR-154 declaration-refusal table and retention paragraph (context_only)
- FCD `semantic-ir.schema.json` `$defs` field/operation/relationship (context_only)

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The grouped-duplicate origin rule is ambiguous in the mixed case: one candidate with a valid origin and another with a missing or malformed origin. "One valid origin is unambiguous across the candidate nodes" reads either as "every candidate agrees" (retain None) or "exactly one distinct valid value exists" (retain it, which is choosing that node). TC-048 step 4 exercises identical, distinct and no valid origins but not the mixed case, so either implementation passes. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1191-1195; spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:1276-1281 |
| FND-002 | medium | The grouped rule's scope is unstated for other multi-node refusals: (a) a relationship identity repeated under two owners (AC-168) is detected one insert at a time, not as a grouped candidate set, so an implementer may retain the second relationship's origin as "the actual offending node"; (b) a same-identity type group refused `malformed-declaration` (shared identity fails the object-id rule, or one candidate's kind is bad) has several candidate origins and no rule, so document order silently chooses one. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1191-1195; spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3800 |
| FND-003 | medium | Nested declarations other than relationships are unpinned. FCD fields, operations and parameters carry their own `identity` and `origin`, but the contract only says a nested defect retains "that declaration's own context" (undefined) and TC-048 tests only a nested relationship. For an unresolved field `typeRef`, retaining the field's identity/origin and retaining the owning type's both satisfy the current text and tests. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1129,1185-1187,3806; spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:1256-1261 |
| FND-004 | low | "Distinct valid origins require None" is an IR-local refinement: FR-154 retains no origin only when the origin is missing or malformed. The text does not say it is a local decision for a case FR-154 leaves open, so a later reader may take it as an FR-154 deviation. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1192-1194 |
| FND-005 | low | The private fields have no production reader by design (the only consumer, the `ValidationFailure` conversion, must drop them). `make lint` also runs `clippy -p quire-contract-model -- -D warnings` without `--all-targets`, where never-read fields raise `dead_code`. The contract does not say how write-only retention is meant to pass that gate, so the code slice will need an undocumented allow. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1159-1161,1206-1211 |

## Verdict

Conditional: the boundary and absence rules are sound. FND-001 to FND-003
need one sentence each in the contract plus one TC-048 case each, so that the
planned code slice has a single correct reading.

## Dispositions

Round 1, fix commit "spec: close IR663 private intake review findings".

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | fixed by commit "spec: close IR663 private intake review findings" |
| FND-002 | fixed | fixed by commit "spec: close IR663 private intake review findings" |
| FND-003 | fixed | fixed by commit "spec: close IR663 private intake review findings" |
| FND-004 | fixed | fixed by commit "spec: close IR663 private intake review findings" |
| FND-005 | fixed | fixed by commit "spec: close IR663 private intake review findings" |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-006 | low | AC-194's line between required 'consumption/release' at the conversion and forbidden 'fake reads solely to suppress unused fields' has no objective test. The only production consumer projects code/path/cause and drops the metadata, so destructure-and-drop there matches both descriptions and two inspectors can disagree. The text does route a gap to owner resolution, so nothing wrong ships, but the Inspection verdict stays a judgment call. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3854 |
| FND-007 | low | 'Unselected' (group / same-identity type group) is undefined; it reads as a subset of same-identity groups but appears to mean every group of two or more type declarations sharing an identity. Drop the word or define it. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3850 |
