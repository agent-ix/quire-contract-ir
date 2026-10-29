---
id: SR-7100
title: IR-738 code-review of candidate application identity
type: SpecReview
analysis: code-review
scope: agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/v2/application_identity.rs,
  crates/quire-contract-model/src/checked_package/v2/encode.rs, crates/quire-contract-model/src/checked_package/v2/mod.rs,
  crates/quire-contract-model/src/checked_package/v2/operations.rs, crates/quire-contract-model/src/lib.rs,
  spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md, spec/model/functional/FR-019-rust-library-interface.md,
  spec/model/functional/FR-046-derive-candidate-application-node-identity.md, tests/it/application_node_identity.rs,
  tests/it/main.rs, spec/model/matrix/TC-058-model-crate-public-interface.md
review_set: subset
---

## Summary

Ticket: IR-738. Rust formal pre-handoff checklist reapplied over the exact diff.

## Verdict

**PASS** — no defects in the examined changed scope.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Rust formal pre-handoff checklist reapplied over the exact diff. No vendored files, compatibility surface, duplicate preimage, warning suppression, new gate weakening, unsafe or runtime recursion introduced. Borrowed constant retained context; deterministic N(N-1)/2 uniqueness scans and checked u64 conversions; explicit O(D) equality/copy frames; owned partial containers and rewritten references have nonrecursive drop. No claim byte limit bounds graph/copy work. Reader still constructs all-group graph-order keys, charges before shared application_digest and translates errors as before. Candidate has private field, immutable accessor and no admitted-package conversion; exports are model-only. Independent JSON preimage oracle tests exact content; group literal ordinals test nonapplication/self/binding/aggregate and external/dependency retention; canonical C/C+1,C-1/zero and typed number errors are discriminating. Deep100000levels tests compare equal and differing deepest leaf on256KiB stack with iterative fixture teardown and early refusal. Ordinary/grouped actual reader admission plus retained-key stale negatives are real controls. Stable rustdoc compile_fail establishes compilation failure, not E0451 verification.

Review-only: no source edits, builds, tests, lock acquisition, push or merge. Exact frozen HEAD and clean status verified before/after. Local main is stale; explicit...HEAD has ten owned paths and merge-bas. No applicable AssuranceProfile found. Installed catalog template/schema read. Exact configured model ID unavailable; host identifies GPT-6 based Codex. Runtime receipts are evidence inspected, not independently executed reviewer tests. Full make ci exit2 at strictcoverage; baseline48unbacked to candidate42, zero new unbacked and zero status contradictions. Native459integration/197model/18modeldoc/7rootdoc, fmt, both clippy lanes, corpus,632document validation and separately run policy/unsafe tail pass. Kani/replay not rerun; no aggregate-green or merge authorization claim.

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
