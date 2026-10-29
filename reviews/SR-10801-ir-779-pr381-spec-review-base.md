---
id: SR-10801
title: 'base review of IR-779 PR #381'
type: SpecReview
analysis: base
scope: agent-ix/quire-contract-ir; spec/checked_package/functional/FR-050-admitted-scalar-path-guard-context.md;
  spec/checked_package/matrix/TC-450-admitted-scalar-path-guard-context.md; spec/checked_package/matrix/tests.md
review_set: subset
---

## Summary

Reviewed IR-779 PR #381 at the public PR head (same tree as the local checkout) against IR main and QSL main. Two medium findings.

## Verdict

**CONDITIONAL** — two medium findings need correction.

## Reviewed Scope

- `FR-050` (examined, `spec/checked_package/functional/FR-050-admitted-scalar-path-guard-context.md`): When a driver requests a scalar proof for one QSL-checked requirement occurrence, Contract IR SHALL expose its ordered path guards only after validating their Boolean expressions against the same admitted `ContractPackageV1` as the application result. QSL owns the checked `RequirementItem::path_condition()` sequence of `PathGuard { occurrence, holds }`, outermost first. The driver owns faithful per-item mapping of that sequence and the context-only lowering roots. Contract IR cannot infer whether a represented guard belongs to an application occurrence from the FR-330 package, a shared express
- `FR-050-AC-1` (examined, `spec/checked_package/functional/FR-050-admitted-scalar-path-guard-context.md`): For QSL FR-062 RR-13, the checked `rational.div` item supplies the one `y != 0` guard with `holds = true`; after one lowering over application, any selected conversion, and guard roots, the admitted query returns one Boolean expression reference with that exact occurrence, polarity, source package, profile and source-map entry. For a checked else-path item over the same guard expression, `holds = false` is retained, and an empty checked path returns an empty list.
- `FR-050-AC-2` (examined, `spec/checked_package/functional/FR-050-admitted-scalar-path-guard-context.md`): Two checked items sharing one application expression node but carrying different ordered guard occurrences retain separate lists and polarities even when all roots are represented in one package. An item with two nested guards returns them outermost first. Repetition of a node key does not collapse distinct occurrences. The query does not infer a guard association from the shared node or source regions.
- `FR-050-AC-3` (examined, `spec/checked_package/functional/FR-050-admitted-scalar-path-guard-context.md`): A guard whose requested root is absent, non-lowered, or absent from the represented graph; whose typed key, `expression` role or ordinal lacks an exact admitted source-map entry; whose represented node is not an expression or is not Boolean; or whose admitted dependency/body is inconsistent returns the corresponding typed located cause for the first failing guard, with no returned prefix, substitute assumption or panic. A bare application-only package refuses a supplied guard key even when its source key is otherwise plausible.
- `FR-050-AC-4` (examined, `spec/checked_package/functional/FR-050-admitted-scalar-path-guard-context.md`): Mutating only the supplied guard key to one with no admitted expression cannot produce a successful assumption. Mutating only `holds` on an admitted Boolean guard changes only the returned polarity; mutating its ordinal or role to an unrepresented occurrence refuses. An independently valid represented guard substituted from another item's list is structurally valid in IR, so only the QSL checked item and faithful driver mapping establish its membership.
- `FR-050-AC-5` (examined, `spec/checked_package/functional/FR-050-admitted-scalar-path-guard-context.md`): Every guard root has an ordinary FR-035 accounting record. The query returns references from one reader-admitted six-field FR-330 package and changes neither its wire/identity nor the application's forward closure. Added roots may change that package's graph, accounting and canonical identity. FR-049's application plus optional direct-narrow result outcomes remain unchanged when guards are also represented.
- `TC-450` (examined, `spec/checked_package/matrix/TC-450-admitted-scalar-path-guard-context.md`): Check QSL FR-062 RR-13 and an else-path counterpart. Record each checked item's application occurrence and outermost-first `PathGuard` values before mapping them to typed IR keys. Lower each application, optional direct conversion, and guard expression as roots in one selected source package and profile. Admit the resulting ordinary FR-330 package, query each item with its own mapped list, and compare every returned occurrence, `holds`, Boolean type, source entry, package and profile with the independent checked inputs. Query an unguarded item with an empty list. Build a nested two-guard case
- `FR-050-matrix` (examined, `spec/checked_package/matrix/tests.md`): | FR-050 | FR-050-AC-1 through FR-050-AC-5 | TC-450 | 🚧 PLANNED/UNRUN (IR-779). The typed admitted guard-context query and its tagged IR tests have not been implemented. QSL's checked `RequirementItem::path_condition()` is available on its current main; driver mapping/context roots and CG assumption rendering are separate consumer work. |
- `TC-450-matrix` (examined, `spec/checked_package/matrix/tests.md`): | TC-450 | Admitted scalar path guards retain checked occurrences and refuse unsupported assumptions | Property | P0 | FR-050-AC-1, FR-050-AC-2, FR-050-AC-3, FR-050-AC-4, FR-050-AC-5 | 🚧 PLANNED/UNRUN (IR-779); no tagged implementation tests or consumer acceptance evidence yet. |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | TC-450 assigns a lowered-but-unrepresented guard root to the public admitted-package query. FR-035/FR-038 admission requires every lowered root to be represented, so this control cannot reach that public query. Put the malformed root at the private defensive seam or assert reader refusal at admission. | spec/checked_package/matrix/TC-450-admitted-scalar-path-guard-context.md:30 |
| FND-002 | medium | FR-050 promises backend rendering and shared symbolic bindings for every admitted Boolean guard, but the IR view exposes only node references and TC-450 only compiles a consumer. IR admission cannot establish CG renderability or binding; state these as downstream CG obligations with typed refusal for unsupported guards and leave IR acceptance to structural references. | spec/checked_package/functional/FR-050-admitted-scalar-path-guard-context.md:47 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | TC-450 now tests a missing represented node as an FR-330 reader refusal and keeps post-admission inconsistency at the private seam. |
| FND-002 | fixed | FR-050 now limits its output to structural admission and polarity; CG IR-304 explicitly owns rendering, shared symbolic binding and typed unsupported refusal. |
