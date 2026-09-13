---
id: ADR-0053
title: "Native Quire source authority and fail-closed mapping profiles"
type: ADR
status: accepted
owner: kreneskyp
relationships:
  - target: ix://agent-ix/quire-contract-ir/PGM-01
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-012
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-013
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-014
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-015
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-016
    type: depends_on
---
# ADR-0053: Native Quire source authority and fail-closed mapping profiles

## Status

**Accepted as amended on 2026-09-09.** The
[owner ruling on #53](https://github.com/agent-ix/quire-contract-ir/issues/53#issuecomment-5611950216)
rejects the earlier OCL-first recommendation and selects native Quire as the
sole editable formal-clause source language. The
[emission clarification](https://github.com/agent-ix/quire-contract-ir/issues/53#issuecomment-5611958246)
retains OCL 2.4, SysML v2/KerML and FRETish as output-only mapping targets. The
[closed-#54 correction](https://github.com/agent-ix/quire-contract-ir/issues/53#issuecomment-5612031500)
keeps accepted ADR-0054 and its completed binding work intact.

This ADR records source authority and fail-closed boundaries. It does not claim
that every Quire profile, Contract IR projection, backend, mapping, or
qualification is implemented.

| Decision field | Accepted state |
| --- | --- |
| Source authority | Native Quire, source identity `ix:native`; exactly one editable authority per clause |
| Semantic profiles | Selected and versioned by the Quire standard independently of IR/backend support; historical definitions retain their meanings |
| Output mappings | OCL 2.4, SysML v2/KerML and FRETish through Rust emitters with explicit loss/refusal results |
| Implementation boundary | First-party compiler, reader, writer, bridge, test and qualification code is Rust |
| Foreign runtimes | No Java, JVM, Maven, Eclipse, Node or Electron production/test/CI/qualification dependency without another owner decision |
| Qualification | Per-stage and per-capability; parser acceptance, IR representation and backend support are never interchangeable evidence |

## Context

Archetype schema definition and datatype generation remain outside formalization.
Native Quire consumes exact model declarations and binds supported checked
expressions to Contract IR. A field described only as `int`, `Dict[str, Any]`,
a generated host type, or a sample value does not supply the finite semantic
domain required by the formal language.

FR-013 has no identity-bearing object reference or recursive record type, and
FR-014 has no temporal expression node. A source spelling cannot create either
capability. Object/graph and temporal work therefore use separately versioned
contracts rather than approximating them in the scalar IR.

## Decision

### One source language and one authority

Native Quire is the only editable formal-clause language. A clause has one
stable identity and one authored source: one language-tagged inline region or
one exact local-file region, never both. The source selection binds exact bytes,
SHA-256, source revision and half-open byte span. A path or floating URL is not
an identity. Multi-clause files bind every clause separately.

Duplicate authorities, stale bytes, ambiguous names and conflicting compiled
declarations refuse before lowering. Generated OCL, SysML, FRETish, schema and
solver text are read-only derivations. Equal derived expression bytes cannot
erase a change in source language/profile, source bytes, model closure, anchor,
tool, or backend selection.

### Meaning and capability are separate axes

A request reports these stages independently:

1. source recognized;
2. exact language edition/profile definition selected;
3. declarations linked and expression statically admitted;
4. runtime inputs validated and native evaluation available;
5. requested lowering available and exact;
6. selected backend supported;
7. claimed qualification evidence available.

Failure at a stage prevents only its dependent output. A recognized but
unadmitted construct receives a located `unsupported_construct`; an admitted
native subject with an unavailable projection/backend receives its typed
unsupported result. Neither can produce partial successful IR, a fabricated
Boolean, an empty collection, or an approximate mapped artifact.

The Quire standard owns profile definitions. The composed-v1 candidates use
`ix:native` edition `1-draft` with independently versioned state-core, query,
finite-graph, temporal, protocol, observation, package and diagnostic
definitions accepted for implementation planning in Quire Specification PR
#15. FS01–FS05 own final adoption and immutable definition pins. Historical
`ix:native` / `0-draft` / `state-finite/0-draft` source, result and definition
bytes remain historical and are never reinterpreted in place.

The released quire-spec-language `v0.2.0` baseline implements its stated
bounded-scalar/compiler scope. That implementation evidence does not define the
language or claim complete object, graph, collection, temporal, protocol or
output-mapping support.

### Bounded admission firewall

The frontend enforces the selected native profile before Contract IR lowering.
Every bound comes from an exact reviewed domain declaration or named profile
parameter, never host width or observed values.

| Concern | Native admission/refusal rule | Contract IR boundary protected |
| --- | --- | --- |
| Integer and rational domains | Finite inclusive signed-64 integer bounds; exact rational sign/GCD normalization before authored numerator/denominator bounds; no float, saturation, widening or implicit conversion | FR-013 numeric values; FR-014 operand types; FR-015 checked definedness |
| Arithmetic | `+`, `-`, `*`, unary negation and rational division only with exact range/nonzero proofs; integer `/`, `div`, `mod` and `rem` refuse | FR-014 closed operator meanings; FR-015 range/nonzero obligations |
| Presence/null/invalid | Preserve independently authored absence and explicit null through a qualified encoding or refuse; invalid is never false or option-none | FR-013 option; FR-015 definedness; FR-014 Boolean root |
| Records and identity | Acyclic typed value containment; no structural substitute for object identity | FR-013 closed acyclic types; FR-014 field/equality ownership |
| References | No scalar-to-object cast, ambient store, recursive inline parent or inferred foreign-key navigation | FR-012 identity; FR-013 closed types; later versioned graph/reference contract |
| Collections | Explicitly ordered, duplicate-preserving finite sequences with authored maximum at most 10,000; unsupported kinds/conversions refuse | FR-013 bounded sequence; FR-014 ordered quantifier domain; FR-016 sequence identity |
| Calls and effects | No unselected calls, recursion, reflection, I/O or ambient library; a signature alone supplies no semantics | FR-013 pure declarations; FR-014 exact call signature |
| Root and control | Exactly one statically defined Boolean root with selected native short-circuit meaning; no truthiness, nullable root or exception recovery | FR-014 Boolean result; FR-015 exact-subject guard facts |
| Resources | Charge selected parser/checker/runtime limits before work; exhaustion is incomplete and emits no partial result | FR-014/FR-019 aggregate bounds; PGM-01 result integrity |

All declarations and syntax are checked, including unused declarations and
unselected branches. A wider IR or backend does not admit a source construct.
Conversely, a narrower backend does not change an admitted native construct's
meaning.

### Evaluation anchors

Every executable clause binds one exact operation or named observation. A bare
context type is insufficient.

| Native clause | `self` | Parameters | Result | `pre(expr)` |
| --- | --- | --- | --- | --- |
| Invariant | Named current observation | unavailable | unavailable | refused |
| Operation precondition | Exact invocation pre snapshot | immutable declared inputs | unavailable | refused |
| Operation postcondition | Same invocation post snapshot | same immutable inputs | declared post-only result | eligible reads from that invocation's pre snapshot |

`pre(expr)` selects eligible state reads; it does not retag a captured post
value, replace a parameter with a field, or reuse a guard from another
observation. Missing/foreign anchors, `pre(parameter)`, `pre(result)`,
cross-observation guard reuse and mismatched operations refuse before lowering.
Exact alias and composite-expression behavior is the Quire standard's
state-contract authority, not an OCL `@pre` rule inferred by similarity.

### ConfigVersion gate and slices

The original `parent: ConfigVersion[0..1]` and an Integer declaring only `min 1`
remain a model/reference gate. A consumer must not invent a maximum, erase the
relationship, inline a recursive record, or cast an opaque UUID into an object.
The finite identity-preserving graph profile is a separate extension; it does
not retroactively qualify that original model.

| Slice | Native authority | Current disposition |
| --- | --- | --- |
| Positive version invariant | Selected state profile over an explicitly finite authored domain | The released 0..1000 scalar corpus has native/oracle/proptest/Kani agreement; the original minimum-only domain remains unqualified |
| Version unchanged across an operation | Native postcondition with exact pre/post invocation anchors | Released scalar example is evidence only for its exact selected model and clause |
| Persisted within a bounded interval | Native temporal profile with explicit trigger, clock, history and progress/closure authority | Separate temporal/TL/observation path; no scalar or FRETish approximation |
| Parent ordering/navigation | Selected finite identity-preserving reference universe and relationship mapping | Unsupported until the exact model/reference contract and backend path are selected |

### Output mappings

OCL 2.4, SysML v2/KerML and FRETish are FS06 output targets. The retained names
`ocl24-bounded-v1`, `sysml2-bounded-invariant-v1` and
`fretish-bounded-ticks-v1` identify mapping contracts, not source profiles.

Each Rust emitter accounts for every selected native obligation as preserved,
conditional, unrepresented or refused; preserves source/result identity; and
returns no successful partial artifact after a loss that the target contract
does not represent. Its oracle is the native meaning plus the explicit mapping
contract and independent expected corpus. An upstream parser accepting emitted
text is not proof of semantic correspondence.

External tool/version/license investigation remains useful provenance for the
mapping tickets but is not a first-party execution prerequisite. The complete
pre-amendment proposal, including Eclipse OCL, NASA FRET and SysML pilot pins,
is preserved at
[Contract-IR `39bffb4`](https://github.com/agent-ix/quire-contract-ir/blob/39bffb40f41b7caceaf026f438546b15bf140ce4/spec/decisions/ADR-0053-formal-clause-source-profiles.md).
The recorded SysML pilot correction remains LGPL-3.0-or-later.

### Eligibility and extension points

A criterion is eligible for formalization only when its author can identify a
precise property, finite typed domain, exact evaluation boundary and supported
native profile without changing its meaning. Ineligible or not-yet-supported
criteria retain their Test, Inspection, Analysis or Demonstration method and an
explicit disposition. Eligibility is not authorship, parsing, lowering, proof,
or qualification. Coverage work #58 preserves the whole criterion population
and every per-stage status.

New native operators, types, semantic families, mappings and backends are
separate versioned extension points. Each extension declares its syntax/profile
selection, types and definedness, source/canonical identity, resource bounds,
compatibility/migration, unsupported behavior and independent evidence. No
plugin, installed backend or foreign syntax extends the language implicitly.

## Consequences

- Contract IR consumes checked native subjects and reports exact capability; it
  neither parses an alternate source language nor decides native meaning.
- ADR-0054 stays accepted and #54 stays closed. Later reference/graph work uses
  a new versioned contract rather than reopening the archetype boundary.
- #55–#57 own Rust output mappings; #58 owns language-neutral coverage states.
- Full Quire v1, a particular backend, and a public release remain separate
  claims with their own evidence and owner decisions.
- OCL-first, SysML-first, FRETish-first, multiple editable authorities,
  automatic EARS formalization and solver text as source are rejected.

## Owner decision record

Owner: `@kreneskyp`. Decision: **amend and accept native Quire as source profile
one; retain OCL/SysML/FRETish as output mappings**. Recorded on #53 on
2026-09-09 with the emission/dependency and closed-#54 clarifications. This
acceptance does not approve public promotion, third-party incorporation,
application migration, a new external runtime, or complete native/backend
qualification.
