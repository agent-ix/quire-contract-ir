---
id: SR-600007
title: "IR-874 PR 587 gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; PR #587; crates/quire-contract-model/src/checked_package/shared.rs, crates/quire-contract-model/src/checked_package/v2/protocol_body.rs, tests/it/checked_package_v2_protocol_body.rs"
review_set: subset
---

## Summary

The new public-reader tests cover over, forward capture, sequence, choice, await, one parallel join, finish and one compensation guard. Several changed stage paths have no direct positive and adverse public-reader evidence.

## Verdict

**CONDITIONAL** — the new implementation lacks focused evidence for terminal activation scope and representative changed role, channel, repeat, retry and recovery paths.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The nine new public-reader tests do not exercise terminal `origin` versus `each` scope, role lifetime or FIFO key, repeat body binder escape, or retry/recovery stage refusals, although this PR implements each path. | tests/it/checked_package_v2_protocol_body.rs:28 |

## Scope

- **FR-038-AC-291 (examined)**: Binder levels and stage tables, including wrong level, duplicate name, forward/foreign/stage reads, branch and await flow, and binders-only model-state precedence.
- **FR-038-AC-292 (context only)**: Decision facts and binder-unavailable precedence; decision-fact checks are separately owned and remain Planned.
- **QSpec FR-464-AC-8 (examined)**: Clause, role, channel, run, finish, terminal and compensation stage-readable sets.
- **QSpec FR-466-AC-10 (examined)**: Sequence, choice, parallel, repeat and await In/Out binder flow.
- **QSpec FR-466-AC-12 (examined)**: Compensation registration, trigger, activation, retry and recovery scopes.

## Coverage

Plan completion: not assessed. `quire matrix --scope . --format json` reports 12 binders for FR-038-AC-291, including the nine new traced tests, and zero for FR-038-AC-292, which remains Planned for the decision-fact slice. Trace tags alone do not cover each stage. The changed source has an owning requirement and no stub. The nine new tests are meaningful for the cases they assert; the stage cases listed above remain untested. Hosted `make lint` failed before tests at the frozen head, so none of the tests has passing exact-head execution evidence yet. No full repository semantic audit or plan-assisted check was requested.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | In the reviewed PR, five traced public-reader tests exercise `origin`/`each` terminal scope, role lifetime, FIFO key, repeat body binder escape, and retry/recovery capture and over refusals. Each checks admitted and adverse reader outcomes. Exact-head hosted execution is pending. |

## Disposition verdict

**PASS for the recorded coverage finding ** — the specific stage paths named in FND-001 now have public-reader tests. AC-291/292 remain Planned until their full criteria are integrated.

## Verification after hosted regression fix

Hosted Rust Checks passed compilation but 13 tests failed, so the prior static disposition had no passing execution evidence., old positive fixtures for `each`, capture, compensation, receive and related ends now use the QSpec binder levels; `terminal` under `each` relevels `finish` before asserting terminal's own refusal. The work-boundary test retains independent literal constants: 37 Tree charges, 448 other pre-protocol charges, five membership charges, seven causal/progress charges and 52 new binder-scope charges, giving admitted M=549, dependency 541/542 and whole-read 548/549. The result-context fixture expects its measured 387 work charges. These are static checks of the exact test diff; hosted Rust Checks are pending. No acceptance criterion has been marked complete by this review.

## Verification after second hosted regression fix

The hosted run contradicted the prior Tree dependency-boundary expectation: the old overcounted threshold reached `/semantic_graph/nodes/40/body/members/0/value/type`, rather than the authored Tree dependency pointer., the constant ledger distinguishes 453 other charges before Tree's 37-charge dependency (489/490), seven causal/progress charges, and 52 scope charges; admitted M remains 549 and the whole-read M-1 pointer remains `/semantic_graph/nodes/38/body/members/0/value/target`. The corrected related-record fixture introduces the peer binder in a preceding event, then admits two distinct JCS-ordered records that read the peer and current send binders. The five traced stage tests remain. The exact-head hosted run is pending; AC-291/292 remain Planned.

## Verification after measured Tree boundary

The hosted diagnostic run's printed charges directly establish dependency `/semantic_graph/nodes/38/dependencies/0` at limit 536, consumed 537, and final `/semantic_graph/nodes/38/body/members/0/value/target` at limit 543, consumed 544., TC-226 asserts those two one-below boundaries, M=544 admission, and exact pointers; it no longer prints diagnostics. The new 47-charge scope increment is documented as the admitted-result fixture's 36 plus 11 additional recursive-fixture graph nodes. The related-record and sibling-branch positive cases passed in the diagnostic hosted run. The five traced stage tests remain. Exact-head hosted Rust Checks are pending; AC-291/292 remain Planned.

## Final exact-head verification

Hosted Rust Checks passed at authored code, including TC-226 and the public-reader stage cases. Supply-chain Checks, guard, CLA and language-boundary checks also passed. The original coverage FND-001 remains fixed with five added tests, and distinct related-record admission is retained. **Final disposition: PASS** for this PR diff. FR-038-AC-291/292 remain Planned until the remaining decision-fact and integration work is complete.
