---
id: SR-600007
title: "IR-874 PR 587 gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; PR #587 f14f62f402adf78a385914589a27079b469085d5; crates/quire-contract-model/src/checked_package/shared.rs, crates/quire-contract-model/src/checked_package/v2/protocol_body.rs, tests/it/checked_package_v2_protocol_body.rs"
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
| FND-001 | fixed | At db4fa435, five traced public-reader tests exercise `origin`/`each` terminal scope, role lifetime, FIFO key, repeat body binder escape, and retry/recovery capture and over refusals. Each checks admitted and adverse reader outcomes. Exact-head hosted execution is pending. |

## Disposition verdict

**PASS for the recorded coverage finding at db4fa435** — the specific stage paths named in FND-001 now have public-reader tests. AC-291/292 remain Planned until their full criteria are integrated.
