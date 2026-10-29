---
id: SR-610047
title: "IR-1034 PR 606 scoped gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; PR #606; crates/quire-contract-model/src/checked_package/v2/refinement.rs, crates/quire-contract-model/src/lib.rs, tests/it/checked_package_v2_refinement.rs"
review_set: subset
---

## Summary

The four FR-061 memory criteria have trace tags and public-reader coverage, but AC-12 and AC-13 each retain an untested named branch.

## Verdict

**CONDITIONAL — add focused reader oracles for the two named branches.**

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | No test adds a schema-valid extra memory class to a concrete catalog whose selected source does not default to that model; the AC-12 extra-class refusal remains unverified. | tests/it/checked_package_v2_refinement.rs:741 |
| FND-002 | medium | The AC-13 `/dependencies` test removes a class node only; no test removes a binder type while retaining its class node. | tests/it/checked_package_v2_refinement.rs:833 |

## Coverage

Plan completion: not assessed. Public-reader tests tagged FR-061-AC-10..13 were inspected; code and trace tags agree on the tested branches. The optional repository-wide semantic review was not requested. No source stub or copied behavior was found. Full package gates remain with the code lead.

## Scope

- **FR-061-AC-10 (examined):** An external consumer builds one typed refinement catalog over source `parallel` controls defaulting to `sc`, `tso` and `ra`. Its `concrete` array contains exactly three `{class: "memory drain", memory_model, drain: "store-buffer-flush"}` entries ordered by the (`class`, `memory_model`) key as `ra`, `sc`, `tso`, then the separate `{class: "memory flush", memory_model: "tso"}` entry. The public writer encodes each without extra members; the strict reader admits all four and retains the typed drain value and model through read/write/read with unchanged node and package identities. The typed drain variant is distinct from the TSO-only flush variant.
- **FR-061-AC-11 (examined):** Starting from an otherwise valid package, deleting `drain` from `memory drain`, changing its value or type, adding it to `memory flush`, assigning `memory flush` model `sc` or `ra`, or adding `node`, `binder_types` or another unknown entry member refuses `malformed_wire` at the closed-wire stage, with the offending node/member location and no admitted package or defaulted field. With identities recomputed, reordering the three drains, repeating a (`memory drain`, `memory_model`) pair, or repeating a declared class spelling refuses `invalid_semantic_graph` at the first out-of-order or later duplicate catalog entry after the dedicated key check.
- **FR-061-AC-12 (examined):** A protocol refinement catalog includes one `memory drain` per distinct `sc`, `tso` or `ra` source-default model selected by a `parallel` in its subject and additionally one `memory flush` for a TSO source default. Two `parallel` controls selecting the same source-default model contribute one entry per model, not one per control. In a present catalog, an omitted required drain or TSO flush, an extra memory class, or a memory class from another selected subject refuses `invalid_semantic_graph` at the catalog array or entry after the protocol-body check; a wholly absent required catalog refuses earlier in the refinement step. A request-time memory override changes neither catalog nor package id.
- **FR-061-AC-13 (examined):** With all identities recomputed, changing a schema-valid drain entry's `memory_model` changes the dedicated refinement node id and package id; retaining its old node id refuses `invalid_package`/`stale-node-key` at `node_id` before catalog joins. A local catalog class node or binder type omitted from the exact `dependencies` list refuses `invalid_semantic_graph` at `/dependencies` before the dedicated key check, while no memory entry contributes a node or binder dependency.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The public-reader test inserts a schema-valid RA drain into a subject whose source defaults only to SC and asserts invalid_semantic_graph at concrete/3. |
| FND-002 | fixed | The dependency test separately removes the declared class node and its binder type, recomputes package identity, and asserts invalid_semantic_graph at /dependencies for each. |

## Disposition verdict

**PASS ** — every recorded finding is fixed in the original PR.

## Rebase verification

**PASS ** — range-diff against the reviewed head keeps the refinement implementation and tests patch-equivalent. The only conflict resolution combines additive crate-root exports and FR-019 inventory rows from IR-1040 and IR-1034; both are retained. All earlier findings remain fixed.

## Fixture correction verification

**PASS ** — compared with, the sole change makes the typed-term roundtrip fixture use `memory_model: tso` for `memory flush`, the only admitted model of `RefinementTsoModel`. No production code, FR-061 acceptance test, public export or FR-019 inventory row changed; the prior dispositions remain fixed.
