---
id: SR-597001
title: "Code review — IR-874 non-memory catalog graph join"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/v2/{mod,model_fields,model_members,protocol_body,refinement}.rs; tests/it/checked_package_v2_{protocol_body,refinement}.rs; FR-038-AC-325; FR-064-AC-5"
review_set: subset
---

## Summary

Ticket: IR-874. This static review covers draft PR #597, including the typed string-edge fix, its model evidence retention, selected clause join, class comparison and new tests. The PR correctly leaves the complete FR-038-AC-325 and FR-064-AC-5 unclaimed, but the current partial join changes admission for cases it cannot yet reconstruct.

## Verdict

**FAIL** — the reader can admit an invented `await` class and reject a protocol refinement whose concrete model contributes operation classes. Cargo gates were not run in this review because the shared build lock was held by another lane. Hosted Rust Checks failed on the earlier head for string-edge gating; the coder has pushed a typed fix, whose hosted result is not part of this review's evidence.

## Scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| FR-038-AC-325 | examined | The protocol step compares every present refinement `class_catalog` against the complete concrete and abstract classes reconstructed from selected model declarations and the admitted local or directly selected dependency protocol graph, including distinct `fork` and `join` classes on one parallel node, checked binder types and source-default memory classes. |
| FR-064-AC-5 | examined | A protocol refinement whose complete concrete catalog includes classes from an admitted clause owned by its selected concrete model is admitted even when no step row names those classes. A clause whose edge names another model contributes no class to that catalog, regardless of similar declaration names or binder types. |
| FR-038-AC-192 | examined | Retained metadata is acquired and released within the existing bounded document/read/work accounting. Exact sufficient work succeeds; the first unpayable charge preserves the existing incomplete result and no partial package. |
| QSpec FR-375-AC-23 | context_only | A protocol refinement carries a complete, canonical `class_catalog` of its source-level concrete and abstract classes and checked binder types in both V2 schemas and its node-key preimage. |
| QSpec FR-432-AC-8 | context_only | Each abstract step class spells and resolves: a node path for an attempt, event, send, receive, timeout, finish and fence node and a template path for a `cattempt`, and `fork`, `join`, `cend`, `duplicate`, `lose`, `spawn`, `retire`, `activate` and `memory` entries as the table states. |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | `await` is reconstructed as a bare catalog class although FR-432's closed class table has no `await` class; an admitted await holder therefore demands an invented class and can admit it. | crates/quire-contract-model/src/checked_package/v2/refinement.rs:2249 |
| FND-002 | high | The concrete side is compared with protocol-only classes, so a concrete model operation class in an abstract-protocol refinement is refused as extra before model-root reconstruction exists. | crates/quire-contract-model/src/checked_package/v2/refinement.rs:1974 |
| FND-003 | medium | An omitted abstract class before a later present class reports the later entry rather than the catalog array, contrary to the specified omission pointer. | crates/quire-contract-model/src/checked_package/v2/refinement.rs:2467 |

## Evidence

- `catalog_operator` classifies admitted `Control/await`; `protocol_member_classes` emits `Some(vec![path])`. QSpec FR-432's concrete and abstract class tables enumerate timeout, fence and other classes but no await. The positive `protocol_holder_case_on(..., "await")` fixture shows an admitted await holder is reachable; the new catalog tests cover parallel, finish and send but no await case.
- `validate_catalog_graph` visits every refinement with a catalog. `protocol_classes` only iterates `ProtocolClause` nodes. An abstract-protocol refinement requires a catalog, but its concrete model may contribute operation classes with no local protocol clause; those classes are then rejected as extras. QSpec FR-375 says the concrete catalog covers selected model operation classes, and IR FR-038-AC-325 requires classes from selected model declarations.
- `compare_abstract_classes` advances expected and compares class strings without the concrete comparator's early omission check. For expected `[A, B]` and actual `[B]`, it returns `/abstract/0`; FR-375 requires the array pointer for an omission.
- The new code is bounded by `WorkMeter` for the nested clause/member scan. Source evidence retention for direct record field order and effective inherited operation signatures is present. The draft limitations for IR-1043, STD-499, STD-501 and STD-502 are stated without changing acceptance criteria or trace tags.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | rejected | QSpec FR-426 defines `timeout` as the step of the thread at an `await` node and names that node in the transition; FR-432 gives timeout the bare node-path class spelling. The original finding mistook the source operator name for the step-class name., the code names this mapping `TimeoutFromAwait` and tests an admitted await-holder catalog. |
| FND-002 | still-open | In the reviewed PR, `protocol_classes` still derives only protocol-clause classes and does not reconstruct selected concrete model operation classes. This remains an IR-1043-dependent limitation; PR #597 must stay draft and the full FR-038-AC-325 claim unmade. |
| FND-003 | fixed | In the reviewed PR, `compare_abstract_classes` peeks at the next expected class and returns the catalog array pointer when it precedes the next actual class; the public-reader test omits fork before join and asserts that pointer. |
| FND-002 | still-open | In the reviewed PR, the graph join still reconstructs only protocol clauses and omits concrete model operation classes; IR-1043 remains the carrier blocker. This partial PR stays draft. |
| FND-004 | fixed | In the reviewed PR, `RetainedModels::new` no longer takes or charges the reader meter and no longer eagerly resolves operation signatures. `operation_binder_types` takes the meter only when the catalog join selects a concrete operation anchor; focused exact-work and bounded-chain tests passed locally at this head, with hosted full Rust Checks pending. |

## New findings (disposition pass 2)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | high | Eager operation-signature retention charges the shared reader meter for unrelated model reads, changing exact work totals and the first unpayable pointer. | crates/quire-contract-model/src/checked_package/v2/model_fields.rs:503 |

Hosted Rust Checks [run 40532190627](https://github.com/agent-ix/quire-contract-ir/actions/runs/40532190627) at exact completed 934 workspace tests and failed 10 existing exact work-accounting tests. New `RetainedModels::new` loops through every selected object and calls `operation_names` and `effective_operation_binder_types` with the admission `WorkMeter` before the graph join, even for packages that have no refinement catalog. The resulting failures include IR-987 work totals 388→398 and 183→193, TC-048 97→99, and first-unpayable pointers shifted from a graph dependency or member target to `/lock/model_selections/0` or `/lock/model_selections/1`. TC-227's previously admitted long-chain boundary also becomes `incomplete`. The fix must preserve the existing accounting on unrelated reads, not change the exact-work tests' expected values.

## Disposition Round 3

| FND | outcome | reason |
| --- | --- | --- |
| FND-002 | fixed | In the reviewed PR, `validate_catalog_graph` obtains the selected concrete `ModelRootOwner.identity`, adds `ModelOwners::operation_classes(identity, meter)` to the clause-derived concrete classes, sorts the combined inventory, and compares the catalog. The new public-reader test admits `Worker::run` with the selected declaration key and receiver type, and refuses omitted, wrong-node and wrong-binder variants. This resolves the protocol-only concrete inventory defect for the modeled operation case; complete FR-038-AC-325 remains unclaimed. |

**CONDITIONAL** — the reviewed code finding is fixed in the current diff. The corresponding abstract-protocol regression case remains open in SR-597002, and the full FR-038-AC-325 and FR-064-AC-5 criteria remain unclaimed. No aggregate gate was run in this reviewer pass.
