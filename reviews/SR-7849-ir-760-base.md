---
id: SR-7849
title: "Base requirements review — IR-760 selected population specification"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; spec/model/functional/FR-019-rust-library-interface.md, spec/model/functional/FR-053-selected-population-binding.md, spec/model/matrix/TC-453-selected-population-binding.md, spec/spec.md"
review_set: subset
---

## Summary

The new requirement leaves the selected finite maximum vulnerable to a retargeted admitted parameter type, omits the requested typed unknown-closure outcome, and has no planned TestMatrix entry.

## Verdict

**FAIL** — The new contract has unresolved correctness and tracking gaps.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The selected source maximum is not bound to the selected occurrence across an admitted V2 package | spec/model/functional/FR-053-selected-population-binding.md:102 |
| FND-002 | medium | Unknown closure has no typed result despite the ticket acceptance | spec/model/functional/FR-053-selected-population-binding.md:105 |
| FND-003 | medium | The new criteria have no planned TestMatrix entry | spec/model/functional/FR-053-selected-population-binding.md:102 |
