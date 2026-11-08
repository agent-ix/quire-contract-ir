---
id: SR-202611070951309540
title: PR987 population key code-review
type: SpecReview
analysis: code-review
scope: agent-ix/quire-contract-ir; IR-1404; model_members.rs; model_members/tests/budgeted_charge.rs
review_set: subset
---

## Summary

Ticket: IR-1404. Reviewed frozen PR987 at f3ba8fd409a05d16971f85e86415b32990d5f25d against main878ee94b. The selected-model reader now pays the second retained population identity via existing Budget::text instead of an uncharged Box<str> clone. The first key payment stays before the meaning check; the second copy still occurs only after a Population meaning succeeds, before the duplicate insertion check. Members and both sorted indexes retain the same values. Budget::text adds only read storage, not logical work. No API, new allocator/budget seam, copied algorithm, dependency, workflow, spec, ignore or threshold changes were introduced.

## Verdict

CONDITIONAL: one confined trace-binding finding must be fixed before merge. Production allocation behavior and retained runtime evidence pass; no introduced production defect was found.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The new selected-model allocation test binds FR-038-AC-497, but AC497 requires the Envelope/package_bytes reservation and reader list classes. This test calls admit_selection directly at ModelDocument position3 and cannot exercise that behavior. Remove the new AC497 tag; keep only the truthful TC472/partial AC511 bindings instead of crediting unrelated envelope evidence. | crates/quire-contract-model/src/checked_package/v2/model_members/tests/budgeted_charge.rs:142 |

## Coverage

Inspected actual native admit_selection allocation control PASS1 (both retained indexes, allocated==reserved, ample/exact/one-byte-short/zero/repeated reads and ModelDocument locus), unchanged owners::tc_229_declared_population_is_query_operand PASS1 with physical and semantic assertions (0.34s), selected Rust1.98.1 workspace all-target Clippy conformance-cli,test-support -Dwarnings PASS16.72s, and scoped formatting/diff PASS. The initial abbreviated exact selector ran zero tests and is compilation-only; it is not credited. No Cargo or aggregate was run by this reviewer. Native assertions exercise real production selection and the actual allocator counter; no mock replaces the reader. No new panic, unsafe code, recursion, asynchronous work or hidden resource owner is introduced. No applicable AssuranceProfile was found under spec/. Whole-corpus AC511 and AC513 remain UNPROVEN.

The selected Quire0.38.1 computed matrix was inspected in full: {'tagged': 1028, 'untagged': 92, 'tagged-by-ignored-test': 8, 'method-without-symbol': 26}. Existing global unresolved rows remain unclaimed. Reverse inspection grounds the second-key payment in FR-038 budget-charged strict read, with the unchanged population-query behavior owned by AC305. The native test gives partial accounting evidence under AC511, but its additional AC497 binding is wrong-stage evidence and must be removed. It does not run the envelope package reader or deny Envelope/package_bytes. Do not replace it with AC503: this one-selected-model test does not prove that whole criterion either. Optional expanded semantic audit was not requested; ordinary production/test/spec agreement was checked. No plan supplied.

Plan completion: not assessed

Full reviewed scope:

```yaml
scope:
- id: model_members::read_semantic_ir population loop
  path: crates/quire-contract-model/src/checked_package/v2/model_members.rs
  role: examined
  excerpt: let population_key = budget.text(node, F)?;
- id: Budget::text
  path: crates/quire-contract-model/src/checked_package/v2/model_members.rs
  role: examined
  excerpt: self.request(text.len(), form)?;
- id: population_keys_are_paid_before_both_retained_copies_are_allocated
  path: crates/quire-contract-model/src/checked_package/v2/model_members/tests/budgeted_charge.rs
  role: examined
  excerpt: '#[trace("TC-472", "FR-038-AC-497", "FR-038-AC-511")]'
- id: FR-038-AC-497
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'Envelope stage. For a genuine admitted package and for the adverse shapes
    of FR-038-AC-511, a read-storage limit equal to the read''s measured requirement
    admits the package, and a read-storage limit one below the cumulative read storage
    admitted through the shared reader''s last package-bytes reservation refuses at
    that reservation: first unpaid `ReadStorage` with the reservation''s byte amount,
    locus `PackageRead(Envelope, 0)` and form `"package_bytes"`, no document and no
    package. Each reader list class (arena nodes, child ids/members, decoded text,
    open-container stack and children, duplicate'
- id: FR-038-AC-511
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: "Whole-read accounting, including each read's owned catalog. A test binary\
    \ whose global allocator counts, on the reading thread during the call, the size\
    \ of each allocation request and the new size of each reallocation request reads,\
    \ under an unlimited budget, an exact budget and each one-below budget of AC-497\
    \ through AC-507: every positive TC-048 fixture, the QSpec checked-package conformance\
    \ packages this repository reads, and the adverse shapes \u2014 the 100000-node\
    \ reference chain, a 10000-node flat graph, a node with 10000 dependencies, a\
    \ 1 MiB string member, 10000 source-map entries, 10000 d"
- id: FR-038-AC-305
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: '`quire.op.model.all_instances` and `quire.op.model.lookup` each admit
    a reference to a declared population as operand 0 and list it in dependencies;
    the same declared population is one reference node across both reads, while the
    two named populations yield distinct reference and application paths within the
    same named package. A `value`/`parameter` typed `Population<O>[N]` remains an
    admitted operand 0. A unit maximum is represented only in the application''s result-type
    collection bounds, not in the declared population node.'
```
