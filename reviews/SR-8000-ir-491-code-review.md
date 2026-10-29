---
id: SR-8000
title: IR-491 code-review review of residual leaf sources
type: SpecReview
analysis: code-review
scope: agent-ix/quire-contract-ir; spec/checked_package/functional/FR-038-consume-checked-package-v2.md,
  spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md; IR-491
review_set: subset
---

## Summary

No code, vendored file, duplicate encoder, or compatibility path is introduced; IR reader behavior is specified against QSpec and the pinned operation catalog.

## Scope

- `FR-038-AC-228` (examined): PLANNED/UNRUN (IR-491). Through `CheckedPackageV2::read`, otherwise valid `structural.eq` and `structural.ne` packages whose first operand is, separately, a reference to a structural value, a reference directly to its type node, an application with that `result_type`, or an option-typed `none` literal, each derive the leaves of the selected declared type. An all-integer record admits with `leaves: []`, and a record with one pinned text field admits with exactly that field's selected-law and mode leaf, while removing the required leaf refuses `invalid_package`/`operation-law-missing` at `operat
- `FR-038-AC-229` (examined): PLANNED/UNRUN (IR-491). For `structural.eq`, `structural.ne` and `collection.contains`, a first operand replaced with a `binding`, `aggregate`, `frame` or `abstraction_relation` never admits by skipping type resolution: after any earlier applicable grammar/reference check, it refuses `ill_typed`/`operator-ineligible` at `arguments/0` if it reaches the operand-family check. A `dependency_reference` in that position is refused by the earlier dependency-reference placement check. Separately, a first operand whose family fits but whose required compared or `inner:0` type remains unresolved after e
- `FR-038-AC-230` (examined): PLANNED/UNRUN (IR-491). For every catalogued `result_inner` identity (`collection.set`, `.bag`, `.ordered_set`, `.map`, `.flat_map`, `.flatten`, `.convert`), a result of `set`, `bag` or `ordered_set` over pinned text admits exactly the selected element-type leaf at the empty path and refuses an omitted one as `operation-law-missing` at `operation.leaves`; where the catalogued result form permits a valid `sequence` result over that text, it admits with `leaves: []` and refuses one supplied leaf as `operation-law-mismatch` at `operation.leaves/0`. A set-like result whose required element type re
- `FR-038-AC-231` (examined): PLANNED/UNRUN (IR-491). An otherwise valid `structural.eq` or `structural.ne` over a record, tuple, option or sequence whose selected compared type reaches Float32 or Float64 at any depth, and `collection.contains` over a sequence whose element type does, each refuse `ill_typed`/`operator-ineligible` at `operation.leaves` with `leaves: []` and with a supplied text leaf; changing only that nested IEEE type to an integer type and re-deriving identities admits with its exact empty leaf list. A valid `collection.flatten` result of `sequence<Float32>` or `sequence<Float64>` admits with no leaves be
- `TC-048` (examined): Use the current first-party catalog from `quire-verification-contracts` and
its current QSpec operation fixtures by reference from their authoritative
checkouts; do not copy those files into this repository. Start each case with
an admitted V2 package whose selected operation, lock definitions, source
occurrences, node keys, identity projection and package id agree. Assert that
the unchanged package admits through `CheckedPackageV2::read`. For each
mutation below, rebuild the affected node keys and package id using the
existing production identity rules, and assert that the adverse package
pas
- `FR-038` (examined): The planned IR-491 residual rule applies only after the earlier package, reference, operation and operand checks have su. The catalog names exactly the following leaf-source cases; the reader SHALL use the selected type in this table and SHALL NOT treat failure to resolve a required type as an empty leaf list:

## Verdict

**PASS** — No defect found in this lens.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
