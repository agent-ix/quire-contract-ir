---
id: SR-558004
title: "IR-1014 PR 562 code and Rust review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir@8b7a22cbed7355880050d6c5705dbfddc1f408a1; PR #562 Rust reader, public refusal, writer vectors, and call sites"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-059
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/FR-060
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/FR-062
    type: reviews
---

# SR-558004: IR-1014 PR 562 code and Rust review

## Summary

Ticket: IR-1014. The review covers the complete 15-file PR diff at 8b7a22c, with rust-review folded into code-review. QSpec FR-475, FR-476, TC-960 and TC-961 at origin/main d818d8e were used as the wire authority. The repo's AGENTS.md, CLAUDE.md, root manifest, lint and format configuration were read. No CI gate, dependency, schema copy, or QSL source changed. The new closed forms, typed duplicate payload, digest-ordered node traversal, and public read/write path otherwise match the specified shape. The author's focused log reports 10 new tests and one model-member test passed; `git diff --check` passed independently. Final aggregate gates belong to the code lead after fixes.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Terminal self-type derivation scans every selected population member for every terminal before duplicate rejection, without charging the reader work meter. An untrusted package with many individually valid terminal nodes and a large selected population document can perform T times P object lookups under a low work limit and return a semantic refusal instead of the required work-incomplete outcome. Cache the selected model's unique self type or meter each traversal before it runs; assert the work limit boundary. | crates/quire-contract-model/src/checked_package/v2/claim_terminal.rs:99 |

## Coverage

Examined FR-059-AC-1 through AC-5, FR-060-AC-1 through AC-5, and FR-062-AC-1 through AC-4 against the changed implementation and tests. FR-038's finite, fallible work requirement was context for FND-001. Public enum variants and inline typed refusal are deliberate API choices; the `large_enum_variant` allowances explain those choices and are not findings. No vendored QSpec material was added. Plan completion: not assessed.

## Verdict

CONDITIONAL. Correct the unmetered terminal self-type walk before the final PR gate.

## Dispositions

Fix round 1, PR head `9303fed058aaa38008d8d43f687d7bc72ae1b97e`:

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | `self_object_key` now charges one work unit for each population and member before inspecting it, propagates `ValidationFailure`, and `terminal_self_population_scan_obeys_work_limit` observes `Work` incomplete at the terminal `parameters` binding. After excerpt: `for members in owner.package.population_members.values() { meter.charge(1, || binding_path(position, 2))?; for object in members { meter.charge(1, || binding_path(position, 2))?;` |

The code-review finding is resolved at this head. Final aggregate gates remain with the lead.
