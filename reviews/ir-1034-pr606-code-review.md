---
id: SR-610046
title: "IR-1034 PR 606 Rust code review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; PR #606; crates/quire-contract-model/src/checked_package/v2/refinement.rs, crates/quire-contract-model/src/lib.rs, tests/it/checked_package_v2_refinement.rs"
review_set: subset
---

## Summary

The closed memory catalog wire forms, canonical keys, subject joins, and public writer roundtrip were inspected. The new subject discovery repeats graph scans without charging the validation work budget.

## Verdict

**CONDITIONAL — charge or avoid the unmetered graph scans before merge.**

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | `protocol_memory_models` repeatedly resolves each clause owner by scanning all nodes before any work charge, allowing quadratic unmetered work under a small work limit. | crates/quire-contract-model/src/checked_package/v2/refinement.rs:2423 |

## Coverage

FR-038-AC-3, FR-061-AC-10..13 and FR-019-AC-5 examined. Source authority: QSpec FR-375-AC-26. The changed Rust paths contain no copied dependency or compatibility layer. The coder reported 25 focused refinement tests and focused Clippy/fmt passing; the reviewer did not run an aggregate build.

## Scope

- **FR-038-AC-3 (examined):** Exact byte, node, edge, occurrence, diagnostic and work limits admit a package; each one-over limit returns `incomplete` with that limit kind, the limit and the consumed counter and no package; no limit kind is a depth (FR-038-AC-117).
- **FR-061-AC-10 (examined):** An external consumer builds one typed refinement catalog over source `parallel` controls defaulting to `sc`, `tso` and `ra`. Its `concrete` array contains exactly three `{class: "memory drain", memory_model, drain: "store-buffer-flush"}` entries ordered by the (`class`, `memory_model`) key as `ra`, `sc`, `tso`, then the separate `{class: "memory flush", memory_model: "tso"}` entry. The public writer encodes each without extra members; the strict reader admits all four and retains the typed drain value and model through read/write/read with unchanged node and package identities. The typed drain variant is distinct from the TSO-only flush variant.
- **FR-061-AC-11 (examined):** Starting from an otherwise valid package, deleting `drain` from `memory drain`, changing its value or type, adding it to `memory flush`, assigning `memory flush` model `sc` or `ra`, or adding `node`, `binder_types` or another unknown entry member refuses `malformed_wire` at the closed-wire stage, with the offending node/member location and no admitted package or defaulted field. With identities recomputed, reordering the three drains, repeating a (`memory drain`, `memory_model`) pair, or repeating a declared class spelling refuses `invalid_semantic_graph` at the first out-of-order or later duplicate catalog entry after the dedicated key check.
- **FR-061-AC-12 (examined):** A protocol refinement catalog includes one `memory drain` per distinct `sc`, `tso` or `ra` source-default model selected by a `parallel` in its subject and additionally one `memory flush` for a TSO source default. Two `parallel` controls selecting the same source-default model contribute one entry per model, not one per control. In a present catalog, an omitted required drain or TSO flush, an extra memory class, or a memory class from another selected subject refuses `invalid_semantic_graph` at the catalog array or entry after the protocol-body check; a wholly absent required catalog refuses earlier in the refinement step. A request-time memory override changes neither catalog nor package id.
- **FR-061-AC-13 (examined):** With all identities recomputed, changing a schema-valid drain entry's `memory_model` changes the dedicated refinement node id and package id; retaining its old node id refuses `invalid_package`/`stale-node-key` at `node_id` before catalog joins. A local catalog class node or binder type omitted from the exact `dependencies` list refuses `invalid_semantic_graph` at `/dependencies` before the dedicated key check, while no memory entry contributes a node or binder dependency.
- **FR-019-AC-5 (examined):** The `quire_contract_model` crate root has no glob re-export, and its default-feature public items are exactly those the Public items table lists as implemented at the tested revision. Entries explicitly marked planned are excluded until their owning code lands. The public-item inventory fails on an added or missing implemented item; no implemented item in the table is reachable through a `quire_contract_ir` path.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The owner lookup now builds a charged node index once, then uses indexed lookups for each clause; no whole-graph scan remains inside the clause loop. |

## Disposition verdict

**PASS** — every recorded finding is fixed in the original PR.

## Rebase verification

**PASS** — range-diff against the reviewed head keeps the refinement implementation and tests patch-equivalent. The only conflict resolution combines additive crate-root exports and FR-019 inventory rows from IR-1040 and IR-1034; both are retained. All earlier findings remain fixed.

## Fixture correction verification

**PASS** — compared with the preceding review, the sole change makes the typed-term roundtrip fixture use `memory_model: tso` for `memory flush`, the only admitted model of `RefinementTsoModel`. No production code, FR-061 acceptance test, public export or FR-019 inventory row changed; the prior dispositions remain fixed.
