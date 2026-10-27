---
id: FR-066
title: "Retain identity-bound execution points in the checked V2 model"
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

# FR-066: Retain identity-bound execution points in the checked V2 model

## Description

When Contract IR reads or writes a checked-package V2 graph, its public
`CheckedSemanticNodeV2` SHALL carry each QSpec execution point at the exact
identity-bearing node-body location, retaining the closed variant and name.
This requirement makes the typed graph, strict reader, writer and identity
derivation agree on the carrier already admitted by
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
or its absence, with the rederived node and package identities; otherwise the
first typed, located reader refusal and no partial admitted package.

## Behavior

The model's `CheckedSemanticNodeV2` body representation SHALL distinguish an
absent point from each of QSpec's four closed values:
`initialization{name}`, `handler{name}`, `pre{operation}` and
`post{operation}`. The state-clause operation member SHALL retain its optional
point as part of the application body. The operation-anchor body SHALL retain
its optional fourth binding as a bare point object, rather than an expression
term. The raw reader form, admitted typed form, borrowed package identity
projection and public writer SHALL preserve those same locations and values.
There SHALL be one body carrier per node; source occurrences do not own
separate points. The producer's source-to-point agreement remains QSL-owned.

The reader and pre-read identity candidate path SHALL derive the
`state_clause` application key from a preimage containing the optional
operation-member point. They SHALL derive the `operation_anchor` structural
key from a preimage containing the optional fourth binding. The package
identity projection SHALL include each complete body and omit source
occurrences. A point-only edit therefore changes the relevant node key and
the package id after dependent nodes are rekeyed. When a supplied node key is
stale, the reader SHALL refuse before state-body semantics. Neither the writer
nor the reader SHALL substitute a sidecar point, a clause-kind default or an
anchor-inferred point.

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
| FR-066-AC-1 | The four published QSpec `node-identity-vectors.json` execution-point application preimages, read directly from the selected current QSpec checkout, each recompute to its recorded SHA-256 and agree with the IR application candidate for the same body. Separately, complete reader-admissible clause and anchor fixtures retain each exact point variant and name in `CheckedSemanticNodeV2`; an omitted point remains absent. No QSpec schema or vector is copied into IR. | Test |
| FR-066-AC-2 | For otherwise equal valid state-clause bodies, absent, `initialization{startup}` and `handler{tick}` points produce distinct application preimages and keys. For otherwise equal valid operation-anchor bodies, absent, `pre{adjust}` and `post{adjust}` points produce distinct structural preimages and keys. Independent canonical derivation agrees with QSpec's four published clause point vectors and the published anchor schema; rekeyed packages have distinct package ids, while retaining a pre-edit key refuses `invalid_package`/`stale-node-key` at the original node's `node_id` before state checks. | Test |
| FR-066-AC-3 | The public typed encoder writes a clause point only at `body.operation.member.execution_point` and an anchor point only as the bare fourth `body.members` binding; an absent point adds neither member. A read/write/read round trip retains equal typed bodies, points, node keys and package id. An external consumer constructs these nodes with the public model and encoder without an untyped envelope or private QSL type. | Test |
| FR-066-AC-4 | A duplicate `execution_point` JSON key, unknown point kind, missing or extra point member, empty name, wrong fourth binding position, point on another operation-member kind, inapplicable point, mismatched operation name, and unequal or one-sided clause/anchor points each return FR-040's specified first code, cause and locus with no admitted package. Paired earlier wire, graph or stale-key defects win according to FR-040-AC-35. | Test |
| FR-066-AC-5 | Inspection finds one identity-bearing body carrier at each QSpec location in `CheckedSemanticNodeV2`, its raw and projection paths, and the writer. No additional top-level point, occurrence-keyed point list, QSpec schema/vector copy, alternate preimage calculation, or fallback inference exists. | Inspection |

## Dependencies

[FR-040](./FR-040-admit-frame-entries-and-state-clauses.md) owns the reader's
state and anchor semantics and refusal order;
[FR-038](./FR-038-consume-checked-package-v2.md) owns strict admission and
identity checks; [FR-051](./FR-051-encode-checked-package-v2-envelope.md)
owns typed encoding. QSpec FR-322-AC-131, FR-341-AC-14 and FR-342-AC-10
own the wire, preimage and published vector oracles. The accessor in
[FR-058](./FR-058-admitted-clause-execution-point.md) consumes the admitted
clause point after this carrier is implemented.
