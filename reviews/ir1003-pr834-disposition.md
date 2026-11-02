---
id: SR-2026110283403
title: "IR-1003 PR 834 review disposition"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; PR 834; code/Rust and scoped gap disposition; FR-352-AC-7 only"
review_set: subset
relationships:
  - type: references
    target: ix://agent-ix/quire-contract-ir/FR-352
---

## Summary

Re-examined the new supported-cycle test and renamed unsupported-control test, their public production paths, and supplied native receipts. This disposition covers FND-001 in both SR-2026110283401 (code/Rust) and SR-2026110283402 (scoped gap); original artifacts and findings remain unchanged. Original Linear review comments: d4761c2d and 260c7a19.

## Verdict

**PASS** for the scoped code/Rust and gap disposition. Both original FND-001 findings are fixed. No new scoped defect was found. The supported representation completes a recursive nominal type using a witnessed Composite handle and a complete one-field record view, rather than transitively unfolding the target record. AC7 requires bounded completion or typed exhaustion; it does not require recursive unfolding of nominal handles.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Dispositions

- SR-2026110283401 / FND-001: **fixed**. The new independently authored selected-model fixture strictly admits 128 records forming a cycle. Each of the 128 public reads examines a real supported record member target, returns the exact complete expected Composite and record-field values at work 132, and returns ExhaustedWork with charged work 131 and the next record's structural locus and record_field_target form at limit 131. Every member, including the closing edge to record zero, is checked on a 256 KiB stack. Repeat and clone equality cover values and exhaustion outcomes; removing the next record's witness yields the named typed refusal at that target. The wire, source document, byte buffer and read evidence are dropped before projection. The old 4096-option graph remains an unsupported/admission control and receives no claim of 4096-level projection traversal.
- SR-2026110283402 / FND-001: **fixed**. The additional supported nominal-cycle test supplies the formerly missing inner-target work evidence. The omitted-inner-charge mutation fails with exit 101 (observed work 131 versus expected 132), and restored production passes both focused tests. This makes the work oracle sensitive to the production charge rather than just a shallow unsupported root. The computed trace binding is supported by the inspected behavior; no separate credit is transferred to AC6.

## Coverage

- FR-352-AC-7 (role: examined; excerpt): Repeated reads of one admitted package and its clone return equal complete values or equal typed outcomes. A nested or cyclic type either completes within the finite limit or reports typed exhaustion with the charged work and locus; no call reparses wire/source, grows the call stack with type depth, or grants admission to an unadmitted fixture.
- Source inspection: project_exact_type charges requested and semantic-type nodes, iterates the finite witness rows and selected fields, charges each nominal record-field target, and calls project_record_field_type. That helper directly matches MemberType::Record, validates its admitted target kind and identity, and returns Composite(key); MemberType::Reference similarly returns its witnessed EffectiveId. Neither helper nor projection recursively calls either function to unfold a target. declaration_identity performs an admitted-declaration lookup and row validation. The nominal cycle therefore cannot cause stack growth with its graph depth. Native 256 KiB execution complements this source argument; it is not represented as a 128- or 4096-frame recursive traversal.
- Complete view assertions include requested/type node, work, record key, field count, IR identity, name, Exact MemberId, presence and target Composite. Stage rows are joined by the public from_stage_rows API to selected-model document/declaration/field provenance. Invalid non-admitted fixture rejection and scalar/unsupported repeat equality remain exercised by the original renamed test.
- Supplied receipts reused under `/home/peter/dev/qualification-logs/ir1003-projection-limits`: fmt.log PASS; lint.log PASS (both Clippy lanes and string-edge scanner); focused.log and green-restored-focused.log PASS (2 tests); red-inner-charge.log and red-inner-charge-status.txt confirm controlled omitted-charge exit 101; red-inner-charge-command.sh restores source and verifies it by cmp. Production source has no diff against the review base. No new compile or aggregate gate was run.
- coverage-fix1.json is the supplied computed trace receipt; inherited classified residuals remain 93, with AC7 the only removed residual from the original 94. This PASS is scoped to AC7 and the changed tests, not the remaining repository criteria.
- AC6 remains out of scope and untraced, with missing-witness versus Unsupported ambiguity owned by 06db3fd4. No production, spec, CI, lockfile, dependency, capability or default change is present.
- Plan completion: not assessed
