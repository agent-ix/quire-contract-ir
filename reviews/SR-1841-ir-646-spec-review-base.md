---
id: SR-1841
title: base review of IR-646 owner wire and lock joins
type: SpecReview
analysis: base
scope: agent-ix/quire-contract-ir@46597863d12f708bbcdf146609deb2b3386469c7; spec/checked_package/functional/FR-038-consume-checked-package-v2.md,
  spec/checked_package/matrix/TC-228-checked-package-v2-wire-owner-and-lock-joins.md,
  spec/checked_package/matrix/tests.md
review_set: subset
---

## Summary

Ticket: IR-646. Reviewed PR #302 at 46597863d12f708bbcdf146609deb2b3386469c7; examined FR-038-AC-153 through AC-155, TC-228 and the matrix entry.

## Verdict

**FAIL** — The owner-join rule omits a required model subtype check; all other changed owner-presence and refusal rules match the cited upstream contracts.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The ModelOwner join calls both model/object_type and model/systems_interface a matching object type without testing interfaceFeatures. QSpec FR-322 requires an object type without interfaceFeatures for object_type and one with interfaceFeatures for systems_interface. The current rule can admit a node under the wrong semantic form; require the distinction and test both cross-kind mutations in TC-228. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1801 |
