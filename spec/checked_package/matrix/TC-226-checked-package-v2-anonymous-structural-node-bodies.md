---
id: TC-226
title: "CheckedPackage V2 refuses a tampered anonymous structural node body"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: verifies
---
# TC-226: CheckedPackage V2 structural nodes and recursion groups

## Description

Verify FR-038's independent closed body, owner, graph, dependency and recursion-group rules under QSpec #517 structural identity. FR-038-AC-123, AC-126, AC-131, AC-148, AC-183 and AC-184 are withdrawn in place; no stale-key or expected-key test survives. Current QSpec fixtures are read directly from `QUIRE_SPECIFICATION_DIR`, never copied.

## Test Procedure

1. Build a package with selected model documents and the ten derived structural shapes: Boolean, Integer, reference, option, set, bag, sequence, ordered set, integer range and collection bounds. Give every node its authored `{package,path,ordinal}` and exact body. The production reader admits every valid shape; the public typed view retains each complete body and id.
2. Mutate one body member, bound, semantic type, owner or reference at a time. A malformed body refuses `invalid_package`/`invalid-derived-shape`; a wrong semantic type, missing selected owner, omitted dependency, unresolved target and wrong referenced form refuse by their respective independent rules. A valid body edit under the same structural id admits if every graph and semantic rule still holds.
3. Build `List`, `Tree` and owner-free G1 groups with producer-assigned structural `node_id.ordinal` values independent of graph position, and separate `{size, ordinal}` group-position evidence in graph order. Include self and internal references, an outside reference and an off-cycle member; verify every member retains its exact structural ID and each in-group reference resolves by group position. Permute the group members while retaining all structural IDs, update only group-position evidence and in-group reference ordinals needed to preserve intended targets, and require admission. In separate copies, duplicate a structural ID, give a structural ordinal an out-of-range or non-integer value, omit a referenced group member, give an in-group reference an out-of-range position or a wrong in-range position naming a type-incompatible member, and break an outside target; require each independent typed graph, reference or wire refusal at its own locus. A valid structural ordinal other than the member's graph position admits.
4. Read every current QSpec positive and adverse fixture through the production reader. Require each published admission or refusal with exact code, cause, pointer and authentic node locus. A missing checkout, fixture, case or wrong outcome fails; no expected-failure entry silently substitutes for a QSpec result.
5. Measure the first unpaid reader work charge in a 100000-node chain and a large recursion group. The exact work setting admits, one unit less returns `Incomplete` at the attempted member or reference with no partial package. A continuation-after-unpaid-charge mutant must fail the observation control. No group digest work exists.

## Expected Results

Every valid package admits with its complete structural ids, bodies, owners and occurrences. Each malformed body, graph, owner, dependency and group mutation reaches its own typed refusal in FR-322 precedence order. A changed valid body is never refused solely because its structural id was retained. Source-owner evidence is selected by exact identity; no caller-supplied document is silently substituted.

## Status

PLANNED/UNRUN (IR-1141). Existing tagged hash-era controls are not evidence for the structural reader until updated and run.
