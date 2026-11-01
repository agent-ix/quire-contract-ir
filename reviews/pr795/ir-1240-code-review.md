---
id: SR-202611010113
title: "IR-1240 PR 795 structural member mapper code review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; PR #795; crates/quire-contract-model/src/checked_package/v2/typed_terms.rs; tests/it/checked_package_v2_node_map.rs"
review_set: subset
---

## Summary

Reviewed frozen head `51b539b1015a6a95056472ff7968cd24157d47c2` against merged FR-048 AC-6 and AC-7. Rust review covered the iterative decoder, all named member slots, raw and typed mirrors, callback order, malformed input, and the external admitted-operation test. No applicable AssuranceProfile was found under `spec/`.

## Verdict

**PASS** — no code defect found in the PR diff. The iterative task stack reconstructs nested node IDs without recursive descent; invalid shapes return `None` without indexing panic. Mapping invokes the callback once per raw member slot, writes only `Some` replacements, and synchronizes retained typed views from that result. The external test would fail on the old flat-only mapper. The external census coverage limitation is recorded in the separate gap analysis.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

- FR-048-AC-6, examined: whole nested structural ID, `None` and `Some` semantics, no cascade, and public external consumer.
- FR-048-AC-7, examined: five declaration variants, relationship-end order and optional population, six fairness target variants, callback count and raw/typed mirror.
- `typed_terms.rs`, examined: structural decoder, operation mapper and changed focused tests.
- `checked_package_v2_node_map.rs`, examined: external admitted operation test.
- FR-048-AC-8, context_only: mapper resource behavior; no 100000-node gate run in this reviewer pass.

## Evidence

The PR has no CI or spec diff, new fixture copy, new public export, or compatibility layer. `structural_member_id` uses a work stack and checked subtraction before `split_off`; each nested node is constructed after its child segments. `map_operation_ids` preserves raw bytes for `None` and uses iterative `node_id_value` for `Some`. Existing and changed tests assert stable relationship-end slot order, fairness target variants, five declaration variants, and a second mapping pass over an externally admitted operation. The coder reported focused tests, scoped Clippy and rustfmt passing; this independent review did not run the aggregate gate, which belongs to the lead on the final reviewed head.
