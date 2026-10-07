---
id: SR-3140
title: "Spec review of the IR-690 structural inequality composite operand extension"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir branch spec/ir690-structural-ne-domains (IR-690, specification only, one commit ahead of main); spec/checked_package/functional/FR-038-consume-checked-package-v2.md; spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md; spec/checked_package/matrix/tests.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: references
  - target: ix://agent-ix/quire-contract-ir/TC-048
    type: references
---

## Summary

Ticket: IR-690. Base spec review of the change that extends the IR-651
composite operand accessor (`CheckedPackageV2::composite_application_operands`)
from `quire.op.structural.eq` to `quire.op.structural.ne`, adds FR-038-AC-197
through FR-038-AC-201 as PLANNED/UNRUN, adds the matching TC-048 procedure and
updates the FR-038 and TC-048 matrix rows. Sub-analyses run alongside this
base: integrity (SR-3141), criterion strength (SR-3142) and gap analysis
(SR-3143). Their findings are recorded there and are not repeated here.

Architecture was checked first:

- One owner. The extension is stated as a widening of the existing accessor's
  eligibility check, reusing the retained admission-derived node index,
  `operations::resolve_family_with`, `operation_catalog().family_fits` and
  `structural::reference_target`. All four seams exist in the model crate today
  and the accessor already calls them. No second domain derivation, public
  type, schema or encoder is allowed.
- The owning checked-operation catalog gives `structural.ne` the same operand
  families (`structural_kind`, `structural_kind`), the same `same_type`
  constraint and the same `operand:0` leaf source as `structural.eq`, so reusing
  the eq family/domain path is sound and no catalog expansion is implied.
- No vendoring, compatibility layer, fallback or depth cap: the text forbids a
  recursion ceiling, caller-drawn bounds and a per-call uncharged index, and
  keeps the existing per-logical-visit work meter.
- Inequality is explicitly not a complement domain and negates no bound, which
  agrees with QSL FR-358 (NotEqual negates Equal at the verdict, over the same
  operand pairs).
- The cited consumers are right: CG FR-033 AC-12 names the actual Eq/Ne
  operation, and QSL FR-358 carries `EqualityOperator` Equal/NotEqual. The
  change claims no CG/QSL replay, whole-domain coverage or scalar Eq/Ne.

Consistency with IR-663 retention (FR-038-AC-174, AC-175, AC-186 through
AC-196): no overlap. The extension touches the post-admission accessor only;
reader admission, refusal metadata retention and first-refusal order are kept
unchanged, as stated.

Hygiene: the diff adds no commit hash or hex identifier of seven or more
characters, no local filesystem path and no date. `quire validate` is clean
(524 of 524 documents). `quire coverage --strict` exits 1 with 31 unbacked
rows: the five new ones are FR-038-AC-197 through AC-201, which are PLANNED
and untagged by design, and the other 26 come from files this change does not
touch.

## Examined units

- FR-038 "Typed authored domains of composite equality and inequality operands", opening statement (examined)
- FR-038 composite error table, `IneligibleOperator` row (examined)
- FR-038 structural.ne extension paragraphs (examined)
- FR-038-AC-197, FR-038-AC-198, FR-038-AC-199, FR-038-AC-200, FR-038-AC-201 (examined)
- FR-038-AC-177 through FR-038-AC-182 (examined, for consistency)
- FR-038-AC-174, FR-038-AC-175, FR-038-AC-186 through FR-038-AC-196 (context_only)
- TC-048 composite operand section and structural inequality extension section (examined)
- tests.md FR-038 and TC-048 rows (examined)
- checked-operation catalog entries for structural.eq and structural.ne (context_only)
- CG FR-033-AC-12 and QSL FR-358 operator table (context_only)

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

Architecture PASS: one owner path, no duplication, vendoring, compatibility
layer or depth cap, and catalog-consistent families. The specification is
CONDITIONAL on the sub-analysis findings. Integrity SR-3141 FND-001 (high) is
the blocking one: FR-038-AC-182 still says structural.ne remains a declared
consumer gap, which contradicts the new criteria.
