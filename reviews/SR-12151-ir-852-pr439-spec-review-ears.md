---
id: SR-12151
title: "IR-852 spec review (ears-conformance)"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-contract-ir@edd6bf4fe524637261c7d6a5c0bfbbaa33efe77d (PR #439); spec/checked_package/functional/FR-038-consume-checked-package-v2.md (Typed population of a reaches application: requirement statements of The link, The accessor, the error paragraph, The reader check, Dependencies)"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: references
---

# IR-852 spec review (ears-conformance)

## Summary

Ticket: IR-852. This review applies the EARS lens to each new requirement
statement in the FR-038 section "Typed population of a reaches application". The
accessor statement ("`CheckedPackageV2::reaches_population` ... shall return, for
the node whose body root is the ... application, the selector FR-053 reads") is a
well-formed ubiquitous statement. Its two bullets are event-driven ("When the
application's `operation.member` has no `population` key, the accessor returns
`Ok(None)`"; "Otherwise it returns ..."). The reader-check, dependency and
no-panic statements use "shall" with the system as subject. Several statements
are written as passive or wire-shape assertions with no responsible system and
no "shall".

## Reviewed scope

| Unit | Role | Path | Excerpt |
| --- | --- | --- | --- |
| The link | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | An admitted quire.op.model.reaches application MAY carry one additional key ... The key is absent, never null, when the walked population has no authored bound. |
| The accessor | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | CheckedPackageV2::reaches_population(...) shall return, for the node whose body root is the quire.op.model.reaches application, the selector FR-053 reads. |
| Accessor bullets | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | When the application's operation.member has no population key, the accessor returns Ok(None) |
| Error paragraph | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | An unknown id shall refuse UnknownNode. A node whose body root is not a quire.op.model.reaches application shall refuse NotReachesApplication |
| The reader check | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | check_relationship_member shall check a population key ... The check refuses with existing code and cause pairs only |
| Dependencies | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | The application-node dependency join (application_dependencies) shall add the population node id |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | Some statements name no responsible system, or have no "shall". (1) "An unknown id shall refuse `UnknownNode`" and "A node whose body root is not ... shall refuse `NotReachesApplication`" make the input the subject; the system that refuses is the accessor. (2) "The check refuses with existing code and cause pairs only" and the three refusal bullets have no "shall". (3) In "The link", "The key is absent, never `null`" and "a `navigate` application carries no `population` key" are wire constraints stated as facts, with no reader obligation; the reader obligation sits only in the reader-check bullets. Fix: "When called with an id absent from the graph, `reaches_population` shall refuse `UnknownNode`". Use "the reader shall refuse ..." in the bullets. State the `null` case as a reader obligation, or say which bullet refuses it. A `null` value is "not a node id", so the first bullet does. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3318-3329, spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3357-3368, spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3374-3391 |

## Verdict

PASS WITH FINDINGS. The core statements are conformant EARS (ubiquitous accessor,
event-driven bullets, state-driven reader check). FND-001 is wording only.

## New findings (disposition pass 1)

Reviewed at fffaf6495052405fa07a0a6a145bbbeb079bcfce.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | low | The new link sentence "The V2 reader shall treat the key as absent, never `null`, when the walked population has no authored bound" gives the reader an obligation it cannot meet. The reader cannot know whether the walked population has an authored bound; that is the producer's rule. The sentence also reads as "treat `null` as absent", which contradicts the next clause ("the reader shall refuse it"). Fix: state the producer rule as a wire fact ("the producer omits the key, never writing `null`, when ...") and keep the reader obligation as "the V2 reader shall refuse a `null` `population` as not a node id". | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3321-3324 |

## Dispositions

Round 1, reviewed at fffaf6495052405fa07a0a6a145bbbeb079bcfce.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The error paragraph now reads "When called with an id absent from the graph, reaches_population shall refuse UnknownNode". The reader rules read "the reader shall refuse ...". The `null` case is a reader obligation ("the reader shall refuse it") and AC-272 names it ("a null value is the first"). The new garbled lead clause is FND-002. |

Round 2, reviewed at 444e675529d80ac4561e1f98a0e3ff1aeb96e475.

| FND | outcome | reason |
| --- | --- | --- |
| FND-002 | fixed | The link now separates the producer rule ("The producer omits the key, and never writes null ... owned by the QSL lowering") from the reader obligation ("The V2 reader shall refuse a present population that is not a node id, null included"). |
