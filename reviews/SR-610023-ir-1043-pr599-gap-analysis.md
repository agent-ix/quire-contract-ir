---
id: SR-610023
title: "IR-1043 PR 599 gap-analysis review"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; PR #599 head 73909ecbc7030eecf5bb9c2bff3154879067d7c5; crates/quire-contract-model/src/checked_package/v2; tests/it; FR-065"
review_set: subset
---

## Summary

Ticket: IR-1043. One frozen-head diff review at 73909ecbc7030eecf5bb9c2bff3154879067d7c5. All eight FR-065 criteria were examined for code and trace claims. AC-5 closed-wire cases are implemented; its well-formed unselected-pair exact oracle remains open on STD-500 and is not claimed. No applicable AssuranceProfile was found under spec/.

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
| FND-001 | medium | The tagged AC-4 tests cover one state-machine declaration and zero selected packages, but not multiple object types or state machines. A root-only zero-declaration selected package exists in an AC-3-tagged test, so the zero-declaration behavior is exercised but not bound to AC-4. No test varies a selected document from one declaration to several and checks that the root count stays one. A count-dependent producer defect could remain green. | tests/it/checked_package_v2_model_root.rs:167-203 |
| FND-002 | medium | The AC-1 producer clause has no source-producer evidence: ensure_model_roots is a test-only JSON fixture helper and the public roundtrip test starts from it. The test proves IR reader/writer carriage, but does not exercise a source producer deriving the root from an existing selection. Keep that clause unclaimed until the authoritative producer supplies an independent fixture or conformance run. | tests/it/support/checked_package.rs:372-478 |

## Coverage

Plan completion: not assessed.
Computed Quire matrix tags AC-1 through AC-7; AC-8 is Inspection with no test symbol. Repository-wide baseline is not a completion claim. Reverse ownership is FR-065; no new unowned source path. Optional semantic audit was not run separately; the changed code and tests were read in this scoped review.

## Verdict

FAIL: findings require a same-PR fix or a truthful unclaimed disposition.
