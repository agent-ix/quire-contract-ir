---
id: SR-611031
title: "IR-1069 terminal condition gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/v2/protocol_body.rs, tests/it/checked_package_v2_protocol_body.rs"
review_set: subset
---

## Summary

Ticket: IR-1069. Compared FR-038-AC-326 and QSpec FR-464 terminal stage and TC-947 TM10–TM13 against the changed reader and tagged TC-048 tests.

## Verdict

**PASS** — no findings in the scoped PR diff.

## Examined units

- `FR-038-AC-326` (examined; `spec/checked_package/functional/FR-038-consume-checked-package-v2.md`): For a protocol activated `on each`, a Boolean `terminal when` condition that reads the protocol's `over` model observation admits, as QSpec FR-464's `terminal` stage and TC-947 TM11 require. The same read admits under `on origin`. Under `on each`, a condition that instead reads a protocol capture or the activation binder refuses `missing_declaration`/`missing-name` at the terminal node with stage `terminal`; under `on origin`, a condition reading an initialized protocol capture admits. The reader applies the stage table to each read independently: allowing `over` under `each` does not make its captures or activation binder readable.
- `TC-048` (examined; `tests/it/checked_package_v2_protocol_body.rs`): public-reader acceptance and refusal tests for the terminal stage.
- `QSpec FR-464` terminal stage (context_only): `over`, and protocol captures under origin; under each no capture and no binder.
- `QSpec TC-947` TM10–TM13 (context_only): published admission and refusal oracles.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Plan completion: not assessed. The two changed tests trace FR-038-AC-326 and exercise admission for origin and each, origin capture admission, and each capture and activation-binder refusals with exact code, cause, stage, path, and locus.
