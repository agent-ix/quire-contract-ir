---
id: ADR-0054
title: "Separate archetype datatype generation from formal type projection"
type: ADR
status: accepted
owner: kreneskyp
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-013
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-023
    type: depends_on
---
# ADR-0054: Separate archetype datatype generation from formal type projection

## Status

**Accepted boundary clarified by the owner on 2026-09-08.** This decision
corrects the premise of
[issue #54](https://github.com/agent-ix/quire-contract-ir/issues/54) before
implementation. It authorizes no change to `filament-core-data`.

## Context

`filament-core-data` is schema-definition and code-generation tooling for
building Quire module/plugin archetypes. A module can define an archetype such
as a domain object and emit usable datatypes for consumers such as
`filament-daemon`, which parses artifacts and architectural objects.

Those generated datatypes describe artifact structure. They do not by
themselves define the meaning of a formal clause, prove a specification
correct, or own Contract IR semantics. The `agent-ix-semantic-ir` crate is an
internal implementation detail of `filament-core-data`; its current existence
does not make it a Contract IR dependency or a formal authority.

Formalization may nevertheless need a model of selected archetype-defined
types. For example, a clause over `ConfigVersion.versionNumber` cannot be
typechecked without knowing the field's value domain. The legitimate boundary
is therefore an explicit projection from the relevant structural schema facts
into Contract IR's formal type system. It is not wholesale adoption of the
archetype compiler's model or emitted host-language datatype.

Contract IR already owns its closed formal type system in FR-013 and its
validated executable projection boundary in FR-023. A frontend can construct a
`DeclarationEnvironment` through the public Rust API and lower supported,
bounded source expressions into that environment without a general Filament
package reader.

## Decision

Archetype schema definition and datatype generation remain outside Contract IR
formal semantics. Generated Rust names, layouts, integer widths, option types,
and collection containers shall not be interpreted as proof semantics.

When a formal clause needs an archetype-defined type, the specification-language
frontend or a separately reviewed adapter may project the exact required schema
closure into FR-013 declarations. That projection shall:

- identify the source schema and exact declaration revision;
- preserve only structural facts that have an explicit formal correspondence;
- provide every finite bound, absence rule, identity rule, observation rule,
  and relationship interpretation required by the admitted Contract IR type;
- refuse unsupported, ambiguous, unbounded, recursive, or semantically partial
  constructs rather than inventing a representation; and
- retain enough provenance to diagnose the originating authored declaration.

The projection is part of the formalization boundary and must be specified and
qualified as such. Successful archetype generation is neither evidence that
the projection is sound nor evidence that a specification is correct.

The modeling language or Quire module that owns an archetype also owns the
meaning of that concept. A `Domain` archetype may be only an organizational
container and need no executable projection. A `StateMachine` archetype can be
relevant to reachability or invariant proofs, but its schema-generated datatype
is only the source representation. Its modeling layer must define states,
initialization, transitions, guards, actions, and observation semantics before
a qualified adapter can lower the applicable finite subset. Contract IR then
receives finite declarations, relations, clauses, and anchors; it need not add
public `Domain` or `StateMachine` variants merely because those archetypes exist.

Contract IR itself remains producer-neutral. It accepts its own public
`DeclarationEnvironment` and executable projection contracts; it does not add
a public `SemanticModelPackage`, mirror every Filament construct, or link and
qualify Filament's internal reader. A concrete adapter may consume a stable
schema artifact as data, but its correctness obligations belong to that
adapter and its specification-language owner.

### `ConfigVersion` consequence

The issue #54 example demonstrates why structural schema and formal semantics
must remain distinguishable:

- `versionNumber` with only `min 1` lacks the finite maximum required by
  FR-013;
- `parent: ConfigVersion[0..1]` is an optional identity-bearing reference, not
  an optional recursively contained record;
- generated Rust field types, host integer widths, observed values, and a
  runtime object store do not fill either semantic gap.

A frontend can parse and retain those declarations, but the current Contract IR
must refuse their executable projection. A future reviewed profile could add a
finite bound or identity/reference model if a real proof requires it. The need
must come from that proof case, not from the mere existence of the generated
datatype.

## Consequences

- Agent A can use FR-013, FR-019, and FR-023 immediately for the generic source
  language and need not wait for `filament-core-data#36` or a new model layer.
- A concrete archetype-to-formal projection is deferred until a current clause
  demonstrates that it is needed; it is not part of the initial language
  parser/compiler gate.
- The modeling language that owns a semantic concept owns its lowering profile;
  `filament-core-data` continues to own only schema/datatype generation.
- The internal `agent-ix-semantic-ir` crate may be renamed, replaced, or removed
  later by its owning repository without changing Contract IR.
- Any future reference, population, unit, or other domain-specific semantics
  require explicit Contract IR requirements and correspondence tests.

## Rejected alternatives

- **Treat generated datatypes as the formal type authority.** Rejected because
  host-language structure omits or distorts formal bounds, identity,
  observation, absence, and relationship semantics.
- **Copy the complete archetype model into Contract IR.** Rejected because it
  creates a second archetype authority without making every construct
  executable.
- **Forbid all use of archetype schema facts in proofs.** Rejected because some
  clauses legitimately need a formal model of the datatype they constrain.
- **Block the generic source language on a concrete Filament adapter.** Rejected
  because parsing, generic typechecking, and lowering against FR-013 can proceed
  independently; only clauses needing that adapter should be gated by it.

## Follow-up disposition

Issue #54 should record this corrected boundary and stop blocking Agent A. If a
concrete specification clause needs an archetype-defined type that cannot be
expressed through FR-013, its owner shall file the smallest specific projection
or Contract IR semantic gap. No general Filament semantic-model reader is
authorized by this decision.
