---
id: SR-11822
title: "spec-review/ears-conformance review of IR-817 PR #418"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-contract-ir@f07017c25c744c66750f0e86ff0bb8b20bb9707d; PR #418; spec/conformance/functional/FR-020-json-conformance-interface.md; spec/conformance/matrix/tests.md; spec/core/functional/FR-012-anchors-clauses-dependencies.md; spec/core/matrix/tests.md; spec/model/functional/FR-023-executable-projection-binding.md; spec/tests.md; tests/it/conformance.rs"
review_set: subset
---

## Summary

Ticket: IR-817. Checked FR-020-AC-3 through AC-12 for singular, direct-assertion phrasing. Eight of ten are singular and testable as written; the split granularity is inconsistent and two rows have attachment ambiguity.

## Verdict

**PASS with low findings.** No criterion is untestable; each low finding is a wording fix.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | AC-5 (and AC-6 by mirror) lists four member groups then "above 9007199254740992"; the bound grammatically attaches only to the last item (coverage trace target revision) | spec/conformance/functional/FR-020-json-conformance-interface.md:138 |
| FND-002 | low | AC-4 keeps accept-at-2^53 and reject-at-2^53+1 as one row (as does AC-3 for accept/reject spellings) while AC-5/AC-6 split refuse/pass into two; split granularity is inconsistent | spec/conformance/functional/FR-020-json-conformance-interface.md:137 |
| FND-003 | low | AC-7's subject is the decoder fixture ("a decoder fixture ... returns"); a fixture does not return, the runner's actual result for that fixture does | spec/conformance/functional/FR-020-json-conformance-interface.md:140 |
