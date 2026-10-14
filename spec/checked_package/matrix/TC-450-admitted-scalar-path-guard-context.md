---
id: TC-450
title: "Admitted scalar path guards retain checked occurrences and refuse unsupported assumptions"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-050
    type: verifies
---
# TC-450: Admitted scalar path guards retain checked occurrences and refuse unsupported assumptions

## Description

Verify [FR-050](../functional/FR-050-admitted-scalar-path-guard-context.md)
through the public Contract IR lowering and admitted-package query, with
independently known checked QSL guard occurrences and polarities.

## Test Procedure

Check QSL FR-062 RR-13 and an else-path counterpart. Record each checked
item's application occurrence and outermost-first `PathGuard` values before
mapping them to typed IR keys. Lower each application, optional direct
conversion, and guard expression as roots in one selected source package and
profile. Admit the resulting ordinary FR-330 package, query each item with
its own mapped list, and compare every returned occurrence, `holds`, Boolean
type, source entry, package and profile with the independent checked inputs.
Query an unguarded item with an empty list. Build a nested two-guard case and
two items sharing one expression node but carrying different guard occurrence
lists; assert their exact order and separate polarities.

For each public-query refusal class in FR-050-AC-3, change exactly one guard
input or context-root disposition in an otherwise reader-admitted package.
Include absent and non-lowered roots; missing, wrong-key, wrong-role and
wrong-ordinal source-map occurrence; a represented non-expression node; and a
represented expression of non-Boolean type. Supply two guards with the second
invalid and assert no first-guard prefix is returned. Query a bare
application-only package with a guard key that was never lowered. As a
mutation control, supply a plausible guard key with no admitted expression
and assert that no IR guard reference is returned. Separately, mutate an
FR-330 wire to claim a lowered guard root without representing its node and
assert reader refusal before any public query. At the private defensive seam,
mutate an admitted body/dependency after admission and assert an existing
typed view error without a partial list.
Flip only `holds` for a valid guard and compare the returned polarity. Supply
a different, independently valid represented guard to document IR's
structural boundary: that input may pass IR and therefore cannot establish
QSL item membership without the checked driver mapping.

Compare the six-field wire, canonical identity and application
`reached_closure` before and after each read-only query. Compare the
application-only package with the contextual package to observe the ordinary
root-driven graph/accounting/identity change. Compare FR-049 result-context
queries with and without represented guard roots. Compile a public consumer
that obtains structural guard references and result context from the same
admitted package and does not read the original QSL graph. CG IR-304 separately
tests rendering, shared symbolic binding and typed refusal of unsupported
guards; this IR procedure asserts none of those backend behaviors.

## Expected Results

Positive queries expose exactly the checked guard list with Boolean
expressions, source correspondence, package/profile and unchanged order and
polarity; the unguarded list is empty. Every invalid guard refuses with the
typed first failing cause and location, without a partial list or substitute
reference. Reader-invalid packages refuse before the public query, and the
private post-admission mutation returns its existing typed view error. A key
with no admitted expression never succeeds. Context-only roots have ordinary
FR-035 accounting records. Queries leave
the package and forward closure unchanged, and FR-049 keeps its existing
application and optional direct-narrow outcomes.
