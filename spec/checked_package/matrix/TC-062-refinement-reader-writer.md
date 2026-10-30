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
package abstract-subject and row-empty controls. Assign structural ids by package, path and ordinal; retain complete authored bodies. Read, write and reread
each positive package, checking its body, target joins, dependencies,
canonical order of the five non-step arrays, retained `steps` declaration
order and occurrences. Include two valid, distinct step rows in both orders;
check that each order admits and survives read/write/read unchanged, with
generated occurrence ordinals and source regions following the received order.

Apply the adverse mutations of FR-061-AC-2 through AC-9 one at a time.
Include each step target kind, `open` in and outside argument position, a
local and a directly selected dependency protocol key, wrong local/external
key forms, an unselected package, each row array's missing/extra member,
noncanonical ordering of a non-step array, a history initial of both admitted
forms and of an application, missing/extra dependencies, two row-empty concrete subjects,
proof-eligible and ineligible occurrence sets, and pairs of simultaneous
defects at the earlier/meaning stages. Do not treat a `steps` permutation as
an order defect. For a missing generated occurrence,
remove its `source_map` entry and assert `/source_map`; for duplicate,
extra-role, extra-ordinal and wrong-region entries, assert the offending
entry's pointer. For within-node precedence, combine a duplicate structural refinement id with a valid permutation of `steps` and assert the duplicate-id refusal; with distinct valid structural ids, combine that permutation with a
wrong-kind `abstract` header and assert the `abstract` join refusal, then
repair the header and assert admission with the received step order. Combine a
wrong-kind `semantic_type` with wrong-kind `abstract`, optional `protocol`
and `profile` headers and assert `semantic_type` first, then repair each
header in that order and assert the next. Combine a bad `profile` header
with a wrong-kind population row and assert the profile refusal first;
repair the header and assert the row refusal. Within the canonical and row
stages, place defects in different arrays and nested `fields`/`updates`,
then two entries of one array and two properties of one row entry; assert
the five non-step array/nested-array order, ascending indices and schema-declared
entry-member order where FR-375 fixes no further tie-break. Retain structural ids on valid body mutations and keep the same selected packages in the body comparisons.

For FR-061-AC-3 and AC-7, construct IR-owned wire packages with two selected
concrete models, each with its own canonical `model_root`, while retaining one
abstract root and profile. Retain the complete structural ids while replacing only `semantic_type`. Admit the row-empty model-subject swap
in a separately read package, and admit the protocol-subject swap only when it has no
concrete key and the newly selected model supplies the same class set, including two
empty sets. Separately refuse an absent protocol catalog at
`/body/class_catalog`; a foreign `populations[0].concrete` before foreign
object and step keys as `invalid_model_binding`/`malformed-declaration` at
`/body/populations/0/concrete`; a foreign object key before a foreign step key
at the object key; and a foreign catalog-present step anchor before a later
catalog discrepancy at the step's `source_node`. Check a foreign history
concrete type, history update anchor and `assume` anchor at their own row keys
against valid declaring-type controls. For row-empty protocol subjects,
compare the concrete catalog with the new model's classes in catalog-key order: an extra earlier
class refuses at its entry, while an omitted earlier class refuses at the
`concrete` array. Keep the abstract root and optional protocol clause joins
and exact local dependencies in each control. QSpec TC-310's SR-01 through
SR-13 are graph-case oracles, not published two-selection wire packages.

For FR-061-AC-14 through AC-16, construct catalog-free model-only refinements
with operation steps anchored at declaring object-type, process and state-
machine nodes of the selected concrete model. Include an operation inherited
by the selected type whose anchor remains at its declaring type. Keep the
refinement `semantic_type` at the canonical concrete `model_root`, assert admission and read/write/read retention. Mutate one
control at a time: remove `source_node`, use an unresolved or wrong-kind
source node, change `source_class` away from the anchor's operation name, and
use an anchor whose context belongs to a foreign model or the wrong selected
subject. Assert the exact FR-061-AC-15 code, cause and row pointer. For
FR-061-AC-16, keep an anchor name/`source_class` mismatch as the later row
defect and make a separate combined control for **each** earlier stage:

1. Add an unknown refinement-body member; assert the closed-body wire
   refusal `invalid_semantic_graph` at the body before the row mismatch.
2. Duplicate a structural node id after a body mutation; assert
   `invalid_semantic_graph` at the later `node_id` before the row mismatch.
3. Reverse two object field rows while retaining the step name mismatch;
   under the authored structural ids, assert `invalid_semantic_graph` at the first
   out-of-order field before the row mismatch.
4. Select a wrong profile in the refinement header; with structural ids retained, assert `invalid_model_binding`/`malformed-declaration` at
   `body/profile` before the row mismatch.
5. Add a closed `class_catalog` with empty `concrete` and `abstract` arrays
   to the model-only subject that requires no catalog; with structural ids retained, assert `invalid_semantic_graph` at `body/class_catalog`
   before the row mismatch. This control exercises the catalog-presence
   stage while the step remains catalog-free in the valid baseline.

For each control, assert the earlier stage, exact code, cause when defined,
and RFC 6901 pointer; then repair only its earlier defect and assert the
remaining row mismatch at `source_class`. Put defects in two steps
and assert the first row in received declaration order wins. Reverse those
two step rows, retain structural ids and check the mismatch follows its row
to the new index without an order refusal. Finally use a separate valid
protocol refinement with a present catalog; change its row class to one
absent from the concrete catalog, then restore the class and change its
`source_node` away from the matching catalog entry. Confirm the existing
missing-class and wrong-node refusals. Retain all structural ids and update exact bodies after each mutation, except for the intentional duplicate-id precedence control.

## Expected Results

Each positive vector admits and round-trips with the same complete node,
node id and package name. Every adverse vector refuses the entire package with
FR-061's code, cause and pointer; no row is defaulted, reordered, omitted or
interpreted under another form. The two concrete subjects produce distinct
node ids despite equal bodies; a changed received `steps` order changes the
complete body under the existing identity rules. Earlier reader stages win
over later refinement joins.
The simultaneous body defects report the first FR-375 stage, then the first
header or array and nested array in that stage, then the first array index
and schema-declared entry member where FR-375 fixes no further tie-break.
Catalog-free operation rows admit declaring contexts owned by the selected
concrete subject, including inherited declarations, without equating the
context to `model_root`. The adverse and combined controls return the
FR-061-AC-15 and AC-16 refusals at their specified loci.
