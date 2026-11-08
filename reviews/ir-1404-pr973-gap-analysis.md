---
id: SR-202611070951309528
title: "IR-1404 PR 973 gap-analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; IR-1404; tests/it/checked_package_v2_budgeted_read.rs, tests/it/checked_package_v2_artifact_refs.rs, tests/it/admitted_result_context.rs"
review_set: subset
---

## Summary

Ticket: IR-1404. Independent final review of draft PR 973.

## Verdict

CONDITIONAL — no new scoped gap; existing repository assurance gaps and explicitly deferred proof remain.

The frozen diff changes exactly three integration test files. The AC-510 expected denied amount is derived from public typed record and single location layouts plus owned pointer bytes, without calling the production charging helper or reading the denied amount. Assertions exclude additional owned contexts. The exact and one-byte-short boundaries, denied resource and amount, configured bound, locus and pending refusal code remain. A charging change that omits a record/location or charges another amount fails the denied-amount assertion. The cumulative baseline is measured as before; this is not an independent whole-read physical allocation proof.

Independent byte comparison after removing the eight exact ignore attributes reproduces both parent files byte-for-byte. All eight receipt logs stop at read_observation.rs:87 physical accounting, before their semantic oracle. Each ignore names IR-1404 and deferred AC-511. Eight new registrations are present in the supplied ignored listing; the listing has thirteen total, including five pre-existing ignores. Makefile line 81 uses --include-ignored, so these attributes do not make aggregate qualification green. Semantic success for the eight is unverified.

The user-authorized scope defers AC-511 physical proof, AC-513 static closure and F1 migration. These remain outstanding; this review does not certify them. No production code, gate script, dependency, safety, concurrency, wire contract or API changes. No new vendoring, duplicated implementation or stubs. No applicable AssuranceProfile found.

Evidence inspected: original red 08.log (303 versus 15), forward-budget-green.log (one passed), forward-fmt.log, forward-ignored-list.log and all eight physical failure receipts. git diff --check passed independently. No Cargo tests, Clippy, deny, Kani or aggregate rerun: explicit merge-first/aggregate-after instruction controls those gates; unrelated warning repair is PR 972. This review uses supplied focused execution evidence, not a claim of a full workspace gate pass.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Plan completion: not assessed

Code-review includes Rust-review lane. Spec-review/sub-analyses are inapplicable: no spec or plan change. Gap-analysis inspected computed Quire 0.38.1 matrix; repository counts {'tagged': 1022, 'untagged': 96, 'tagged-by-ignored-test': 9, 'method-without-symbol': 26}. No execution evidence is asserted by the matrix. The existing FR-322-AC-294 tag on two deferred controls does not resolve in the computed criterion index; it is unchanged and is not recorded as a verified binding. No new behavior needs reverse ownership; existing refusal behavior belongs to FR-038. Optional repository semantic audit was not run; focused oracle independence review was explicitly requested.

## Examined scope

```yaml
scope:
- id: an_unaffordable_refusal_diagnostic_is_exhausted_with_the_pending_refusal
  path: tests/it/checked_package_v2_budgeted_read.rs
  role: examined
  excerpt: '#[trace("TC-472", "FR-038-AC-510")]

    #[test]

    fn an_unaffordable_refusal_diagnostic_is_exhausted_with_the_pending_refusal'
- id: FR-038-AC-510
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: On a budgeted entry, a work denial whose diagnostic is affordable returns
    `BudgetIncomplete` with `denied` `Work` and its amount, the budget's configured
    work limit, cumulative work plus the denied amount, and the value's pointer; a
    read-storage denial likewise returns `BudgetIncomplete` with `denied` `ReadStorage`.
    When the diagnostic's own request is denied, or the denied request was a refusal
    diagnostic, the read returns `Exhausted` holding the original denied resource
    and amount, its configured bound (including zero), its `PackageRead` locus and
    the pending outcome class (refusal code or s
- id: tc_048_a_definition_reference_with_a_digest_member_is_an_unknown_member
  path: tests/it/checked_package_v2_artifact_refs.rs
  role: examined
  excerpt: '#[trace("TC-048", "FR-038-AC-48")]

    #[test]

    #[ignore = "IR-1404: deferred AC-511 whole-read allocation proof"]

    fn tc_048_a_definition_reference_with_a_digest_member_is_an_unknown_member'
- id: FR-038-AC-48
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: PLANNED/UNRUN (IR-1141). A definition reference at a lock, diagnostics.catalog
    or operation-law site carrying revision, digest_domain, digest or export refuses
    unknown_member at the first extra member in document order and returns no package;
    the earlier five-member shape also refuses at its first extra member.
- id: tc_048_a_clock_row_precedes_a_malformed_temporal_law_definition
  path: tests/it/checked_package_v2_artifact_refs.rs
  role: examined
  excerpt: '#[trace("TC-048", "FR-038-AC-49", "FR-322-AC-294")]

    #[test]

    #[ignore = "IR-1404: deferred AC-511 whole-read allocation proof"]

    fn tc_048_a_clock_row_precedes_a_malformed_temporal_law_definition'
- id: FR-038-AC-49
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: An `operation.laws[].definition` that carries a member beyond `authority`
    and `identity`, lacks one or holds one of the wrong kind refuses as `invalid_semantic_graph`
    at that member, or at the `definition` when a member is absent, as every closed
    shape inside an `operation` does (FR-038-AC-36), never as `unknown_member` or
    `malformed_wire`.
- id: tc_048_a_source_reference_with_an_extra_member_is_an_unknown_member
  path: tests/it/checked_package_v2_artifact_refs.rs
  role: examined
  excerpt: '#[trace("TC-048", "FR-038-AC-50")]

    #[test]

    #[ignore = "IR-1404: deferred AC-511 whole-read allocation proof"]

    fn tc_048_a_source_reference_with_an_extra_member_is_an_unknown_member'
- id: FR-038-AC-50
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A `lock.sources` row, `source_map` region `source` or `diagnostics.entries[].loci[].source`
    that carries `revision`, `digest_domain`, `digest` or `export` refuses as `unknown_member`
    at that member and returns no package.
- id: tc_048_a_definition_reference_member_that_is_absent_empty_or_not_a_string_is_malformed
  path: tests/it/checked_package_v2_artifact_refs.rs
  role: examined
  excerpt: '#[trace("TC-048", "FR-038-AC-51")]

    #[test]

    #[ignore = "IR-1404: deferred AC-511 whole-read allocation proof"]

    fn tc_048_a_definition_reference_member_that_is_absent_empty_or_not_a_string_is_malformed'
- id: FR-038-AC-51
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: PLANNED/UNRUN (IR-1141). A definition reference at a lock, diagnostics.catalog
    or operation-law site whose authority or identity is absent or not a string refuses
    malformed_wire at the value or reference, and an empty value refuses malformed_wire
    at that member.
- id: tc_048_a_lock_source_digest_member_is_unknown_before_row_validation
  path: tests/it/checked_package_v2_artifact_refs.rs
  role: examined
  excerpt: '#[trace("TC-048", "FR-038-AC-53")]

    #[test]

    #[ignore = "IR-1404: deferred AC-511 whole-read allocation proof"]

    fn tc_048_a_lock_source_digest_member_is_unknown_before_row_validation'
- id: FR-038-AC-53
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A `lock.sources` row carrying `digest_domain` or `digest` refuses as `unknown_member`
    at the first extra member in document order, even when an `authority` or `identity`
    value is empty; no source digest is checked or retained.
- id: tc_048_a_lock_source_refuses_an_empty_authority_or_identity
  path: tests/it/checked_package_v2_artifact_refs.rs
  role: examined
  excerpt: '#[trace("TC-048", "FR-038-AC-54")]

    #[test]

    #[ignore = "IR-1404: deferred AC-511 whole-read allocation proof"]

    fn tc_048_a_lock_source_refuses_an_empty_authority_or_identity'
- id: FR-038-AC-54
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A `lock.sources` row with an empty `authority` or `identity` refuses as
    `malformed_wire` at that member.
- id: tc_048_a_clock_row_precedes_an_unselected_temporal_law
  path: tests/it/checked_package_v2_artifact_refs.rs
  role: examined
  excerpt: '#[trace("TC-048", "FR-038-AC-57", "FR-322-AC-294")]

    #[test]

    #[ignore = "IR-1404: deferred AC-511 whole-read allocation proof"]

    fn tc_048_a_clock_row_precedes_an_unselected_temporal_law'
- id: FR-038-AC-57
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: An `operation.laws[].definition` of a profile role (`temporal_profile`
    or `protocol_profile`) admits only when its `{authority, identity}` pair equals
    the pair of the lock's `profile_selections` row of that role; any other pair refuses
    `operation-law-unselected` at that law's `definition`.
- id: tc_048_numeric_narrow_result_equals_type_argument
  path: tests/it/admitted_result_context.rs
  role: examined
  excerpt: '#[trace("TC-048", "FR-038-AC-247", "FR-038-AC-250")]

    #[test]

    #[ignore = "IR-1404: deferred AC-511 whole-read allocation proof"]

    fn tc_048_numeric_narrow_result_equals_type_argument'
- id: FR-038-AC-247
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'Implemented (IR-225). Through `CheckedPackageV2::read`, a catalogued `result:
    member`/`member: type_argument` application whose member has a valid `declaration`
    admits only when `result_type` equals that declaration, regardless of whether
    its family resolves or the entry has a `member_family` constraint. A mismatch
    refuses `ill_typed`/`operator-ineligible` at `/semantic_graph/nodes/{i}/body/result_type`,
    with the application node as locus. `collection.fold` and `collection.reduce`
    admit equal results without a family constraint and refuse unequal results. `model.lookup`
    and `model.all_instance'
- id: FR-038-AC-250
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented (IR-225). Through the public reader, valid `quire.op.numeric.narrow`
    and `quire.op.quantity.convert` applications each admit with equal member declaration
    and `result_type`, and each refuses a mismatched `result_type` as `ill_typed`/`operator-ineligible`
    at `/semantic_graph/nodes/{i}/body/result_type` with the application node as locus,
    after re-deriving affected identities. These two non-IEEE catalogue entries demonstrate
    entry-driven selection beyond the IEEE conversion identities. The `field`, `position`,
    `element`, `relationship_end` and `operation` member-result kinds do not a
- id: FR-038-AC-511
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: context_only
  excerpt: "Whole-read accounting, including each read's owned catalog. A test binary\
    \ whose global allocator counts, on the reading thread during the call, the size\
    \ of each allocation request and the new size of each reallocation request reads,\
    \ under an unlimited budget, an exact budget and each one-below budget of AC-497\
    \ through AC-507: every positive TC-048 fixture, the QSpec checked-package conformance\
    \ packages this repository reads, and the adverse shapes \u2014 the 100000-node\
    \ reference chain, a 10000-node flat graph, a node with 10000 dependencies, a\
    \ 1 MiB string member, 10000 source-map entries, 10000 d"
- id: FR-038-AC-513
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: context_only
  excerpt: The V2 read path (`CheckedPackageV2::read` from entry through returning
    its result, including the owned catalog, and every function it reaches) calls
    no `serde_json` parser, deserializer or `RawValue`, constructs no `serde_json::Value`,
    calls no `num-bigint` parse or arithmetic, sorts only with `sort_unstable*`, and
    builds no `BTreeMap`, `BTreeSet`, `HashMap` or `serde_json::Map`; its tables are
    sorted exact-capacity slices and its containers are reserved at exact capacity
    before they are filled. Every read parses the catalog with the shared `quire_canonical`
    reader and charges its own catalog
bindings:
- test_id: an_unaffordable_refusal_diagnostic_is_exhausted_with_the_pending_refusal
  ac_id: FR-038-AC-510
  trace: correct
- test_id: tc_048_a_definition_reference_with_a_digest_member_is_an_unknown_member
  ac_id: FR-038-AC-48
  trace: correct
- test_id: tc_048_a_clock_row_precedes_a_malformed_temporal_law_definition
  ac_id: FR-038-AC-49
  trace: correct
- test_id: tc_048_a_source_reference_with_an_extra_member_is_an_unknown_member
  ac_id: FR-038-AC-50
  trace: correct
- test_id: tc_048_a_definition_reference_member_that_is_absent_empty_or_not_a_string_is_malformed
  ac_id: FR-038-AC-51
  trace: correct
- test_id: tc_048_a_lock_source_digest_member_is_unknown_before_row_validation
  ac_id: FR-038-AC-53
  trace: correct
- test_id: tc_048_a_lock_source_refuses_an_empty_authority_or_identity
  ac_id: FR-038-AC-54
  trace: correct
- test_id: tc_048_a_clock_row_precedes_an_unselected_temporal_law
  ac_id: FR-038-AC-57
  trace: correct
- test_id: tc_048_numeric_narrow_result_equals_type_argument
  ac_id: FR-038-AC-247
  trace: correct
- test_id: tc_048_numeric_narrow_result_equals_type_argument
  ac_id: FR-038-AC-250
  trace: correct
```
