---
id: SR-610024
title: 'IR-1053 PR #602 code-review review'
type: SpecReview
analysis: code-review
scope: agent-ix/quire-contract-ir; spec/checked_package/functional/FR-038-consume-checked-package-v2.md;
  spec/checked_package/functional/FR-064-bind-protocol-clause-to-owner-model.md; spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md
review_set: subset
---

## Summary

Ticket IR-1053. Independent review of quire-contract-ir PR #602 at 2da06ffc3b40620500d8810c93974e7c096d3522 against merged QSpec FR-375/FR-464 at e3d48d76 and IR FR-061/FR-038/FR-064.

## Examined units

- FR-038-AC-325 (examined): The protocol step compares every present refinement `class_catalog` against the complete concrete and abstract classes reconstructed from selected model declarations and the admitted local or directly selected dependency protocol graph, including distinct `fork` and `join` classes on one parallel node, checked binder types and source-default memory classes: `memory drain` for each selected `sc`, `tso` and `ra` source default with `drain: store-buffer-flush`, and `memory flush` for `tso`, even with no step row. The concrete refinement `semantic_type` resolves to a selected package `model_root`,
- FR-064-AC-4 (examined): With wire, graph and identities valid, an `owner_model` pointing at a non-model node, a source-local declaration, a model declaration owned by no selected declaration, or a selected declaration's model node of any form other than `object_type` (a `state_machine`, `process`, `record_value_type`, a relation node, or the selected package's `model_root`) refuses at the edge under QSpec FR-464's typed model-binding check before clause interpretation. A separate earlier wire, graph or stale-key fault wins when paired with such a meaning defect. A source producer given a clause declared `over` model 
- FR-064-AC-5 (examined): A protocol refinement whose concrete `semantic_type` is its selected package's canonical `model_root` and whose complete concrete catalog includes classes from an admitted clause whose distinct `owner_model` is the object-type node of a model owned by its selected concrete declaration is admitted even when no step row names those classes. A clause whose edge names a model owned by another declaration or an unselected package contributes no class to that catalog, regardless of similar declaration names or binder types. A catalog missing an owned clause's class or including a foreign clause's cl
- TC-048-step-9 (examined): Build an independent positive package through the public typed writer: select concrete model package A with its canonical `model_root` R, owned object-type node O and protocol clause C whose `owner_model` is O; also select model package B with a different root and object-type node. Put R in the refinement's concrete `semantic_type`, not O. Give C a checked control class and omit C from the refinement's `steps`, while placing its class in the complete concrete catalog. Require public strict read and typed read/write/read to admit with the same keys, package identity, root, owner and catalog (FR
- TC-048-step-10 (examined): From that independently valid package, make one identity-consistent mutation per case, rebuilding every affected node key, dependency and package identity: retarget C to B's object-type node; retarget it to a selected `state_machine`, `process`, `record_value_type`, relation or `model_root` node; and retarget it to a source-local or unselected declaration node. Require the FR-064-AC-4 typed edge refusal for each wrong-kind or unselected target, and require B's otherwise valid clause to contribute only to B's catalog, never A's. A source producer given C declared over A SHALL emit O even when B
- TC-048-step-11 (examined): Keeping the positive package's owner join and protocol body valid, mutate the concrete catalog separately to omit C's class, add B's foreign class, duplicate or reorder an entry, change C's class node or binder signature, omit a catalog required by a protocol or memory class, and add a catalog to a model-only refinement without those classes. Require respectively `invalid_semantic_graph` at the concrete array for omission, that code at the extra entry, the earlier refinement canonical-order refusal for duplicate or disorder, `invalid_model_binding`/ `malformed-declaration` at the changed entry
- TC-048-step-12 (examined): For owned source `parallel` nodes, derive separate valid catalogs with source defaults `sc`, `tso` and `ra`. Require a node-free, binder-free `memory drain` entry with `drain: store-buffer-flush` for each default; require `memory flush` with `memory_model: tso` only for `tso`. Keep the memory class when no refinement step row names it. In separate rekeyed packages, omit a required memory class, add `memory flush` under `sc` or `ra`, change the drain model or drain member, and duplicate or reorder a memory entry. Compare the exact QSpec FR-375 code, cause and catalog pointer, including earlier 
- FR-061-AC-1 (context_only): A positive model-subject refinement with one population, object, operation-step, history and fairness row admits as one `relation`/`refinement` node with the exact nine required body members and its complete rows; empty arrays are retained, optional `protocol` and `abstract_internal` are absent, `semantic_type` names the concrete model, and the public writer's output reads back to the same node and package identities.
- FR-061-AC-7 (context_only): Two row-empty refinements over one abstract model and profile but distinct concrete models have equal bodies, distinct `semantic_type` keys and distinct `quire.refinement-node/v1` node ids. Mutating any meaning-bearing body row or optional internal list with recomputed identities changes node and package ids; retaining a stale key refuses at the earlier key stage; source row permutation that canonicalizes to the same arrays changes neither id.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

PASS for this method; no findings.
