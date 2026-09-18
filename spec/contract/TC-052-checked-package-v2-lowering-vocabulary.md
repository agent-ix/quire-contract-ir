---
id: TC-052
title: "CheckedPackage V2 lowering vocabulary and outcome selection conform"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: verifies
---
# TC-052: CheckedPackage V2 lowering vocabulary and outcome selection conform

## Description

Verify FR-038-AC-7 and FR-038-AC-8: the closed seven-member lowering record
vocabulary, the unreachability of the two defensive kinds from an admitted
package, and the total order by which one record is selected for one request.

## Test Procedure

Classify every record through a wildcard-free match over the seven kinds, so an
eighth kind fails to compile. Lower every node of both vendored fixtures under a
profile supporting every tag. Produce each of the five reachable kinds
independently. Lower a closure holding both an out-of-profile tag and an
unbounded type; lower an absent key under a zero work limit and under a
sufficient one; lower a request whose first-visited node and least-keyed node
differ and are both offending. Rewrite the scalar and composite semantic form
across every declared form of the two type families that a non-nominal node may
carry, and lower the nominal fixture's enum, unit and dimension nodes under a
bounds-required profile. Read back each lowered record's dependency, bound and
claim lists.

## Expected Results

No admitted package yields `invalid_body` or `body_incomplete`; the five
reachable kinds are each named and distinct. A closure that is both unsupported
and unbounded returns `unsupported`. A zero work limit returns `failed` for an
absent key while a sufficient limit returns `invalid_input` for the same key.
Each named offending key is the least in ascending order, not the first visited.
Exactly the four scalar and four composite unbounded forms return
`requires_bound`, and every other declared form of those families lowers. Every
key in a record's `bounds` and `claims` also appears in its `dependencies`,
which is ascending and excludes the requested node.
