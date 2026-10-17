---
id: SR-12170
title: "IR-861 base spec review of IR PR #441"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir@243009d0d25f271521b9a372fe30bfc4f8bde109 (PR #441, base origin/main 555639d54ff55f30d5d401b7748342d717fb5007); git diff origin/main...HEAD: spec/checked_package/functional/FR-038-consume-checked-package-v2.md (38 Verification cells gain ' (TC-048)': AC-165..175, 177..182, 186..201, 214, 236..238, 246) and spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:1459-1490 (new section 'Selected variant types'); checked against the TC-048 procedure sections, every other TC document under spec/, spec/checked_package/matrix/tests.md, tests/it/checked_package_v2_model_members.rs, tests/it/checked_package_v2_enum_order.rs and reviews/ir-777-*.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: references
  - target: ix://agent-ix/quire-contract-ir/TC-048
    type: references
---

# IR-861 base spec review of IR PR #441

## Summary

Ticket: IR-861. The PR is spec-only. A word diff confirms that the FR-038 change
adds only the text ` (TC-048)` to 38 Verification cells and changes no criterion
text. The TC-048 change adds one new procedure section for AC-236..238.

For each of the 38 ids, I matched the criterion text against the TC-048
procedure text. I did not rely on headings alone:

- AC-165..173: "FCD relationships and named ends", items 1-10 (165: item 1;
  166: item 2; 167: item 3; 168: item 4; 169: item 5; 170: items 6-7; 171:
  item 8; 172: item 10, with item 11 superseding it; 173: item 9).
- AC-174, 175, 186..196: "Private intake refusal-origin checks", steps 1-7
  (174: step 3; 175: step 5; 186: step 2; 187: steps 1-2; 188: step 1; 189:
  step 3; 190: step 4; 191: step 5; 192: steps 5-6; 193: step 6; 194: step 7;
  195: step 7's Clippy instruction; 196: step 4).
- AC-177..182, 246: "Typed package-authored composite operand projection",
  in a paragraph per criterion.
- AC-197..201: "Structural inequality operand extension", steps 1-5. AC-201 is
  the source Inspection in step 5.
- AC-214: "Reader peak memory", which has both the measurement and the
  inspection paragraph.
- AC-236..238: the new "Selected variant types" section.

I found no id bound to TC-048 dishonestly. No other TC document names any of
the 38 ids, either by id or inside a "through" range. The only other places
they appear are the FR-038 coverage row and the TC-048 row in tests.md.

The new section matches the tagged tests. Step 1 matches
`selected_variant_types_field_parameter_and_result` and
`selected_variant_requires_its_exact_model_owner`. Step 2 matches the three
AC-237 tests. Step 3 matches the two AC-238 tests in `checked_package_v2_model_members.rs`
and `ordinary_scalar_enum_still_refuses_nonidentifier_cases`. Each code,
cause and pointer the section names is one the tests assert. A section inside
TC-048 is the right design, not a separate TC row: TC-048 is the single
strict-reader TC, and every other FR-038 reader cluster is a section of it.
The ticket's "new TC row if needed" option is not needed.

## Reviewed scope

| Unit | Role | Path | Excerpt |
| --- | --- | --- | --- |
| FR-038-AC-165..173 | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4720-4728 | PLANNED / UNRUN (IR-661). A selected, independently authored FCD-shaped document ... Test (TC-048) |
| FR-038-AC-174, AC-175 | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4729-4730 | IMPLEMENTED; final-head runtime/Clippy checks UNRUN ... (IR-663 identity retention / private constructors) ... Test (TC-048) |
| FR-038-AC-177..182 | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4732-4737 | Implemented by `composite_application_operands`; verified by the TC-048 composite operand tests ... Test (TC-048) / Test, Inspection (TC-048) |
| FR-038-AC-186..192, AC-196 | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4741-4747,4751 | IMPLEMENTED; final-head runtime/Clippy checks UNRUN, full gates pending (IR-663 ...) ... Test (TC-048) |
| FR-038-AC-193, AC-194 | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4748-4749 | IMPLEMENTED; source Inspection completed (IR-663 ...) ... Inspection (TC-048) |
| FR-038-AC-195 | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4750 | IMPLEMENTED; final-head runtime/Clippy checks UNRUN ... (IR-663 consumer lint gate) ... Test (TC-048) |
| FR-038-AC-197..201 | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4752-4756 | Implemented (IR-690) ... Test (TC-048) / Inspection (TC-048) |
| FR-038-AC-214 | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4776 | CODE-GATED (IR-575 ...) ... Test, Inspection (TC-048) |
| FR-038-AC-236 | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4793 | IMPLEMENTED (IR-777). A selected Semantic IR model declaration of a nonempty variant type, with a self-typed declaration node and exact model owner, is admitted through `CheckedPackageV2::read`. ... Test (TC-048) |
| FR-038-AC-237 | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4794 | IMPLEMENTED (IR-777). A value typed by the selected variant accepts an `enum_value` whose `case` is an exact selected UTF-8 case name ... Test (TC-048) |
| FR-038-AC-238 | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4795 | IMPLEMENTED (IR-777). An empty selected case declaration refuses `invalid_model_binding`/`malformed-declaration` ... Test (TC-048) |
| FR-038-AC-246 | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4832 | Implemented (IR-803). For a reader-admitted `quire.op.structural.eq` application ... Test (TC-048) |
| TC-048 "Selected variant types" | examined | spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:1459-1490 | Verify, through `CheckedPackageV2::read` and the tests in ... that a selected Semantic IR model declaration of a nonempty variant type is admitted and typed exactly. |
| TC-048 FCD, intake, composite, ne, memory sections | context_only | spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:859-965,1035-1090,1572-1614,1676-1991 | Typed package-authored composite operand projection; Structural inequality operand extension; Reader peak memory; FCD relationships and named ends; Private intake refusal-origin checks |
| AC-236..238 tagged tests | context_only | tests/it/checked_package_v2_model_members.rs:474-747, tests/it/checked_package_v2_enum_order.rs:282-297 | /// Trace: FR-038-AC-236 / -237 / -238 |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | AC-192 is bound to TC-048, but the TC-048 private-intake procedure has no step for the criterion's positive clause, "Exact sufficient work succeeds". Step 5 covers the work-exhaustion side ("bounded work exhaustion; assert the same first refusal ... no partial package"). Step 6 covers "No second parse, raised ceiling or diagnostic collection". No step measures the sufficient work and asserts that exactly that budget succeeds. The tagged tests at intake_retention.rs:309 and :495 may do it, but the procedure does not say so. The procedure text is pre-existing; this PR makes the binding. Fix: add the exact-budget success clause to step 5. | spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:1957-1969, spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4747 |
| FND-002 | low | The new section requires outcomes that the criteria it cites do not state, and then says (line 1489) that they "satisfy FR-038-AC-236 through FR-038-AC-238 with the stated code, cause and pointer". In step 1, "Remove the owner and require `malformed_wire` at the node" appears nowhere in AC-236, which only covers changing the owner to an absent or different declaration. In step 2, the code and cause `ill_typed`/`operator-ineligible` for `enum.lt` and `structural.eq` are not in AC-237, which says only "refuse that unordered variant at the first ineligible argument". Both come from the tests, not the criteria. Both match the reader's general wire and operator-eligibility rules, so this is not wrong, but the TC adds obligations that no criterion owns. Fix: label these as extra test controls outside the criterion, or add them to the criteria. | spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:1468-1481, spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4793-4794 |
| FND-003 | low | Step 3 says "Admit an ordinary scalar enum whose cases include a non-identifier spelling and require `invalid_semantic_graph`". The input is refused, not admitted, so "Admit" contradicts the expected outcome. The test `ordinary_scalar_enum_still_refuses_nonidentifier_cases` reads the package and requires the refusal. Fix: change "Admit" to "Read" (or "Author and read"). | spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:1486-1487, tests/it/checked_package_v2_enum_order.rs:284 |

## Verdict

PASS on the base review: all 38 bindings are honest, and the new section is
accurate to the criteria and to the tests. All three findings are low and none
blocks merge on its own. The blocking finding for this PR is in SR-12171
FND-001: the TC-048 Traces To cell does not list AC-236..238.
