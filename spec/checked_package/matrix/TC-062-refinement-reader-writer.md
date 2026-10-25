---
id: TC-062
title: "Read and write closed checked-package refinement nodes"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-061
    type: verifies
---

# TC-062: Read and write closed checked-package refinement nodes

## Description

Verify [FR-061](../functional/FR-061-admit-refinement-nodes.md) through the
public V2 reader and writer against QSpec FR-375, FR-322 and TC-310's
published refinement forms. Every vector is authored in IR; no QSpec file,
schema or fixture is copied here.

## Test Procedure

Construct an admitted V2 graph with local concrete and abstract model
declarations, selected infinite-trace profile, one population and object
row, operation and stutter steps, history initial and update, and weak and
strong fairness rows. Construct separate protocol-subject, dependency-
package abstract-subject and row-empty controls. Recompute the graph keys,
refinement's `{version, semantic_type, body}` node id and `package_id`
through the repository's production identity path. Read, write and reread
each positive package, checking its body, target joins, dependencies,
canonical array order and occurrences.

Apply the adverse mutations of FR-061-AC-2 through AC-9 one at a time.
Include each step target kind, `open` in and outside argument position, a
local and a directly selected dependency protocol key, wrong local/external
key forms, an unselected package, each row array's missing/extra member,
noncanonical ordering, a history initial of both admitted forms and of an
application, missing/extra dependencies, two row-empty concrete subjects,
proof-eligible and ineligible occurrence sets, and pairs of simultaneous
defects at the earlier/meaning stages. For a missing generated occurrence,
remove its `source_map` entry and assert `/source_map`; for duplicate,
extra-role, extra-ordinal and wrong-region entries, assert the offending
entry's pointer. For within-node precedence, put two wrong-kind targets in
different body members; repeat with two entries of one array and with two
properties of one row entry. Rebuild identities after every body
mutation except the stale-key control; keep the same selected packages in
the identity comparisons.

## Expected Results

Each positive vector admits and round-trips with the same complete node,
node id and package id. Every adverse vector refuses the entire package with
FR-061's code, cause and pointer; no row is defaulted, reordered, omitted or
interpreted under another form. The two concrete subjects produce distinct
node ids despite equal bodies, and only the canonical meaning-bearing edits
change identity. Earlier reader stages win over later refinement joins.
The simultaneous body defects report the first member in FR-061's fixed
order, then first array index and first schema-declared entry member.
