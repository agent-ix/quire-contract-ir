---
id: SR-1558
title: "Failure-domain review, disposition round 2, of quire-contract-ir PR #295 (IR-627 fixtures carry derived keys)"
type: SpecReview
analysis: failure-domain
scope: "agent-ix/quire-contract-ir@3c2fe37b2444da05b109c83d4274da0005f04b9a; spec/checked_package/functional/FR-038-consume-checked-package-v2.md ('Fixtures carry derived keys', IR-627-Q5, FR-038-AC-133..135), spec/checked_package/matrix/TC-226-checked-package-v2-anonymous-structural-node-bodies.md"
review_set: subset
---

## Summary

Ticket: IR-627. These are new findings from disposition round 2, measured against QSpec origin/main
2b2dd28 with an independent JCS hash. Confirmed:
- **Placeholder keys exist** in positive-all-families, positive-clause-operations, positive-union-nodes,
  positive-control-operations, positive-operation-identities and node-identity-vectors.
- **IR reads exactly the three positive fixtures plus `adverse.json`** for AC-107 and AC-112.
- **QSL-635 exists** (Backlog).
- **The grammars match the reader** (structural.rs is_non_negative_integer), with the new signed
  grammar correctly distinguished from is_nonzero_integer.
- **AC-133/134 placeholder rows are reachable.** No node-order check exists, and the projection check
  and reference resolution run after the new stage (mod.rs:1760-1842).

The "measured" fixture inventory still misclassifies declared nodes as derived shapes. It also leaves
out one QSpec file IR reads, and it miscounts the adverse entries.

## Verdict

Not mergeable: one medium and one low new finding.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The measured fixture inventory is not accurate. (a) In `positive-all-families.json`, node 0 (`scalar_type`/`boolean`, `aaaa`, declaration `Example::Flag`) and node 2 (`integer_range`, `cccc`, `Example::Small`) carry a `declaration`. They are gated, and item 1 of the decided rule does not take them. So "node 0 is the derived boolean scalar" is wrong, and AC-133's row placing `aaaa...` on the boolean node cannot produce `stale-node-key` on that fixture; only the clause and control fixtures hold an undeclared `aaaa` boolean. (b) IR also reads `dependency-selection-vectors.json` (AC-113), whose base is `positive-all-families.json` and whose recorded `package_id` changes when QSL-635 regenerates that base. Neither Q5's read/not-read list nor AC-133 names it. (c) AC-135 says "the four `malformed_wire` body-grammar entries", but `adverse.json` has five. Its "measured over the current list" pointer list also omits `negative-temporal-interval-bound` (`/semantic_graph/nodes/29/.../interval/lower`) | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1892-1928, :1948-1964, AC-133, AC-135 |
| FND-002 | low | AC-134 and fixture item 2 understate the in-repo migration. The in-repo `aaaa` `scalar_type`/`boolean` has a `literal` body, not `aggregate{[]}`, and the `bbbb` `option` is typed at `aaaa` with an empty aggregate body (tests/it/support/checked_package.rs:1306-1322). Items 3 and 5 refuse these whatever their key, so regenerating keys alone does not make the fixtures admit: their bodies and `semantic_type` change too | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1910-1911, AC-134 |

## Dispositions

Round 3, reviewed at 60500e7bbb4318b93d0f6ea51e3985c31a7983ea, re-measured against QSpec origin/main 2b2dd28 fetched fresh.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 60500e7bbb4318b93d0f6ea51e3985c31a7983ea |
| FND-002 | fixed | 60500e7bbb4318b93d0f6ea51e3985c31a7983ea |
