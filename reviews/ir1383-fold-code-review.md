---
id: SR-202611070951309498
title: "IR-1383 fold binders \u2014 code-review"
type: SpecReview
analysis: code-review
scope: agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/v2/operations.rs;
  tests/it/checked_package_v2_operand_families.rs; spec/checked_package/functional/FR-038-consume-checked-package-v2.md;
  spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md
review_set: subset
---

## Summary

Ticket: IR-1383. Reviewed frozen PR quire-contract-ir#961 at 42002f2b587ce1a066821d301a666452a5d1476e against origin/main. No applicable AssuranceProfile exists under spec/.

## Verdict

PASS for the scoped frozen diff — no defects found. The code lead retains the aggregate gate.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

The exact operation identity gates the singleton aggregate/element binding projection. Other operations retain the direct binder path. No generic aggregate typing, API exposure, dependency or CI gate changed. `operand_type_node_typed` still returns None for aggregates and bindings. The slice pattern is constant work and borrows the members; it allocates no vector or traversal stack. The only returned type-ID clone uses the existing operand-resolution path. The two extra edges are charged before descent; refusal propagates from WorkMeter before the check. Direct binder behavior is unchanged.

Empty/multiple wrappers and non-binding members return no bound, causing application-local ill_typed/operator-ineligible. Boolean final steps fail the exact member comparison. Typed-term validation rejects a further aggregate step earlier with malformed_wire, no cause, at its value. The operand check still precedes result equality, and the existing direct control checks the result_type locus. A malformed outer accumulator or missing element cannot satisfy the Binding patterns.

Non-mutating old-behavior proof: origin/main binder_bound passes the outer binder's Aggregate directly to operand_type_node_typed; its exhaustive term projection accepts Literal/Application/Reference and returns None for Aggregate. Both genuine producer fixtures contain that outer Aggregate, so the former implementation necessarily fails their mandatory Admitted arm. The new tests cannot pass vacuously: unexpected admission/refusal shapes panic, both binder names and singleton wrapper are checked, and the referenced step semantic type must equal the read application result.

The original monitor fixture compares byte-for-byte with /tmp/IR-1383-monitor-base.json. It retains 1114 nodes and selected model evidence. The user explicitly requested these genuine producer fixture bytes; their inclusion is the authorized regression scope, not an unsolicited copied implementation or schema. No upstream implementation is embedded.

## Examined scope

```yaml
scope:
- id: FR-038
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'For `quire.op.collection.fold` and `quire.op.collection.reduce`, the reader

    SHALL resolve `bound_is_member` through [QSL FR-093](https://github.com/agent-ix/quire-spec-language/blob/main/spec/functional/FR-093-lower-checked-value-expressions-to-fr-322-terms.md)''s

    two-binder encoding in [QSpec FR-322](https://github.com/agent-ix/quire-specification/blob/main/spec/objects/interfaces/FR-322-checked-package-artifact.md) terms: an accumulator `binding` whose value is an `aggregate`

    containing exactly one element `binding`, whose value names the final step.'
- id: FR-038
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: The final step's resolved type SHALL equal the `type_argument` declaration.
- id: FR-038
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'An empty or multiple-member aggregate, a non-binding member, or a final step

    of a different type SHALL refuse `ill_typed`/`operator-ineligible` at the

    application node.'
- id: FR-038
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'A further nested aggregate step retains the earlier

    `malformed_wire` refusal at that step''s value, without a cause.'
- id: FR-038
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'Direct single-binder

    bounds retain their existing meaning.'
- id: FR-038
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'This rule does not assign a type to

    arbitrary aggregate operands or change other operations'' binder resolution.'
- id: FR-038
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'FR-038-AC-247 and FR-038-AC-251 include these positive and negative controls

    and strict admission of both genuine Integer-only QSL fold bytes and the

    complete original monitor-base document with its selected model evidence.

    '
- id: FR-038-AC-247
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'Implemented (IR-225). Through `CheckedPackageV2::read`, a catalogued `result: member`/`member: type_argument` application whose member has a valid `declaration` admits only when `result_type` equals that declaration, regardless of whether its family resolves or the entry has a `member_family` constraint. A mismatch refuses `ill_typed`/`operator-ineligible` at `/semantic_graph/nodes/{i}/body/result_type`, with the application node as locus. `collection.fold` and `collection.reduce` admit equal results without a family constraint and refuse unequal results. `model.lookup` and `model.all_instance'
- id: FR-038-AC-251
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'Implemented (IR-225; amended from an expected-set shape guard because the reader selects from the current catalogue entry). Member-result checking is selected by the catalogue entry shape alone (`result: member` and `member: type_argument`). Every other catalogued operation, including `model.lookup`, `model.all_instances` and the other member kinds, keeps its pre-IR-225 admission outcome; `collection.fold` and `collection.reduce`, which have no `member_family`, are checked for result-type/declaration equality only. A catalogue-wide differential test reads every entry from the same published ca'
- id: TC-048
  path: spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md
  role: examined
  excerpt: 'For fold and reduce, also read the accumulator binding whose aggregate holds

    one element binding and resolve the final step''s type.'
- id: crates/quire-contract-model/src/checked_package/v2/operations.rs
  path: crates/quire-contract-model/src/checked_package/v2/operations.rs
  role: examined
  excerpt: fn binder_bound(
- id: tests/it/checked_package_v2_operand_families.rs
  path: tests/it/checked_package_v2_operand_families.rs
  role: examined
  excerpt: fn nested_fold_and_reduce_binders_resolve_only_the_single_final_step() {
- id: genuine_integer_fold_with_both_authored_binders_is_admitted
  path: tests/it/checked_package_v2_operand_families.rs
  role: examined
  excerpt: fn genuine_integer_fold_with_both_authored_binders_is_admitted() {
- id: genuine_monitor_base_fold_is_admitted_without_removing_semantics
  path: tests/it/checked_package_v2_operand_families.rs
  role: examined
  excerpt: fn genuine_monitor_base_fold_is_admitted_without_removing_semantics() {
- id: fold_and_reduce_admit_without_member_family_and_refuse_unequal_result
  path: tests/it/checked_package_v2_operand_families.rs
  role: examined
  excerpt: fn fold_and_reduce_admit_without_member_family_and_refuse_unequal_result() {
- id: tests/fixtures/fold-binders/README.md
  path: tests/fixtures/fold-binders/README.md
  role: examined
  excerpt: '# Fold binder producer fixture


    `integer-fold.json` is genuine QSL checked-package output from

    `qsl-package/src/emit/tests.rs`, `fold_with_both_authored_binders_reads_back`.

    It was emitted without mod'
- id: tests/fixtures/fold-binders/integer-fold.json
  path: tests/fixtures/fold-binders/integer-fold.json
  role: examined
  excerpt: '{"capability_report":[{"disposition":"available","feature":"quire.value.complete/v1"}],"contract_version":"quire.checked-package/v2","diagnostics":{"catalog":{"authority":"agent-ix","identity":"quire.'
- id: tests/fixtures/fold-binders/monitor-base.json
  path: tests/fixtures/fold-binders/monitor-base.json
  role: examined
  excerpt: '{"capability_report":[{"disposition":"available","feature":"quire.value.complete/v1"}],"checked_term_data":{"dispatch_tables":[],"equality_decisions":[{"application":{"ordinal":0,"package":"fixture","'
- id: tests/fixtures/fold-binders/monitor-model.json
  path: tests/fixtures/fold-binders/monitor-model.json
  role: examined
  excerpt: '{"canonicalization":{"algorithm":"rfc8785-v1"},"constructs":[{"construct":{"identity":"none","meaning":"quire.meaning.model.object-type/v1","members":{},"shape":"record"},"kind":{"module":"example/con'
- id: tests/fixtures/fold-binders/monitor-base.json.license
  path: tests/fixtures/fold-binders/monitor-base.json.license
  role: examined
  excerpt: 'SPDX-License-Identifier: AGPL-3.0-only

    '
- id: tests/fixtures/fold-binders/monitor-model.json.license
  path: tests/fixtures/fold-binders/monitor-model.json.license
  role: examined
  excerpt: 'SPDX-License-Identifier: AGPL-3.0-only

    '
```

## Validation

Actual reviewer runs: nested malformed/mismatched control, genuine Integer, complete monitor, direct binder/result control; formatting and scoped Clippy. Logs: /tmp/ir1383-review-{nested,integer,monitor,direct,fmt,clippy-model,clippy-test}.log. Broad workspace tests, full lint, cargo-deny, make ci and Kani are intentionally reserved for the lead's single aggregate gate. The scoped commands use the prescribed lock, nice/ionice, finite build jobs and existing target.
