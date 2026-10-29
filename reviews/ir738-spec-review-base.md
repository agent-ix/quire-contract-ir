---
id: SR-7102
title: IR-738 spec-review-base of candidate application identity
type: SpecReview
analysis: base
scope: agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/v2/application_identity.rs,
  crates/quire-contract-model/src/checked_package/v2/encode.rs, crates/quire-contract-model/src/checked_package/v2/mod.rs,
  crates/quire-contract-model/src/checked_package/v2/operations.rs, crates/quire-contract-model/src/lib.rs,
  spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md, spec/model/functional/FR-019-rust-library-interface.md,
  spec/model/functional/FR-046-derive-candidate-application-node-identity.md, tests/it/application_node_identity.rs,
  tests/it/main.rs, spec/model/matrix/TC-058-model-crate-public-interface.md
review_set: subset
---

## Summary

Ticket: IR-738. Base spec review scopes the implementation status, public inventory and AC6 evidence wording changed by the PR, with their impacted procedure/resource text.

## Verdict

**CONDITIONAL** — two low documentation defects require correction.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | TC-058 still labels the candidate API planned/unrun and locates a planned export paragraph removed by this PR; readers following the procedure encounter a missing locator and contradictory implementation status. | spec/model/matrix/TC-058-model-crate-public-interface.md:50 |
| FND-002 | low | FR-046 still calls the present recursive body clone an implementation gap although this PR replaces it with iterative copying and nonrecursive cleanup; the implemented-status requirement now describes a closed gap as current. | spec/model/functional/FR-046-derive-candidate-application-node-identity.md:153 |

## Coverage

Base spec review scopes the implementation status, public inventory and AC6 evidence wording changed by the PR, with their impacted procedure/resource text. Requirement field rules remain authoritative in FR038; no duplicated preimage grammar introduced. No new relationships/domain objects/architecture, EARS condition or structural artifact changes justify additional lenses. Findings concern stale status integration after implementation. Preserve currentmain FR038AC224 aliaswork planned/unrun; it is disjoint and no implementation credit is claimed.

Review-only: no source edits, builds, tests, lock acquisition, push or merge. The reviewed PR tree was clean before and after review. Local main was stale; the PR delta against its merge base covered ten owned paths. No applicable AssuranceProfile was found. The installed catalog template and schema were read. The configured model ID was unavailable; the host identified a GPT-6 based Codex session. Runtime receipts were inspected, not independently executed as reviewer tests. The full `make ci` run exited 2 at strict coverage: the baseline had 48 unbacked rows and the candidate had 42, with no new unbacked rows or status contradictions. The recorded run passed 459 integration tests, 197 model tests, 18 model doctests, 7 root doctests, formatting, both Clippy lanes, the corpus, 632-document validation and the separately run policy and unsafe checks. Kani and replay were not rerun; this review claimed neither a green aggregate gate nor merge authorization.

## Examined units

```yaml
scope:
- id: FR-046-AC-1
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: An external model-crate consumer derives a candidate from a typed draft
    before any package read, obtains its candidate key and matches typed errors; no
    helper output is an admission verdict or admitted-package constructor. The named
    public items are exported only by the model crate.
- id: FR-046-AC-2
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: An ordinary application produces the independently constructed canonical
    application-preimage digest. Changing node tag, semantic form, semantic type,
    declaration presence/content or body independently changes the bound digest. Changing
    only its provisional selector does not change an ungrouped candidate.
- id: FR-046-AC-3
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: A group containing applications and nonapplication nodes uses every group
    node in graph order. Self and internal references through application arguments,
    aggregate members and binding values produce the expected ordinals; provisional-to-final
    key substitution preserves the candidate. Outside-group and dependency references
    remain unchanged, and changing an outside target changes the candidate.
- id: FR-046-AC-4
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: 'Missing grouped context, absent target, repeated graph key, mismatching
    target fields or context from another draft return their exact typed variants/payloads
    and no hash candidate. Repetition order is deterministic; nonapplication input
    refuses. Inspection confirms borrowed immutable constant-size context, checked
    counters and the stated scan bounds, without a new cap or hidden graph copy. On
    a 256 KiB thread stack, grouped and ungrouped in-memory application drafts with
    100000 nested JSON levels complete structural equality and hashing under a sufficient
    byte limit, refuse CanonicalEncoding '
- id: FR-046-AC-5
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: For an independently encoded preimage of C canonical bytes, byte limits
    C and C+1 produce the same candidate; C-1 and zero refuse CanonicalEncoding with
    the canonical byte-limit error. A canonical nonencodable-number control retains
    its typed encoder error. The byte ceiling is not asserted to bound graph/context/body-copy
    memory or work.
- id: FR-046-AC-6
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: 'After correct final-key/reference substitution, reader-admissible ordinary
    and grouped controls read successfully with the helper candidate. Changing the
    body while retaining that key produces the reader''s stale-node-key outcome and
    distinct expected key. Inspection finds one authoritative preimage/encoder and
    unchanged reader charge-before-hash ordering; an exhausted existing work charge
    wins before an otherwise failing application hash. Public context validation adds
    no reader refusal or work charge. The implementation and focused checks have passed;
    full gates and independent review remain '
- id: FR-019-public-application-items
  path: spec/model/functional/FR-019-rust-library-interface.md
  role: examined
  excerpt: '| `checked_package` (V2 candidate application identity) | function `derive_application_node_id`;
    types `ApplicationNodeDraft`, `ApplicationIdentityContext`, `CandidateApplicationNodeId`,
    `ApplicationIdentityError` | FR-046 |'
- id: TC-048-application-identity
  path: spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md
  role: examined
  excerpt: 'Implemented (IR-738). Six public application-identity controls, five focused

    reader/preimage regression tests and the private-construction compile-fail

    doctest have passed. Full gates and independent review remain pending.'
- id: TC-058
  path: spec/model/matrix/TC-058-model-crate-public-interface.md
  role: examined
  excerpt: 'PLANNED/UNRUN (IR-738). Verify

    [FR-046](../functional/FR-046-derive-candidate-application-node-identity.md),

    AC-1, and the planned API paragraph before Public items in

    [FR-019](../functional/FR-019-rust-library-interface.md).'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: 'The present recursive body clone is an implementation gap

    to close before this pre-read surface ships, not a stack-safety exemption.'
bindings:
- test_id: ordinary_candidates_bind_the_preimage_fields_before_any_read
  ac_id: FR-046-AC-1
  trace: correct
- test_id: draft_context_refusals_have_exact_payloads_and_deterministic_precedence
  ac_id: FR-046-AC-1
  trace: correct
- test_id: ordinary_candidates_bind_the_preimage_fields_before_any_read
  ac_id: FR-046-AC-2
  trace: correct
- test_id: grouped_candidates_use_all_members_in_order_and_preserve_external_references
  ac_id: FR-046-AC-3
  trace: correct
- test_id: draft_context_refusals_have_exact_payloads_and_deterministic_precedence
  ac_id: FR-046-AC-4
  trace: correct
- test_id: deep_borrowed_drafts_copy_compare_hash_and_refuse_on_a_small_stack
  ac_id: FR-046-AC-4
  trace: correct
- test_id: canonical_byte_boundaries_and_number_refusals_preserve_encoder_errors
  ac_id: FR-046-AC-5
  trace: correct
- test_id: tests::application_identity_work_refusal_precedes_a_canonical_hash_refusal
  ac_id: FR-046-AC-6
  trace: correct
- test_id: caller_substitution_admits_and_a_retained_candidate_refuses_a_changed_body
  ac_id: FR-046-AC-6
  trace: correct
```

## Dispositions

Round 1; reviewed. Both original findings fixed in the actual documentation. Original findings and round-zero verdict remain unchanged.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Fixed in the reviewed PR. |
| FND-002 | fixed | Fixed in the reviewed PR. |

## Disposition round 1 verdict

**PASS** — both findings fixed; no new regression in the documentation fix or integrated delta.

Actual parent chain: fix follows merge with parents original reviewed and main. Production, tests, Cargo, Make, scripts and CI are unchanged versus the reviewed source. Integrated IR-490 documentation still labels FR-038-AC-224 PLANNED/UNRUN and receives no implementation credit. Current TC-058 locator resolves the five-export FR-019 Public items row; source inventory and compiled consumer evidence remain separate. Resource text now names the implemented iterative copy and nonrecursive cleanup.

This is disposition-only source inspection, no fresh review, runtime/build/test or locks. Exact HEAD and clean status verified. The pending gate/review wording is honest during this round; for durable merged documentation, recommend removing only temporal pending clauses and retaining focused-check evidence while tracker/review artifacts carry qualification. No new finding for this recommendation. Original candidate make ci strict-coverage failure and separately passing tail remain the only measured full-gate evidence; final changed-head full gate is pending. No aggregate-green or merge authorization claim.

```yaml
dispositions:
- fnd: FND-001
  outcome: fixed
  after_excerpt: 'Implemented (IR-738). The compiled public consumer controls and
    the three

    source/spec inventory controls have passed; full gates and independent review

    remain pending. Verify

    [FR-046](../functional/FR-046-derive-candidate-application-node-identity.md),

    AC-1, and the candidate application identity row under Public items in

    [FR-019](../functional/FR-019-rust-library-interface.md).'
- fnd: FND-002
  outcome: fixed
  after_excerpt: 'The implementation uses the iterative body copy and

    nonrecursive cleanup described above.'
```
