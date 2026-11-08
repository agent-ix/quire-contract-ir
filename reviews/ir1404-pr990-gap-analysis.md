---
id: SR-202611070951309545
title: PR990 public temporal consumer gap-analysis
type: SpecReview
analysis: gap-analysis
scope: agent-ix/quire-contract-ir; IR-1404; crates/quire-contract-model/tests/temporal_binding_public.rs
review_set: subset
---

## Summary

Ticket: IR-1404. Independently reviewed PR990 frozen 33437e005b016387d5794f5b6e5fcf87b5a3b868 against its main base caeef9b6. The change is one external default-feature test file. It expands the existing published CN-P2 <clause> NodeRef token with the fixture's already published temporal binding declaration instead of its graph-held operand-slot ID. The source clause-name and source-map references are checked for agreement, and the designated graph node remains checked as a temporal clause. No production decoding, range check, identity lookup, API, new fixture, normalization algorithm, mock, ignore, gate, dependency or schema is changed.

## Verdict

PASS for this fixture-consumer repair: no introduced finding. The changed test retains its genuine public reader/writer behavior and preserves strict document-held placement. This is not repository-wide assurance qualification.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Inspected the complete original test, the genuine published positive-timed-clause-nodes fixture and CN-P2 vector note/patches, FR-038 node-held/document-held placement rules, and the Makefile's explicit default-model consumer component. Graph node8 holds operand slots5/0/6/4, its row resolves those to document-held node indices27/23/29/0, and the existing binding/name/source-map declaration is exactly that document-held ID. The setup fix therefore uses an authoritative existing value rather than weakening the reader. The original expected CheckedNodeId, two borrowed binding/name rows, occurrence order, clock name/typed clock authority, and typed writer/read-back equality assertions are unchanged. The test still invokes the real public reader and writer; the new fixture-consistency assertions do not replace those production assertions. No applicable AssuranceProfile exists under spec/.

Retained focused evidence: selected Rust1.98.1 Makefile-authorized default-feature temporal_binding_public exact test PASS1, zero ignored/filtered, cold compile36.51s/test0.10s, exit0 receipt /tmp/ir1404-public-temporal-consumer-receipt.json and full /tmp/ir1404-public-temporal-consumer-test.log. The dispatch/runner evidence specifies QSPEC_CONFORMANCE=1 and the genuine QUIRE_SPECIFICATION_DIR, so the test's absent-fixture early-return path is not credited. Scoped formatting and diff checks pass. No Cargo command, aggregate, deferred-case replay or broad test was run by this reviewer. This review does not claim hosted CI green, whole-corpus AC511 or AC513.

Selected Quire0.38.1 computed matrix inspected: tagged1028, untagged92, tagged-by-ignored-test8, method-without-symbol26. Existing global unresolved rows remain unclaimed. This test's pre-existing FR-019-AC-22 binding supplies only the borrowed temporal-binding API portion; compensation rows and absence behavior remain other existing tests' responsibility. FR-019-AC-25 directly owns the two names/shared clause/public writer-reread assertions. No new trace binding or whole-criterion credit is added. Production placement criteria AC521/523 are context/representation checks, not newly claimed by this fixture-only change. No plan supplied. Optional expanded semantic audit was not requested; ordinary changed-test/fixture/spec agreement was examined.

Plan completion: not assessed

Full reviewed scope:

```yaml
scope:
- id: default_feature_consumer_borrows_admitted_clock_rows
  path: crates/quire-contract-model/tests/temporal_binding_public.rs
  role: examined
  excerpt: let clause = fixture["temporal_bindings"][0]["declaration"].clone();
- id: CN-P2-shared-node-two-names
  path: quire-specification/proposals/checked-package-v2/temporal-clause-name-vectors.json
  role: examined
  excerpt: The token "<clause>" in a patch value stands for the NodeRef of the base
    package's one temporal_clause node (graph index 8)
- id: FR-038-AC-521
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: placement of an index segment in a node-held id and of a slot in a document-held
    id
- id: FR-038-AC-523
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: '`document_held_id` returns a node''s id with index segments, the form
    a request names it by.'
- id: FR-019-AC-22
  path: spec/model/functional/FR-019-rust-library-interface.md
  role: context_only
  excerpt: A default-feature external consumer calls the existing `CheckedPackageV2::temporal_bindings()`
    read on a reader-admitted package with a temporal clause and two compensation
    nodes, then borrows three `TemporalBinding` rows in ascending structural `NodeRef`
    order.
- id: FR-019-AC-25
  path: spec/model/functional/FR-019-rust-library-interface.md
  role: examined
  excerpt: A default-feature consumer borrows `Option<&[TemporalClauseName]>` through
    `CheckedPackageV2::temporal_clause_names()` after strict admission and observes
    two separate named occurrences of one shared clause node; it passes those same
    rows to the encoder and strictly rereads equal rows.
- id: TC-058
  path: spec/model/matrix/TC-058-model-crate-public-interface.md
  role: context_only
  excerpt: public items by name with no glob re-export, its default-feature public items
- id: conformance-qspec temporal_binding_public component
  path: Makefile
  role: examined
  excerpt: QSPEC_CONFORMANCE=1 $(CARGO) test $(LOCKED) -p quire-contract-model --test
    temporal_binding_public default_feature_consumer_borrows_admitted_clock_rows --
    --exact
bindings:
- test_id: default_feature_consumer_borrows_admitted_clock_rows
  ac_id: FR-019-AC-22
  trace: correct
- test_id: default_feature_consumer_borrows_admitted_clock_rows
  ac_id: FR-019-AC-25
  trace: correct
```
