---
id: TC-226
title: "CheckedPackage V2 refuses a tampered anonymous structural node body"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: verifies
---
# TC-226: CheckedPackage V2 refuses a tampered anonymous structural node body

## Description

Verify the owner-free ungrouped criteria FR-038-AC-123 through FR-038-AC-135
and the closed-shape part of AC-183, retaining each criterion's own baseline
status, together with the
PLANNED, UNRUN IR-630 extension of AC-127, AC-131 and AC-145 through AC-150.
IR-630 authenticates every structural node's actual owner, declaration and
body, then every labelled group's digest. It replaces the skip criteria and
explicitly retires AC-146's ordinary-key-plus-lone-label admission and
AC-147's ordinary-key-plus-off-cycle-same-label admission.

The reader's groups contain all nodes sharing a retained label, with ordinals
from graph array position among those members. QSL's names-graph SCC and
content order describe its producer; this test adds no reader SCC or content
order rejection. Existing owner/schema, graph-shape, application, nominal and
abstraction identity stages retain their own responsibilities. Published
fixtures are read at their authoritative checkout, never copied into this
repository. Existing trace tags or residual-admission controls do not establish
any new IR-630 refusal.

## Test Procedure

Over a lock selecting a domain package document built here (an object type
with fields declared `Int[0, 1000]`, `Int[0, 0]`, the `i128` extremes,
`Option<Int[0, 1000]>`, `Set<Int[0, 1000]>`, `Sequence<Int[0, 1000]>`, a
bounded collection and a `Reference`, and an operation with a parameter typed
`Int[0, 1000]`), build a checked package whose reads name each type node, a
scalar operand that reaches an `Int[0, 1000]` node without a member read, and a
state field whose body names one. Admit the unmutated package. Then apply each
single mutation the criteria name (a body value, a body member, a
`semantic_type`, a literal `type`, a body `reference`), keep the `node_id`,
patch `identity_projection`, recompute `package_id` through `quire-canonical`
in the test and not through the reader, and read each mutated package. Also
read the packages of AC-127 that pair a tampered node with another defect, and
the AC-129 package whose `model_selections` row is re-pointed, with and
without the other document in the evidence. Read the three QSpec positive
fixtures of FR-038-AC-107 and the in-repo fixture packages with their derived
keys (admit) and with one placeholder key restored on an undeclared
derived-shape node (`stale-node-key`); read the in-repo `aaaa` and `bbbb`
nodes with keys regenerated but bodies and `semantic_type` unmigrated
(`stale-node-key`); and apply `adverse.json` (eleven entries) and
`dependency-selection-vectors.json` over `positive-all-families.json` carrying
derived keys (AC-133 through AC-135). The QSpec fixtures are read
from the checkout `QUIRE_SPECIFICATION_DIR` names, never copied.

For the IR-630 controls, build correctly keyed `List` and `Tree` packages
under joined source owners, and the genuine owner-free G1 self-option group.
Keep the authoritative external golden controls at their owning checkout.
Read all positives through the production reader. Then apply one mutation at
a time, retain the affected old keys, mirror `identity_projection` and refresh
`package_id` independently through `quire-canonical`:

- Change `Tree`'s collection bounds, its bounded node's `semantic_type`, or
  its sequence body's target to another member (AC-148).
- Start with a correctly keyed ungrouped `Option<Integer>`; replace its body
  reference by its own unchanged node id and add a label. This forgedSelfOption
  is a real wire self-cycle with an old key, not a mocked validation result.
- Repeat the former bounded-set/collection-bounds forged cycle and the
  `integer_range` retag mutation under old keys (AC-148).
- Add a lone label to an otherwise unchanged ordinary-key range, Boolean,
  Integer or reference node (AC-146).
- Add an off-cycle node with the genuine `Tree` label and its unchanged
  ordinary ungrouped key (AC-147). Count it among the labelled members.
- Change a joined owner to another valid selected owner while retaining keys;
  move the declaration source regions coherently so the key check is reached.
- Permute labelled members while retaining their keys. Separately recompute
  every key, label, reference, projection and package identity for a permuted
  wire order; the latter is judged by FR-322 rather than an invented content
  order refusal.
- Rekey members using an incorrect retained label, so the first pass succeeds
  and the second detects the label mismatch (AC-150).

Combine defects to exercise owner-before-key, graph-shape-before-key,
application-before-structural, first-pass-before-second-pass and least retained
node-id offender selection. In particular, combine a higher-key stale
structural node with a lower-key group-label defect: the first-pass node wins.
For the group-label-only cases, compare the reported node with the least
retained member and compare multiple failing groups by their least members.

For a fully valid `Tree` read, account independently for membership entries,
node visits, local-preimage term visits and digest computations. Determine the
exact smallest sufficient work limit M; read with M and M-1 and compare the
incomplete accounting and first unpaid pointer. Keep this whole-read boundary
control; M-1 may exhaust at a later reader stage after group work is complete.

Separately select a charge inside the production group-local preimage or
group-digest work for valid `Tree`, with a later group visit or digest still
pending. Set the existing work ceiling so that this selected group charge is
denied. Record the attempted charge's value pointer and actual group visit and
digest events. Require work-incomplete at that failed-charge pointer, no
admitted package, and no subsequent group visit or digest event. The pending
work must be observable on a sufficient-budget control using the same group;
an empty remainder cannot prove early termination.

Exercise a genuine continue-after-unpaid-charge mutant in the production group
walk: let it perform pending group work instead of immediately propagating
the selected charge error. Require the same observation-based control to fail
because a later group visit or digest occurs, even if the final outcome remains
incomplete. Restore immediate propagation and require the control to pass with
the failed-charge pointer and no package. These internal-group exhaustion,
mutant and restored controls are PLANNED and UNRUN. They do not replace the
whole-read M/M-1 control. No limit increase, new cancellation surface or
test-only bypass of the production group walk is allowed.

The required regression test is the IR-627 tamper probe: the `Int[0, 1000]`
node with `max` changed to `10`, and separately to `5000`, each refused.

## Expected Results

The genuine positive packages admit with actual-owner keys and verified
labels. Every old-key mutation refuses `invalid_package`/`stale-node-key`
without a package, at the first-pass least stale node's `node_id`. Adding a
label to an ordinary ungrouped key is now a stale-key case even when its body
is unchanged. Off-cycle same-label members are never verified as ungrouped.
A label-only mismatch reached after all member keys pass reports the group's
least retained member. Both passes preserve the stage precedence above.
The forgedSelfOption refuses at the old option node's `node_id`; genuine G1
admits. Closed-shape defects with no derivable key retain no expected key.
A fully rekeyed wire permutation is not refused solely for its order.
Exact sufficient work admits; the first unpayable charge returns work-incomplete
with existing accounting and no partial package. For the separate selected
internal-group denied charge, the result names that failed charge's pointer;
no pending group visit or digest occurs. The genuine continuation mutant
violates this event assertion, and the restored immediate-return control
satisfies it. These outcomes remain PLANNED and UNRUN.

The re-pointed model selection still refuses `missing_import`/
`missing-selection` without its document and admits with matching evidence;
this reader does not choose the caller's authoritative domain document.
For AC-183, the tampered, unreferenced `base.zero` node's refusal retains the
reader-derived expected key and original `locus`; a fresh package with that
node rekeyed at every occurrence, including its source-map entry (using the
existing `rename_node` and `rebuild_source_map` fixture helpers), and its
package identity refreshed admits in full. A malformed
shape with no derivable key has no expected key.

## Status

IR-630 owner-bearing, declared and labelled-group controls are PLANNED and
UNRUN. No IR-630 code, runtime tests, Cargo/Kani gates or external conformance
runs are claimed by this specification amendment. Existing owner-free,
ungrouped controls and the implemented AC-183 retained-key control describe
baseline evidence only. Former AC-145 through AC-149 residual skip/admission
controls must be amended with the IR-630 code and cannot prove the new
criteria through their old trace tags. The owner-wire boundary remains
AC-153 through AC-155; checkout conformance remains AC-176 and gains its
planned IR-630 authentication checks without copying upstream artifacts.

## Retained derived-key refusal (FR-038-AC-183)

Implemented and run by `tc_048_derived_expected_keys_rekey_unreferenced_nodes_in_both_branches`. Run the owner-free, ungrouped,
unreferenced `integer_range` `base.zero` tamper case through the production
reader. Require its `expected_node_id()` to return the key that reader
derived, while `locus` keeps the tampered key and code/cause/path remain
unchanged. In a fresh package, replace that unreferenced node's id with the
returned typed key at every occurrence, including its source-map entry (using
`rename_node` and `rebuild_source_map`), mirror its projection, refresh the
package identity and require the whole package to admit. Change only the bound
again and require a
different returned key. A malformed body shape with no derivable key returns
`None`; no test-side FR-092 encoder or digest constant is an oracle.
