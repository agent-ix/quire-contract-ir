---
id: FR-054
title: "Expose ordered scalar operands from an admitted ContractPackage"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/FR-035
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-049
    type: references
  - target: ix://agent-ix/quire-specification/FR-330
    type: references
---
# FR-054: Expose ordered scalar operands from an admitted ContractPackage

## Description

When a consumer supplies one exact QSL-checked application occurrence for a
scalar integer expression, Contract IR SHALL project its operands from the
reader-admitted six-field `ContractPackageV1`. The projection SHALL retain the
authored `body.arguments` order, including repeated references, and distinguish
a represented graph reference from an inline integer literal. It SHALL use the
same IR-owned scalar argument interpretation as the checked-package reader;
the consumer SHALL NOT parse `serde_json::Value` or return to the original
checked graph to reconstruct these facts.

The caller supplies a typed application node key and its exact
`ContractOccurrenceKeyV1` (node key, `expression` role and ordinal). The
admitted source map proves that occurrence's correspondence to the represented
node. As with FR-049, a different valid represented occurrence of the same
node can pass IR's structural check; the QSL checked request and faithful
driver mapping establish which occurrence belongs to the routed item. This
query does not establish a unique original expression by itself.

## Outputs

The typed result SHALL distinguish a lowered application, its exact
non-lowered feature-accounting record, and an absent request. A lowered result
SHALL carry the admitted source package and lowering profile, package identity,
the supplied authenticated application occurrence, and one entry per authored
argument position. Each entry SHALL retain its zero-based position. A graph
reference SHALL carry its exact represented child key and admitted child
source-map entries. An inline integer SHALL carry its canonical signed integer
value, the application occurrence and its argument position; it SHALL NOT
claim a separate graph node or source-map entry. The application's admitted
source entry remains available for the inline position.

Each entry SHALL separately describe its own integer range as either a finite
inclusive range or a typed absence. An unbounded integer type or endpoint
outside the finite native range SHALL be distinguishable without erasing the
argument entry. A defensively encountered missing range shape SHALL have its
own typed absence; strict checked-package admission can reject such a shape
before this accessor is reached. A literal's exact value remains available
even when it cannot be represented as a finite native range. The query SHALL
NOT infer, select or invent a generation interval; the consumer owns its
choice among own, inferred and result-bound intervals.

A foreign or missing application, a non-lowered request, an invalid supplied
occurrence, an ineligible operation or an inconsistent represented child SHALL
return a typed non-success without any selected argument list. No first
occurrence, lowest ordinal, sorted closure key or duplicate elimination may
repair an invalid occurrence. Defensive body inconsistency and work exhaustion
retain the existing typed admitted-view error discipline.

## Behavior

Contract IR SHALL validate the exact lowered feature-accounting record,
represented application node and matching `expression` source-map occurrence
before reading positions. It SHALL use the admitted application's operation
identity to admit the scalar integer arithmetic family and its validated
argument terms. For every graph reference it SHALL require the represented
child and source-map correspondence. For every inline integer it SHALL read the
canonical literal through IR-owned typed parsing. It SHALL preserve two
positions even when their referenced child keys are equal. Contract IR SHALL
make the full query succeed or fail atomically: a malformed later argument exposes no
partial prefix.

The checked-package operand accessor may still require every finite `i128`
range for its own contract. The admitted-package accessor SHALL expose
position and child identity independently of that older all-or-error range
policy. The two accessors SHALL share one IR-owned interpretation of scalar
argument terms and range facts, not maintain two raw-JSON extractors.

Querying SHALL not mutate any of FR-330's six fields, canonical package
identity, request disposition, FR-049 result context or the application's
forward reached closure. No seventh wire field, copied schema, second
lowering product, checked-graph fallback or compatibility alias is introduced.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-054-AC-1 | For a reader-admitted positive scalar application with a graph reference followed by an inline integer literal, a public typed `ContractPackageV1` query returns positions 0 and 1 in that order, the exact child key and admitted sources at 0, and the exact signed literal value qualified by the application occurrence and position at 1, with no fabricated literal node. | Test (TC-458) |
| FR-054-AC-2 | For an admitted application whose two argument positions reference the same represented child, the query returns two distinct ordered entries with positions 0 and 1 and the same authentic child key and sources; reached-closure set order and deduplication do not alter them. | Test (TC-458) |
| FR-054-AC-3 | For admitted applications with unbounded or outside-native-range own facts, the query still returns the complete positional identity vector and distinguishes those facts from finite ranges. It supplies no inferred or result-fallback interval and does not change the interval selected by an existing consumer. | Test (TC-458) |
| FR-054-AC-4 | A missing, foreign, non-lowered or non-application key, invalid role/ordinal/source correspondence, ineligible operation or inconsistent represented child returns typed absence, disposition, refusal or error with no selected operand list; an independently valid represented occurrence of the same node remains structurally valid and does not prove unique-original-expression ownership. | Test (TC-458) |
| FR-054-AC-5 | A separate consumer test compiles against public owning types and reads the preceding cases from reader-admitted six-field packages without a copied schema, consumer JSON field reads or a CheckedPackageV2 generation fallback. Repeated queries leave the wire, identity, request disposition, FR-049 result context and forward closure unchanged; this accessor alone makes no full scalar replay claim. | Test (TC-458) |

## Dependencies

FR-035 owns lowering and request disposition. FR-038 owns strict checked-package
and body admission. FR-049 owns independent result-context selection. QSpec
FR-330 owns the six-field package wire and source correspondence. QSL owns
the checked requirement occurrence supplied by the driver. CG owns generation
interval choice and scalar replay settlement.
