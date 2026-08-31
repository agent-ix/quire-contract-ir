---
id: FR-012
title: "Anchor clauses and derive referenced-value dependencies"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-011
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-014
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-016
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-017
    type: references
---
# FR-012: Anchor clauses and derive referenced-value dependencies

## Description

The model shall represent source spans, named execution points, executable and
non-executable clause kinds, and mechanically derived referenced-value
dependencies.

## Inputs

Source ranges, anchor names, clause kinds, expressions, and declarations.

## Outputs

Validated anchored clauses with deterministic dependency sets and structured
reference diagnostics.

## Behavior

Executable preconditions, postconditions, invariants, assertions, and cases
require a named initialization, handler, or pre/post anchor. Informational
clauses remain unanchored and non-executable. Anchor compatibility is closed:
preconditions use pre anchors; postconditions use post anchors; invariants use
initialization or handler anchors; assertions use any named anchor; and cases
use handler anchors.

Issue #6 defines a recursive dependency-source abstraction whose nodes expose
zero or more referenced identities and child nodes. Dependency derivation walks
that structure, deduplicates by structural identity, and returns a deterministic
set. TC-015 verifies traversal and deduplication using the issue #6 reference
body; TC-016 verifies that the issue #8 typed expression tree exposes every
input, state value, field, enum variant, and called pure function. This ordering
is not the canonical byte ordering owned by FR-016.

Reference resolution requires exact package, requirement revision, and clause
identity. Malformed references, absent current requirements/clauses, and stale
revisions produce distinct structured diagnostics from STD-001. An orphaned
reference here means a missing semantic target; FR-017 separately owns orphaned
artifact trace-coverage classification.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-012-AC-1 | A floating executable clause is rejected at its source span with a stable diagnostic code. | Test (TC-015) |
| FR-012-AC-2 | The issue #6 recursive reference body returns each exposed structural dependency identity exactly once in deterministic order. | Test (TC-015) |
| FR-012-AC-3 | Malformed, cross-package, stale-revision, orphaned-requirement, and orphaned-clause references produce distinct STD-001 diagnostic codes and the narrowest available source span. | Test (TC-015) |
| FR-012-AC-4 | Every executable clause kind accepts only its declared anchor kinds using `floating_executable_clause` or `incompatible_clause_anchor`; informational clauses reject any anchor using `informational_clause_anchored`; malformed anchor identifiers use `invalid_identifier`. | Test (TC-015) |
| FR-012-AC-5 | The issue #8 typed expression tree implements the dependency-source contract and exposes every input, state, field, enum variant, and pure-function reference. | Test (TC-016) |
| FR-012-AC-6 | Source spans reject zero line/column positions, decreasing byte offsets or positions, and endpoints from different source-document identities/revisions using `invalid_source_span`. | Test (TC-015) |

## Dependencies

FR-011 supplies package and requirement identity. FR-014 later supplies the
complete typed expression tree; FR-016 later defines canonical encoding and
canonical byte ordering.
