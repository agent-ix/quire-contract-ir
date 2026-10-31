---
id: FR-066
title: "Retain execution points in the checked V2 model"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-040
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-051
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-058
    type: references
  - target: ix://agent-ix/quire-specification/FR-322-AC-131
    type: references
  - target: ix://agent-ix/quire-specification/FR-341-AC-14
    type: references
  - target: ix://agent-ix/quire-specification/FR-342-AC-10
    type: references
---

# FR-066: Retain execution points in the checked V2 model

## Description

When Contract IR reads or writes a checked-package V2 graph, its public
`CheckedSemanticNodeV2` SHALL carry each QSpec execution point at the exact
node-body location, retaining the closed variant and name.
This requirement makes the typed graph, strict reader and writer
agree on the carrier already admitted by
[FR-040](./FR-040-admit-frame-entries-and-state-clauses.md). It adds no
occurrence-keyed or top-level node point: an optional state-clause point is
`body.operation.member.execution_point`, and an optional operation-anchor
point is the fourth `body.members` binding's bare value.

## Inputs

The existing typed V2 graph used by the public encoder of
[FR-051](./FR-051-encode-checked-package-v2-envelope.md), or untrusted V2
bytes and reader limits accepted by
[FR-038](./FR-038-consume-checked-package-v2.md). The point shape and its two
locations are the published QSpec FR-322, FR-341 and FR-342 contracts.

## Outputs

Canonical V2 bytes and an admitted typed graph retaining each present point
or its absence and its structural node id; otherwise the
first typed, located reader refusal and no partial admitted package.

## Behavior

The model's `CheckedSemanticNodeV2` body representation SHALL distinguish an
absent point from each of QSpec's four closed values:
`initialization{name}`, `handler{name}`, `pre{operation}` and
`post{operation}`. The state-clause operation member SHALL retain its optional
point as part of the application body through a public typed point field or
equivalent typed constructor and accessor. The operation-anchor body SHALL
retain its optional fourth binding as a bare point object through the same
public typed point value, rather than an expression term. A caller SHALL be
able to construct and inspect both points without building or decoding a
`serde_json::Value`. The raw reader form, admitted typed form, borrowed
public writer SHALL preserve those same
locations and values.
There SHALL be one body carrier per node; source occurrences do not own
separate points. The producer's source-to-point agreement remains QSL-owned.

The writer SHALL preserve the point as a member of the complete node body. The reader SHALL retain the authored structural id and validate point shape, applicability and clause-anchor agreement independently of that id. A valid point-only edit can retain the same id and changes the admitted body and point accessor result. Neither the writer nor the reader SHALL substitute a sidecar point, clause-kind default or anchor-inferred point.

The strict reader SHALL apply the existing [FR-040](./FR-040-admit-frame-entries-and-state-clauses.md)
shape, applicability, equality and refusal precedence to both carriers.
Malformed point objects fail the closed body shape before state checks;
well-formed but inapplicable or disagreeing points fail at the state step.
A clause-bound operation anchor and its clause SHALL agree, including
one-sided absence. A protocol-only anchor may omit its point. This
requirement SHALL preserve the exact code, cause and RFC 6901 locus of
FR-040-AC-31 through FR-040-AC-36.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-066-AC-1 | Complete reader-admissible clause and anchor fixtures retain each exact execution-point variant and name in CheckedSemanticNodeV2; an omitted point remains absent. The public typed writer preserves those same values and locations after read/write/read. No QSpec schema or vector is copied into IR. | Test |
| FR-066-AC-2 | Withdrawn by the IR-1141 structural-identity decision under QSpec #517: the reader does not recompute identity from body content or refuse a retained structural id solely because a valid body changed. This id is retained and never reused. | Withdrawn |
| FR-066-AC-3 | PLANNED/UNRUN (IR-1141). An external consumer constructs and inspects clause and anchor points through public typed fields or constructors without serde_json::Value. The encoder writes points only at the QSpec clause and anchor body locations; absence adds no point. Read/write/read preserves both complete bodies and their structural node ids. An untyped-only carrier does not pass. | Test |
| FR-066-AC-4 | PLANNED/UNRUN (IR-1141). Duplicate, malformed, misplaced, inapplicable, mismatched and one-sided execution points return FR-040-AC-31 through FR-040-AC-35 first applicable code, cause and locus with no package. Two anchors of the same context and operation with distinct pre/post points admit; duplicate (context, operation, point) tuples refuse ambiguous_declaration/ambiguous-name at the second anchor in structural-id order. | Test |
| FR-066-AC-5 | Inspection finds one typed body carrier for each QSpec execution-point location in CheckedSemanticNodeV2, reader and writer. No top-level point, occurrence-keyed point list, schema/vector copy, preimage calculation or fallback inference exists. | Inspection |

## Dependencies

[FR-040](./FR-040-admit-frame-entries-and-state-clauses.md) owns the reader's
state and anchor semantics and refusal order;
[FR-038](./FR-038-consume-checked-package-v2.md) owns strict admission and
structural graph checks; [FR-051](./FR-051-encode-checked-package-v2-envelope.md)
owns typed encoding. QSpec FR-322-AC-131, FR-341-AC-14 and FR-342-AC-10
own the wire, structural body and published fixture oracles. The accessor in
[FR-058](./FR-058-admitted-clause-execution-point.md) consumes the admitted
clause point after this carrier is implemented.
