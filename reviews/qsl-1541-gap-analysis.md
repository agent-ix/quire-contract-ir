---
id: SR-958002
title: "gap-analysis of QSL-1541/QSL-1542 guarantee and composition reader"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/checked_package/matrix/TC-474-implementation-guarantee-and-component-composition-bodies.md, tests/it/checked_package_v2_guarantee_composition.rs"
review_set: subset
---

## Summary

Manual AC-to-test gap check (Quoin gap-analysis not run as a full matrix pass; diff-scoped) of PR #958 at e2553549. Ticket QSL-1541, with QSL-1542. Plan completion: not assessed.

## Examined scope

- FR-038-AC-531..538 (examined), each bound to one TC-first `#[trace("TC-474", ...)]` test.
- QSpec FR-491 static checks 2 to 5 and its reader paragraph (examined).

## Verdict

FAIL on coverage of refusal paths and on an undeferred QSpec gap. Trace tags are TC-first and each AC has one test; admission cases are covered.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | QSpec FR-491 static checks 4 and 5, and check 2's clause-of-component lookup, are not made; FR-038 and TC-474 disclose this but no follow-up ticket is named | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:5776-5786 |
| FND-002 | medium | Untested refusals: out-of-order supplies, out-of-order composition environment, a malformed node key (malformed_wire), and a cross-package composition target; FR-038-AC-535 omits supplies although FR-038 step 3 orders it | tests/it/checked_package_v2_guarantee_composition.rs:496-566 |
| FND-003 | low | FR-038 prose says the reader joins composition subject and clause targets of component packages to dependencies; the graph stage refuses them (SR-958001 FND-002), so the text claims behaviour the code lacks | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:5776-5781 |
