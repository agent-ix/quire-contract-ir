---
id: FR-050
title: "Expose admitted scalar path-guard context for an exact requirement occurrence"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/FR-035
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-049
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: references
  - target: ix://agent-ix/quire-specification/FR-330
    type: references
  - target: ix://agent-ix/quire-spec-language/FR-062
    type: references
  - target: ix://agent-ix/quire-spec-language/FR-075
    type: references
---
# FR-050: Expose admitted scalar path-guard context for an exact requirement occurrence

## Description

When a driver requests a scalar proof for one QSL-checked requirement occurrence,
Contract IR SHALL expose its ordered path guards only after validating their
Boolean expressions against the same admitted `ContractPackageV1` as the
application result. QSL owns the checked `RequirementItem::path_condition()`
sequence of `PathGuard { occurrence, holds }`, outermost first. The driver owns
faithful per-item mapping of that sequence and the context-only lowering roots.
Contract IR cannot infer whether a represented guard belongs to an application
occurrence from the FR-330 package, a shared expression key, or source regions.

## Inputs

One reader-admitted FR-330 `ContractPackageV1`; the exact QSL-checked
application occurrence and expression key for one routed item; the exact
ordered QSL-checked guard sequence, each mapped to IR's typed
`ContractOccurrenceKeyV1` and Boolean `holds`; and the source package name
and lowering profile selected for the application's IR lowering. Each guard
occurrence retains its `expression` role and ordinal. The caller supplies
guard expression nodes as additional explicit lowering roots for that same
call. An empty checked sequence is valid and yields an empty guard list.

The guard sequence is supplied at the read-only query boundary. The original
QSL graph is not a query input.

## Outputs

For a lowered application, a typed view containing its exact admitted source
package, selected lowering profile, application occurrence and an
outermost-first list of guard references. Each reference carries the supplied
typed occurrence and polarity, its represented Boolean expression key and
node, and that node's admitted source-map correspondence. This IR view asserts
structural admission and polarity only; it does not establish renderability,
a shared symbolic variable assignment, or
an executable assumption. The view contains no rendered assumption or second
graph.

An absent or non-lowered application retains its exact [FR-035](./FR-035-complete-v1-contract-package-lowering.md)
request disposition. A guard-validation failure returns one typed, located
cause and no guard list or partial application result. The cause distinguishes
absent guard root, non-lowered guard root and its disposition, missing or
mismatched source-map occurrence, wrong occurrence role or ordinal,
non-Boolean expression, and wrong node form. A lowered root absent from the
represented graph cannot pass FR-330 admission; the reader refuses it before
this public query. A private admitted-then-mutated defensive check may return
an inconsistent-package/body error. The failing guard's supplied occurrence
and list position are available to the caller; when a matching admitted source entry exists, its
locus is retained. Error variants and typed fields, not formatted messages,
define the API.

## Behavior

### Context roots and admission

When the driver lowers one checked item, it SHALL include the application,
its optional direct conversion from [FR-049](./FR-049-admitted-scalar-result-context.md),
and every guard expression as explicit roots of the same FR-035 lowering call.
Guard roots are context-only: each receives its own ordinary feature-accounting
record and represented source correspondence. The driver owns keeping those
roots out of its routed item and claim lists; CG owns proof-obligation creation.
Shared node keys may be deduplicated
in the one represented graph; distinct source occurrences and supplied order
remain distinct in the query. The ordinary forward `reached_closure` of the
application does not acquire enclosing guards as reverse consumers. Adding
guard roots may change the represented graph, accounting and canonical
bytes while retaining the package name, as FR-035 requires. The output remains one ordinary five-member
FR-330 package, with no added field or copied QSL schema.

### Admitted query

Contract IR SHALL first verify the supplied application occurrence against
the admitted source map, its requested expression key and lowered record, as
[FR-049](./FR-049-admitted-scalar-result-context.md) does. For every guard in
the supplied sequence, Contract IR SHALL require a `lowered` feature-accounting
record for its exact typed node key; a represented local expression node; an
exact admitted source-map entry matching key, `expression` role and ordinal;
and an admitted `Boolean` semantic type. It SHALL validate the guard's
represented body and dependencies using the existing IR reader and reached
view, including selected external dependency identity. A guard key without a
represented expression is never proof authority. A source-map entry for a
different node, role or ordinal cannot satisfy the supplied occurrence.

The query SHALL preserve the supplied order and polarity exactly, including
two occurrences of the same expression key with different paths or polarities.
It SHALL NOT sort, deduplicate, change `false` to `true`, infer guards from
source regions, or discover them from represented nodes. Contract IR cannot
detect a swap between two separately valid represented guards of the same
item or between items without a QSL-visible binding witness; the QSL checked
record and driver's per-item mapping are authoritative for membership and
order. A caller-created list alone is not evidence that QSL checked those
guards. A consumer SHALL use this view only with that faithful mapping.

If any guard fails a check, Contract IR SHALL return the first failing guard
in supplied order with its typed located cause, without exposing a prefix or
substitute reference. A package admission failure, including a lowered root
without a represented node, is returned before the query and is never
repaired by a supplied guard list. A post-admission body inconsistency or
exhausted work limit SHALL return the existing typed view error without a
partial result. Querying SHALL leave all five FR-330 fields,
complete canonical value, selected profile and application forward closure unchanged.
The optional direct-narrow rule of FR-049 remains application plus optional
conversion only; guard roots and this guard query do not alter its selection
or result-context outcomes.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-050-AC-1 | For QSL FR-062 RR-13, the checked `rational.div` item supplies the one `y != 0` guard with `holds = true`; after one lowering over application, any selected conversion, and guard roots, the admitted query returns one Boolean expression reference with that exact occurrence, polarity, source package, profile and source-map entry. For a checked else-path item over the same guard expression, `holds = false` is retained, and an empty checked path returns an empty list. | Test |
| FR-050-AC-2 | Two checked items sharing one application expression node but carrying different ordered guard occurrences retain separate lists and polarities even when all roots are represented in one package. An item with two nested guards returns them outermost first. Repetition of a node key does not collapse distinct occurrences. The query does not infer a guard association from the shared node or source regions. | Test |
| FR-050-AC-3 | The public guard query returns the corresponding typed located cause for the first failing guard, with no returned prefix, substitute reference or panic, for these inputs against a reader-admitted package: an absent or non-lowered guard root; a typed key, `expression` role or ordinal lacking an exact admitted source-map entry; a represented non-expression node; or a non-Boolean expression. An application-only package refuses a supplied guard key lacking its lowering root despite a plausible source key. A malformed FR-330 package claiming a lowered guard root but omitting its represented node refuses at admission; a private admitted-then-mutated body/dependency inconsistency returns the existing typed view error without a partial list. | Test |
| FR-050-AC-4 | Mutating only the supplied guard key to one with no admitted expression cannot produce a successful IR guard reference. Mutating only `holds` on an admitted Boolean guard changes only the returned polarity; mutating its ordinal or role to an unrepresented occurrence refuses. An independently valid represented guard substituted from another item's list is structurally valid in IR, so only the QSL checked item and faithful driver mapping establish its membership. | Test |
| FR-050-AC-5 | Every guard root has an ordinary FR-035 accounting record. The query returns references from one reader-admitted five-member FR-330 package and changes neither its wire, package name nor the application's forward closure. Added roots may change that package's graph, accounting and complete canonical value. FR-049's application plus optional direct-narrow result outcomes remain unchanged when guards are also represented. | Test |

## Dependencies

[FR-035](./FR-035-complete-v1-contract-package-lowering.md) owns the one
lowering and reached closure; [FR-038](./FR-038-consume-checked-package-v2.md)
owns checked-package/body admission; [FR-049](./FR-049-admitted-scalar-result-context.md)
owns the independent scalar result-context selection. QSpec
`ix://agent-ix/quire-specification/FR-330` owns the package wire. QSL
`ix://agent-ix/quire-spec-language/FR-062` owns checked path conditions, and
`ix://agent-ix/quire-spec-language/FR-075` owns the routed item. The driver
carries the exact guards and roots. CG IR-304 owns rendering a supported
guard and its polarity as an assumption with the same symbolic variable
assignment as the application, refusing unsupported guard forms or bindings
with a typed cause, and binding occurrence-aware proof identity. IR's
structural reference alone does not discharge those CG obligations.
