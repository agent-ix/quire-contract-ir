---
id: SR-2477
title: "Gap analysis of quire-contract-ir PR 947 against FR-358-AC-10"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; PR 947 head 2f73ecc6ca9c1be4de6582d54f083958e5c5c924 diff against origin/main c838196ce; spec/checked_package/functional/FR-358-compose-native-identity-join-budget.md (statement, FR-358-AC-10); spec/checked_package/matrix/TC-471-monitor-authority-receipt-and-layout-context.md step 7; tests/it/native_join_budget.rs"
review_set: subset
---

## Summary

Ticket: IR-1406. PR: quire-contract-ir#947. Planless, scoped to the PR's
criterion FR-358-AC-10. Plan completion: not assessed. Out of scope per the
brief: FR-361 authority row/receipt and FR-357-AC-16 (IR-1384).

FR-358-AC-10 clauses against the one new test,
`external_loci_report_their_exact_source_at_the_allowance_edge`:

- Each of `AdapterRow(i)`, `AuthorityRow`, `PreparationOutput(k)`,
  `PreparationSelection(k)` admits at an allowance equal to cumulative
  consumption including that charge: covered (2 + 5 at allowance 7,
  consumption 7, no denial). `MonitorAuthority` is also exercised.
- One unit below, the first unpaid locus is exactly that source and form, with
  kind and attempted amount, without allocating: covered (allowance 6, second
  charge refused with `Work`/5, `first_unpaid_locus() == Some(locus)`,
  consumption stays 2).
- "admitted charges with these sources are cumulative with the following
  native read": not covered. The test makes two external charges and no
  native read (FND-001). TC-471 step 7 asks for exactly this: "admitted ones
  followed by a native read".

Production code: the five variants have an owning requirement (FR-358
statement). No orphan code, no stub.

## Verdict

**FAIL (one medium gap).** One clause of FR-358-AC-10 has no test.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-358-AC-10's clause "admitted charges with these sources are cumulative with the following native read" is untested: the test only composes two public `accounted_charge` calls and never runs a native read (for example `SelectedExactIdentities::from_stage_rows`) after the external charges. TC-471 step 7 requires one. A regression where a native read resets or ignores external charges made with the new sources would pass this test | tests/it/native_join_budget.rs:246-306 |
| FND-002 | low | The test is tagged `#[trace("FR-358-AC-10")]` only; TC-471 says to tag its tests `#[trace("TC-471", "<criterion id>")]`, so the test is not attached to its TC | tests/it/native_join_budget.rs:248 |
