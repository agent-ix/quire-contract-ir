---
id: SR-202611010311
title: IR-1252 PR 808 spec-review-base
type: SpecReview
analysis: base
scope: 'agent-ix/quire-contract-ir; IR-1252; PR #808; changed code and five status/inventory
  spec files'
review_set: subset
---

## Summary

Reviewed frozen code candidate 325136e87378cf90b7e06f51444aec698367ddfe against main 8079fcfd. Ticket: IR-1252; PR #808. The latest carrier-only ticket direction supersedes its obsolete scheduler-certificate description. No applicable AssuranceProfile exists under spec/. No Cargo, test, build, or Kani command was run by this reviewer.

## Verdict

**PASS** — No defect found in the status/inventory delta.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

The five spec changes only activate implementation status and the existing FR-019 public-item inventory. The carrier contract, signatures, error precedence, exact values and QSL boundary remain those in merged PR #805; this diff introduces no requirement statement, relationship edge, new domain object or vocabulary. The inventory matches the model crate exports, raw optional writer input and private borrowed reader surface. No broad spec catalog re-review or unrelated pin-purge was performed. Verification remains the existing TC-048/TC-451 and scoped public tests. EARS, dependency, object, risk, evidence and other lenses are not separately selected because their subject did not change.

## Scope

```yaml
scope:
- id: FR-038-AC-396
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented. The public strict reader admits complete packages constructed around QSpec TC-976 SCC-01 and SCC-02, retains one row for occurrence 0 and two distinct rows for the shared node's occurrences 0 and 1 respectively, and exposes their exact borrowed declaration, claim occurrence and FR-482 body in canonical order. A package with no schedulability occurrence omits the index and returns an empty typed iterator. Equal display names or source regions do not select or merge claims.
- id: FR-038-AC-397
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented. SCC-03/04/05/12 refuse `malformed_wire` at their published row/member loci. SCC-06/07/08 refuse `invalid_package`/`invalid-value` at the published declaration/occurrence loci; additional absent-local, wrong-kind and wrong-role controls have the same corresponding refusal. SCC-09 refuses `invalid_package`/`conflicting-definition` at the later row; SCC-11 refuses `invalid_package`/`invalid-value` there. SCC-10 and an omitted required index refuse that code/cause at `/schedulability_claims`. Each refusal compares the exact typed primary locus and exposes no admitted package.
- id: FR-038-AC-398
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented. Wire and graph/aggregate validation precede carrier joins; declaration precedes occurrence, repeated key precedes order, coverage follows all rows, and carrier validation precedes FR-482 body validation. Combined defects distinguish each adjacent check. Each visited row charges one existing validation visit before inspection; an exact measured budget admits the positive package, and one below the measured charge refuses with the existing incomplete/resource-exhaustion result at the first uncharged row and no partial typed claim view.
- id: FR-038-AC-399
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented. The reader admits the full FR-482 schedulable/response bodies and validates its shape, literal vocabulary, form fit, strict task order, common time unit and policy-specific task-set premises in that order after the protocol body step. QSpec FR-482-AC-3's adverse bodies, including negative/excess BCET, wrong time unit, duplicate fixed-priority priorities and the EDF equal-priority positive control, have their exact published codes, causes and loci. Existing graph refusals for nonempty dependencies, owner, semantic type or identity precede the dependent body checks. The typed claim/
- id: FR-038-AC-400
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented. TC-048 reads QSpec's `proposals/checked-package-v2/schedulability-claim-vectors.json`, TC-976 and FR-482/TC-967 cases by reference from a selected QSpec checkout, constructs complete IR packages around their resolved-node context, applies every carrier mutation and compares admission or exact typed code, cause and primary locus. The evidence states the selected checkout revision; no upstream schema, fixture or vector is copied into IR and the row vectors are not treated as complete packages. Complete public read/encode/read retains every identity row and the body, including both s
- id: FR-019-AC-19
  path: spec/model/functional/FR-019-rust-library-interface.md
  role: examined
  excerpt: Implemented. A default-feature external consumer imports `CheckedSchedulabilityClaimRef`, matches `ClaimForm::Schedulability`, and iterates an admitted package through `CheckedPackageV2::schedulability_claims`. Each view borrows `&CheckedNodeId`, `&CheckedOccurrence` with role `Claim`, and `&CheckedTerm` through the specified immutable accessors; the iterator reports the exact row count and preserves canonical order. The consumer exhaustively matches the specified closed kind/policy/arrival/criticality enums and reads task fields, optional response and HI-work values, optional priority and exa
- id: FR-051-AC-7
  path: spec/checked_package/functional/FR-051-encode-checked-package-v2-envelope.md
  role: examined
  excerpt: Implemented. An external producer encodes the complete SCC-01 and SCC-02 packages with `CheckedPackageSidecarsRef::schedulability_claims`; strict read and borrowed re-encode/read preserve the complete FR-482 body and every exact declaration-occurrence row, including both shared-node occurrences. The bytes equal the private envelope's canonical bytes at an exact measured byte limit; one byte less returns only the typed canonical limit error. `None` omits the index and `Some(empty)` encodes an empty array, with no writer normalization; malformed joins, missing coverage or reversed rows can be en
- id: FR-482-AC-1
  path: /home/peter/dev/worktrees/IR-1252-qspec/spec/objects/interfaces/FR-482-checked-package-schedulability-claim-body.md
  role: context_only
  excerpt: 'Over TC-372''s `Ctl` in a unit that declares the unit `ms` of the dimension `Time`, `schedulable CtlFp on Ctl under fixed_priority;` produces one `claim`/`schedulability` node whose body has exactly the bindings `kind`, `policy`, `task`, `bound` and `tasks` in that order, with `kind` `schedulable`, `policy` `fixed-priority`, `task` and `bound` the empty aggregate, and `tasks` the three task aggregates `t1`, `t2`, `t3`, each with its eleven bindings: `arrival` `periodic`, `period` `4`, `6`, `12`, `deadline` equal to the period, `bcet` `0`, `wcet` `1`, `2`, `5`, `wcet_hi` the empty aggregate, `cr'
- id: FR-482-AC-2
  path: /home/peter/dev/worktrees/IR-1252-qspec/spec/objects/interfaces/FR-482-checked-package-schedulability-claim-body.md
  role: context_only
  excerpt: '`CtlFp` and `schedulable CtlEdf on Ctl under edf;` have different node ids and different obligation identities; `CtlFp` over `Ctl` with `t3`''s `wcet` 6 ms or `bcet` 1 ms has a different node id and obligation identity; `CtlFp` over `Ctl` with its tasks listed `t3`, `t1`, `t2`, with `jitter 0 ms` added to `t1`, with other whitespace and comments, or with the claim named `Other` has `CtlFp`''s node id and obligation identity; one unit declaring `CtlFp`''s claim twice, as `CtlFp` and `CtlFp2`, has one node with two `claim` occurrences, ordinals 0 and 1.'
- id: FR-482-AC-3
  path: /home/peter/dev/worktrees/IR-1252-qspec/spec/objects/interfaces/FR-482-checked-package-schedulability-claim-body.md
  role: context_only
  excerpt: 'Fed to the reader, each edit of `CtlFp`''s node refuses with the code at the locus stated: `tasks` with no member, `invalid_semantic_graph` at `tasks`; `policy` and `kind` swapped in order, `invalid_semantic_graph` at the body; `policy` `rate-monotonic`, `invalid_package`/`invalid-value` at `/semantic_graph/nodes/{n}/body/members/1`; `kind` `response` with `task` still the empty aggregate, `invalid_semantic_graph` at the `task` binding; `t1`''s `wcet_hi` a `quantity` literal with `criticality` `lo`, `invalid_semantic_graph` at `t1`''s `wcet_hi` binding; tasks in the order `t2`, `t1`, `t3`, `inval'
- id: FR-482-AC-4
  path: /home/peter/dev/worktrees/IR-1252-qspec/spec/objects/interfaces/FR-482-checked-package-schedulability-claim-body.md
  role: context_only
  excerpt: '`CtlFp`''s exact local claim `NodeRef` and occurrence are carried by one `schedulability_claims` row; when the same claim node has two `claim` occurrences, both occurrence keys are present once in canonical order. The IR read returns the FR-482 body and identity by reference. It does not interpret the timed model''s release bindings or task successors; QSL alone performs those joins and recomputations under FR-418. Every positive and malformed carrier row has the outcome and locus in TC-976.'
- id: FR-322-AC-300
  path: /home/peter/dev/worktrees/IR-1252-qspec/spec/objects/interfaces/FR-322-checked-package-artifact.md
  role: context_only
  excerpt: A package with no `claim`/`schedulability` occurrence omits `schedulability_claims`; a package with such occurrences contains exactly one closed `{declaration, occurrence}` row for each, in structural-`NodeRef`, then occurrence order. Each row resolves only to its local `claim`/`schedulability` node and one of that node's `claim` occurrences; it copies no claim body, task value, source release binding, policy or task-transition template. The positive and shared-node/multiple-occurrence vectors are schema-valid. Missing coverage, a foreign, absent or wrong-form node, an unowned occurrence, a re
```

## Changed files examined

- crates/quire-contract-model/src/checked_package/shared.rs
- crates/quire-contract-model/src/checked_package/v2/callable.rs
- crates/quire-contract-model/src/checked_package/v2/comparison_operands.rs
- crates/quire-contract-model/src/checked_package/v2/contract_package.rs
- crates/quire-contract-model/src/checked_package/v2/exact_projection.rs
- crates/quire-contract-model/src/checked_package/v2/flat_wire.rs
- crates/quire-contract-model/src/checked_package/v2/identity.rs
- crates/quire-contract-model/src/checked_package/v2/intake.rs
- crates/quire-contract-model/src/checked_package/v2/lower.rs
- crates/quire-contract-model/src/checked_package/v2/lower/ceiling_tests.rs
- crates/quire-contract-model/src/checked_package/v2/mod.rs
- crates/quire-contract-model/src/checked_package/v2/model_fields/tests.rs
- crates/quire-contract-model/src/checked_package/v2/operations.rs
- crates/quire-contract-model/src/checked_package/v2/parameter_declaration.rs
- crates/quire-contract-model/src/checked_package/v2/scalar_operands.rs
- crates/quire-contract-model/src/checked_package/v2/schedulability.rs
- crates/quire-contract-model/src/checked_package/v2/structural.rs
- crates/quire-contract-model/src/checked_package/v2/timed_model.rs
- crates/quire-contract-model/src/checked_package/v2/vocabulary.rs
- crates/quire-contract-model/src/lib.rs
- spec/checked_package/functional/FR-038-consume-checked-package-v2.md
- spec/checked_package/functional/FR-051-encode-checked-package-v2-envelope.md
- spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md
- spec/checked_package/matrix/TC-451-typed-v2-envelope-encoding.md
- spec/model/functional/FR-019-rust-library-interface.md
- tests/conformance_qspec/main.rs
- tests/conformance_qspec/probabilistic_claims.rs
- tests/conformance_qspec/projection_read_form.rs
- tests/conformance_qspec/schedulability.rs
- tests/conformance_qspec/single_body.rs
- tests/conformance_qspec/temporal_clock_bindings.rs
- tests/conformance_qspec/timed_model.rs
- tests/conformance_qspec/transition.rs
- tests/it/application_node_identity.rs
- tests/it/checked_package_v2_canonical_encoding.rs
- tests/it/checked_package_v2_claim_terminal.rs
- tests/it/checked_package_v2_dimension_equivalence.rs
- tests/it/checked_package_v2_envelope.rs
- tests/it/checked_package_v2_frame_entries.rs
- tests/it/checked_package_v2_identity_digests.rs
- tests/it/checked_package_v2_model_root.rs
- tests/it/checked_package_v2_node_map.rs
- tests/it/checked_package_v2_refinement.rs
- tests/it/checked_package_v2_schedulability.rs
- tests/it/checked_package_v2_temporal.rs
- tests/it/checked_package_v2_term_data_writer.rs
- tests/it/checked_package_v2_typed_terms.rs
- tests/it/main.rs

## Disposition pass 1

Reviewed frozen head 9585f6fdfcb9663de8f75e5cf7f37c2cf7a82119, including the complete 325136e8..9585f6fd diff, helper fix 7266b054 and merge of main 3962a9b6. Production crates and src are byte-for-byte unchanged from the originally reviewed candidate. Both fixture lanes now call the single rebuild_complete_source_map in tests/it/support/checked_package.rs:1974. The original first-occurrence helper remains unchanged, and the shared complete helper supplies rows for every actual additional role/ordinal, including sparse ordinals. No test oracle, negative mutation or admission condition was weakened.

The merge conflict resolution keeps implemented FR-019-AC-19 and main's planned FR-019-AC-20; both statuses and the public-item inventory are consistent. Main's FR-038-AC-401 through AC-405 remain planned. The Makefile is identical to main 3962a9b6, including its exact one-test positive-package guard and nonzero subprocess propagation. No new review finding was found in the fix or conflict resolution.

Inspected supplied integration logs: 10 focused IT, 6 QSpec conformance groups and 1 affected positive-package guard pass; workspace clippy, fmt, string-edge and scoped specification validation complete successfully. Spec validation includes existing grammar warnings and provider diagnostics, not an error. Default-model clippy evidence remains the earlier unchanged-production run. No reviewer Cargo, tests, builds or source edits were performed. Full aggregate/full IT remain the lead's final-head gate.
