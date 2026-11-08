---
id: SR-202611080004002
title: IR remediation batch 1 (PR 954) gap analysis
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; PR 954 head b3363171 vs main; FR-038-AC-495..AC-529, FR-358-AC-13/14, FR-354-AC-13; crates/quire-contract-model/src/checked_package/**, tests/it/checked_package_v2_budgeted_read.rs, tests/conformance_qspec/{node_index,carried_quantity}.rs, spec/checked_package/matrix/tests.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: reviews
---

## Summary

Tickets: IR-1404 (stage 1), IR-1396, IR-1405. Planless gap analysis over the computed Test Matrix (`quire matrix --format tsv`, quire 0.38.3) at head b3363171. Plan completion: not assessed.

The renumbering is complete. The matrix binds IR-1396 AC-519, IR-1405 AC-520..523 and IR-1398 AC-524..529 to tests, and no code or test text added in this batch still carries an old AC-495..498 or AC-515..520 tag for those criteria. The only AC-495..518 tags in added code are IR-1404's own criteria, which merged main owns. Old numbers survive only in the committed review records of PRs 946 and 949, which are historical and correct as written.

IR-1404 stage-1 criteria in the matrix: AC-495, AC-496, AC-497, AC-498, AC-508, AC-510, AC-514 and AC-515 are tagged. AC-499..507, AC-509, AC-511, AC-512, AC-516 and AC-517 are untagged, and AC-513 is method-without-symbol. The untagged stage tables (AC-499..507) and whole-read accounting (AC-511/512/516) belong to later stages and are not findings here.

## Verdict

Coverage gaps on stage-1 code: the shared-reader grammar is now the production envelope, but AC-509 has no test, and two tagged criteria are overclaimed (see SR-202611080004001 FND-004). The matrix status of the remaining stage-1 tags is real.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-038-AC-509 (strict grammar and precedence under the shared reader) is untagged, but this batch makes `quire_canonical::read_with_reservation` the production V2 envelope and cites AC-509 for its order (budgeted_read.rs:497). The old precedence (strict shape scan, then the 128-depth refusal, then canonical tokens) is replaced: shared reader, then `nesting_depth`, then the canonical comparison. No test pins BOM, lone-surrogate, duplicate-then-truncation, or depth-128-versus-noncanonical precedence on the new path. Scenario: the shared reader's duplicate-at-close order differs from the old scan for one QSpec adverse vector, and nothing in the batch fails. | crates/quire-contract-model/src/checked_package/v2/budgeted_read.rs:497-534 |
| FND-002 | low | spec/checked_package/matrix/tests.md still lists `FR-038-AC-495 through FR-038-AC-517 / TC-472` as Planned (line 18), though AC-495..498, 508, 510, 514 and 515 now have tagged tests. It also has a row for AC-519 (line 153) but none for AC-520..529. | spec/checked_package/matrix/tests.md:18 |

## Coverage

- FR-038-AC-519..529, examined: every criterion is tagged (node_index.rs, carried_quantity.rs, operand_families.rs, operation_catalog.rs, operations.rs).
- FR-038-AC-495, AC-496, AC-508, examined: tagged; the oracle compares the budgeted and unbudgeted results and the work delta against `all_families_read_work()`.
- FR-038-AC-497, AC-498, AC-510, AC-515, examined: tagged; the overclaims are recorded in SR-202611080004001.
- FR-038-AC-514, examined: tagged by checked_json.rs unit tests only (see SR-202611080004001 FND-005).
- FR-358-AC-13/14, examined: tagged through the budgeted_read tests.
- FR-354-AC-13, examined: the converted D10 test. Independent check: resolving the converted d10-package.json operand slots through its `operand_rows` gives back origin/main's emitted index-form fixture exactly (34 nodes, value equality).
- Pre-existing, not this batch: origin/main has two documents with `id: TC-472` (TC-472-inherited-operation-dispatch.md and TC-472-strict-v2-read-exact-budget-charging.md).

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | 37a92011 adds `the_shared_reader_orders_first_faults_on_the_v2_envelope`, tagged FR-038-AC-509, on the V2 envelope. It checks invalid UTF-8 after a repeated member, a repeat then truncation, a repeat closed before a later fault (`duplicate_member` at /x/a), and a grammar fault before `noncanonical_wire`. |
| FND-002 | fixed | tests.md now splits stage group 1 (AC-495..498, 508, 509, 510, 515: Implemented) from the later stages (Planned), and adds rows for AC-520..523 and AC-524..529. |
