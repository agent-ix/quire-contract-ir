---
id: SR-1843
title: failure-domain review of IR-646 owner wire and lock joins
type: SpecReview
analysis: failure-domain
scope: agent-ix/quire-contract-ir@46597863d12f708bbcdf146609deb2b3386469c7; spec/checked_package/functional/FR-038-consume-checked-package-v2.md,
  spec/checked_package/matrix/TC-228-checked-package-v2-wire-owner-and-lock-joins.md,
  spec/checked_package/matrix/tests.md
review_set: subset
---

## Summary

Ticket: IR-646. Reviewed PR #302 at 46597863d12f708bbcdf146609deb2b3386469c7; examined FR-038-AC-153 through AC-155, TC-228 and the matrix entry.

## Verdict

**PASS** — The changed rules address absent owners, wrong owner variants, source-map drift, unreachable model nodes, ordering, and work accounting. The subtype defect is recorded in the base review.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
