---
id: SR-2780
title: "IR-651 composite operand accessor code and Rust review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; PR #320; crates/quire-contract-model/src/checked_package/v2/composite_operands.rs, crates/quire-contract-model/src/checked_package/v2/mod.rs, tests/it/checked_package_v2_composite_operands.rs, tests/it/main.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: references
---

## Summary

Ticket: IR-651. Pull request agent-ix/quire-contract-ir#320, reviewed against its
merge base on IR main, which includes the merged IR-680 and IR-654 code. This file
holds the code review with the rust-review lane folded in. The reviewed head is
recorded only in the Linear review marker.

Examined units:

- `CheckedPackageV2::composite_application_operands`: the node, application,
  occurrence and operation order; the per-argument inline, application and
  reference branches; family eligibility; the parameter and literal dispatch.
- `Meter` (`charge`, `length`, `path`, `text`), `Graph::node`, `kind`, `typed_id`,
  `target`, `resolved_kind`, `forwarding_target` and `structural_family`.
- `project_type`, `descriptor`, `next_edge`, `depth_position`, `position` and
  `bound`: suppression of nested bounded wrappers, recursion through the ancestor
  chain, optional-presence wrappers, and the refusal of unanchored cycles.
- `closed_literal`: the closed-value walk, value-cycle detection, union handling
  and missing-node handling.
- The public surface re-exported from `v2/mod.rs` (eleven types and one method),
  compared with the FR-038 accessor table.
- The crate-internal unit tests and the seven integration tests.

Rust-lane checks: production code has no `unwrap`, `expect`, slice indexing or
unchecked arithmetic on package data. Both walks use explicit heap stacks and do
not recurse. Every visit, edge, retained path and copied byte is charged before it
is used. There is no depth cap, and the meter saturates to `u64::MAX` on overflow.
The error enum is typed and closed: no caller has to parse Display text. The
derived `Debug` and `PartialEq` impls expose only public content. No dependency,
reader refusal type or IR-680 expected-key field changes, so AC-183 through AC-185
are not affected.

## Verdict

**CONDITIONAL.** The accessor's main paths match the FR-038 contract. The
refusal order, the decimal-text bounds without narrowing, Depth keys taken from the
ancestor chain, the optional-presence paths and the closed public types are all
correct as read. Three medium findings need a fix round. First, every logical
charge also pays for an uncharged linear scan over the whole node array. Second,
the eligibility check re-encodes the catalog family and the reference decoding
that the reader already owns. Third, `consumed_work` charges more than the
specified logical formula. Two low findings concern test fixtures and one locus.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Each charged node lookup scans the whole node array linearly, so CPU grows as work limit times node count; the retained `kinds` and the reader's index pattern are not reused | crates/quire-contract-model/src/checked_package/v2/composite_operands.rs:278-296 |
| FND-002 | medium | Eligibility re-encodes the catalog's `structural_kind` group and forwarding (`structural_family` and `resolved_kind`) and reference decoding (`typed_id` and `target`) instead of reusing the reader's `resolve_family`, `OperationCatalog::family_fits` and `structural::reference_target` | crates/quire-contract-model/src/checked_package/v2/composite_operands.rs:298-315 |
| FND-003 | medium | `consumed_work` exceeds the FR-038 logical charge formula: a transient path clone is charged before every child, then charged again as the retained shape path, and an optional-presence target is resolved twice | crates/quire-contract-model/src/checked_package/v2/composite_operands.rs:1012 |
| FND-004 | low | The unit-test fixture forges a whole `CheckedPackageV2` with `kinds: Vec::new()`, which disagrees with its nodes, instead of mutating a reader-admitted package after admission | crates/quire-contract-model/src/checked_package/v2/composite_operands.rs:1172-1208 |
| FND-005 | low | An application subterm reached by graph reference reports `child.semantic_type`, but FR-038 names the application's result type; the reader does not tie the two together | crates/quire-contract-model/src/checked_package/v2/composite_operands.rs:433-438 |

### FND-001 detail

`Graph::node` runs `self.nodes.iter().find(..)` for every charged resolution
attempt. With the default 10,000-node read limit, a caller work limit of one
million performs up to ten billion uncharged identity comparisons. FR-038 lets the
lookup strategy vary without changing the logical charges, but the repository rule
is that every loop is bounded. The package already retains `kinds` aligned with
`nodes`, and the reader's stages look nodes up through a
`BTreeMap<&CheckedNodeId, usize>`. Building that index once per call costs
O(n log n), bounded by the admitted graph, and makes each charged lookup
logarithmic. `kind()` also decodes each node's tag text again instead of reading
the retained `kinds`.

### FND-002 detail

The operation stage decides operand eligibility with `resolve_family`, which
follows only `bounded_domain` chains (an alias names no family), and with
`catalog.family_fits(actual, "structural_kind")`. The accessor hard-codes the
eight forms instead and also forwards through Alias. The two definitions agree
with today's catalog (option, record, tuple, union, sequence, set, bag,
ordered_set), but nothing makes them stay aligned. FR-038 states that the
accessor's eligibility "does not expand the owning checked-operation catalog".
`typed_id` and `target` duplicate `structural::reference_target`, with stricter
checks of the domain and digest.

### FND-003 detail

In `project_type`, the edge arm calls `meter.path(&frame.path)` to clone the
parent path, which is never retained. The pending visit then charges
`meter.path(&path)` again for the retained shape path. The optional-presence arm
in `next_edge` (lines 905-907) charges a resolution of the Option target, and
`project_type` (lines 960-962) charges a second resolution of the same target. The
result is deterministic, so the exact-budget clause holds. The returned
`consumed_work` is public, however, and a consumer that budgets from the specified
formula (one unit per resolution, visit, edge, retained path element and copied
byte) under-budgets. No test computes the expected charge independently (see
SR-2781).

### FND-004 detail

TC-048 allows crate-internal *post-admission* mutations. `package()` instead
deserializes a wire that the reader never admitted and sets `kinds` empty. The
tests pass only because the accessor decodes kinds again on every lookup (see
FND-001). A fix that reads the retained `kinds` would break these tests without
any behaviour change, and the fixture can hold states that admission plus mutation
cannot produce.

### FND-005 detail

FR-038's locus rule reads: "`type_node` shall identify ... the application
subterm's result type". The inline branch at line 422 reads `result_type`, but the
graph-node branch uses the node's `semantic_type`. The only test that covers this
branch sets both to the same id, so a wrong-field mutant survives.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-006 | low | The FND-002 fix maps every type with no operand family (unit, dimension, compound_unit, alias, relation forms) and every bounded-domain cycle to `MalformedDomain` at the operand's semantic type, not `NonStructuralType`; before the fix these refused `NonStructuralType` | crates/quire-contract-model/src/checked_package/v2/composite_operands.rs:437-452 |

### FND-006 detail

`resolve_family_with(..)?.ok_or_else(|| type_error(ordinal, Some(child.semantic_type.clone())))`
turns the reader's `None`, which covers a missing node, a bounded-domain cycle
and a valid type with no operand family, into `MalformedDomain`. FR-038 says
"nonstructural operand types shall refuse reason `NonStructuralType`", and it
keeps `MalformedDomain` for a missing type, malformed structure or an unanchored
cycle. A post-admission mutation that retypes a parameter to a `compound_unit`
node now returns `MalformedDomain { ordinal, type_node: Some(<unit type>) }` where
FR-038 expects `UnsupportedOperand { reason: NonStructuralType }`. A cycle
reports the operand's type instead of the first malformed node. Admission already
refuses these operands, so only the defensive path is affected. No test covers a
family-less valid type.

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | commit "Close IR651 composite accessor review findings" |
| FND-002 | fixed | commit "Close IR651 composite accessor review findings" |
| FND-003 | fixed | commit "Close IR651 composite accessor review findings" |
| FND-004 | fixed | commit "Close IR651 composite accessor review findings" |
| FND-005 | fixed | commit "Close IR651 composite accessor review findings" |

## Disposition evidence

Round 1. A static re-check of the fast-forward fix head against the original
reviewed head. No cargo was run, and the author reports that the repaired exact
test and Clippy have not run yet, so every runtime claim below needs the second
full gate.

- FND-001: `CheckedPackageV2` now keeps a private `node_index:
  BTreeMap<CheckedNodeId, usize>`, built once at admission and left out of
  `PartialEq`. `Graph::node` charges 1 and then makes an O(log n) map lookup, and
  `Graph::kind` reads the reader's retained `kinds` through the same index. No
  linear scan of the node array remains. Every in-crate struct literal sets the
  new field. Mutant killed: a linear `find` is no longer present, and the
  per-charge cost is logarithmic.
- FND-002: family eligibility now calls the reader's
  `operations::resolve_family_with` (`resolve_family` delegates to it) and
  `operation_catalog().family_fits(family, "structural_kind")`. `structural_family`
  is deleted, and `target` and `typed_member` delegate to
  `structural::reference_target` and `structural::checked_node_identity`. The
  reader's `reference_target` now validates domain and digest. Intake
  (`common::visit_reference`) already refuses any body reference that fails those
  checks before any stage that calls `reference_target`, so admission behaviour is
  unchanged. Residual risk: confirm with the full gate.
- FND-003: the transient path clone is no longer charged (`frame.path.clone()`),
  the separate pushed-index charge is gone, and the optional-presence target is
  resolved once, during its own visit, where its Option kind is checked. Two
  integration tests assert `consumed_work` against hand-derived totals
  (`1 + 2*(3+5+5+2+4+2+6+3+text)` and `1 + 2*(3+6+2+3+5)`). I re-derived both from
  the code's charge sites and they match. The exact-budget, one-below (exact
  payload) and zero cases follow. Mutants killed: double path charging and a
  double optional resolution both change the asserted totals.
- FND-004: the unit fixture is now a wire with test-derived canonical keys,
  admitted through `CheckedPackageV2::read`, with corruptions applied afterwards
  (`kinds` is updated with every form mutation). Admission is checked at runtime
  only (`panic!` if refused) and has not yet run.
- FND-005: the graph-reference ApplicationSubterm branch now reads
  `typed_member(&child.body, "result_type")`. The unit test uses
  result_type != semantic_type, which kills the wrong-field mutant, and the public
  integration case asserts the application's result type.

Regression checks: the public API is unchanged (no new `pub` item; the new helpers
are `pub(super)`). There is no new unwrap, expect, index or unchecked arithmetic
in production code, and no recursion or depth cap. The IR-680 AC-183 through
AC-185 paths are untouched: the `resolve_family` refactor keeps its semantics,
with `seen` keyed by node id instead of position. Strict coverage reports 25
unbacked rows and 0 contradicted at the fix head, the same as round 0. The one new
low finding is FND-006.
