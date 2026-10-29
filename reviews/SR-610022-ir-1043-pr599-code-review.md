---
id: SR-610022
title: "IR-1043 PR 599 code-review review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; PR #599; crates/quire-contract-model/src/checked_package/v2; tests/it; FR-065"
review_set: subset
---

## Summary

Ticket: IR-1043. One frozen-head diff review. All eight FR-065 criteria were examined for code and trace claims. AC-5 closed-wire cases are implemented; its well-formed unselected-pair exact oracle remains open on STD-500 and is not claimed. No applicable AssuranceProfile was found under spec/.

## Examined units

- FR-065-AC-1 (examined): An external consumer constructs a complete typed V2 package selecting one domain package, with its self-typed `model`/`model_root` node and closed `ModelRootOwner`, calls `encode_checked_package_v2`, then reads the bytes through the public strict reader. The admitted graph, occurrence-free projection and canonical read/write/read retain the same owner, root node id and package id. A source producer derives the root from the existing selection without a new FCD declaration or lock field.
- FR-065-AC-2 (examined): The root has absent `declaration`, `recursion_group` and nominal preimage, empty aggregate body and dependencies, and exactly one `generated` occurrence at ordinal zero whose source-map region is the source occurrence of the selected model package. A declaration occurrence, wrong role or ordinal, wrong or missing region, or extra occurrence refuses through FR-322's occurrence/source-map check; none is silently repaired.
- FR-065-AC-3 (examined): The root's node id equals the JCS SHA-256 of the exact closed `ModelDeclarationNode` preimage containing `ModelRootOwner` and `semantic_type: null`, including QSpec's published model-root vector. Changing only a source alias while preserving the selected identity and digest leaves the root id unchanged. A different selected identity or changed selected-package content gives a different identity or digest and, after rebuilding, different root and package ids. A retained stale root key refuses under FR-322's structural-key check.
- FR-065-AC-4 (examined): For zero, one and multiple object types or state machines in a selected model document, the package has exactly one root for that selection; zero selected packages produce no root. After earlier wire, selected-evidence, graph, occurrence, owner and key checks pass, a selection with no root refuses `invalid_semantic_graph` with no cause at `/semantic_graph/nodes`, in selected-row order before declaration and refinement meaning. Cloning a valid root as a second graph node refuses `invalid_semantic_graph` with no cause at the later `/semantic_graph/nodes/{j}/node_id` in graph shape, before key ch
- FR-065-AC-5 (examined): An absent or extra `ModelRootOwner` member, a `ModelOwner` on a root, or a `ModelRootOwner` on another form refuses at the closed wire/owner-presence stage. A well-formed root whose owner identity/digest pair is not selected also refuses, without inferred selection or repaired owner; an earlier independent defect wins according to FR-322's order. The exact stage, code, cause and RFC 6901 pointer for this unselected-pair case remain pending QSpec STD-500, which must reconcile FR-322-AC-132, the detailed owner-join rule, the schema and an authoritative negative control before this portion is cla
- FR-065-AC-6 (examined): A model-subject refinement uses the selected concrete root for `semantic_type` and the selected abstract root for its local `abstract` key, including an otherwise row-empty refinement. A protocol-subject refinement uses the abstract model root plus the exact protocol clause key. A root in a directly selected dependency resolves only through its package key. A header naming an object type, state machine, alias-derived surrogate, unselected dependency or wrong selected root refuses under FR-375's header joins after earlier FR-322 checks; the reader does not infer a substitute.
- FR-065-AC-7 (examined): An external consumer uses the public typed writer and strict reader to carry `model_root` and `ModelRootOwner` through the existing V2 graph and envelope; read/write/read retains the exact root owner, node id, package id and generated source correspondence.
- FR-065-AC-8 (examined): Source inspection finds one typed V2 graph and envelope path for the root, with no parallel untyped writer, fallback owner inference, compatibility reader, copied QSpec schema or fixture.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The reader validates only one generated occurrence and a nonempty region from any locked source. It never joins that region to the source occurrence that selected the root owner model. A root mapped to a different locked source or unrelated range can be admitted, violating the exact selected-model source correspondence. The synthetic builder likewise copies the first source-map region or first locked source regardless of the model selection. | crates/quire-contract-model/src/checked_package/v2/model_root.rs:47-65 |
| FND-002 | high | The test embeds QSpec origin/main node-identity-vectors.json model-root-counter-cas preimage and expected digest verbatim in IR. This is a second copy of the authoritative QSpec fixture, prohibited by repository instructions and review policy. Resolve and test the vector from QSpec at test time through the existing external conformance lane. | tests/it/checked_package_v2_model_root.rs:209-220 |

## Coverage

Rust typed root, owner, structural, refinement and source-map paths; no code was changed by reviewer. Hosted final gates belong to code lead.

## Verdict

FAIL: findings require a same-PR fix or a truthful unclaimed disposition.

## Dispositions

Round 1, verified at PR #599. The original finding rows remain unchanged.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | deferred | STD-505: QSpec ModelRef has only identity/digest and SourceMapEntry has only raw source regions; no selected-model-to-source-occurrence relation exists. The PR expressly leaves this AC-2 source-correspondence clause unclaimed and retains the full requirement. IR cannot invent the missing alias/range oracle. |
| FND-002 | fixed | The model_root conformance test reads node-identity-vectors.json from the authoritative QSpec proposal directory, selects model-root-counter-cas, constructs a typed draft, calls derive_structural_candidates, and compares the production candidate digest with row["sha256"]. The copied preimage and digest literals were removed from IR. |

## Current verdict

CONDITIONAL: FND-002 fixed; FND-001 deferred under STD-505, with AC-2 source correspondence unclaimed. Review-mergeable only as an explicit partial implementation; IR-1043 remains open.
