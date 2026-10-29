---
id: SR-558001
title: "Spec base review of IR-1013 PR #558"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-058-admitted-clause-execution-point.md, spec/checked_package/matrix/TC-462-admitted-clause-execution-point.md, spec/spec.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-058
    type: reviews
---
# SR-558001: Spec base review of IR-1013 PR #558

## Summary

Ticket: IR-1013. Spec-only draft PR, frozen,
(current main). It adds FR-058, TC-462 and one master-index paragraph. No
existing FR, AC or TC is touched.

Checked against the repo at that head, quire-spec-language origin/main,
quire-specification origin/main (FR-341, FR-342), and CG drafts #572, #573 and #574:

- `quire_contract_model::ExecutionPoint` exists with exactly the four variants
  FR-058 names (`crates/quire-contract-model/src/identity.rs:937`). It is owned by
  Contract IR. QSL's native `ClauseBinding.execution_point`
  (`src/checking.rs`) uses that type. QSL's modern `qsl-package::export_bindings`
  (`qsl-package/src/export.rs:70`) carries no execution point, and nothing in
  `qsl-package/src` names one. The Dependencies section states this correctly.
- The selection `(&CheckedNodeId, &CheckedOccurrence)` and the first three refusals
  match the existing `CheckedPackageV2::direct_integer_comparison` exactly
  (`comparison_operands.rs:260-290`). The new accessor reuses one selection model
  and invents no parallel one.
- In V2 each `claim` occurrence of a clause node is one declaration of the clause
  (QSL `clause_root.rs`). Nodes are content-keyed, so two equal invariant bodies
  share a node and carry two claim occurrences. This makes AC-2's two-claims-on-one-node
  case real, and the claim-occurrence key necessary.
- FR-341 and FR-342 bodies carry no point. The accessor is marked Planned,
  QSL-1082 and the QSpec carrier are named as prerequisites, and no
  implementation is claimed. No QSL type is copied. No compatibility or fallback
  path appears.
- `quire validate` passes on all 1380 docs, with 0 grammar findings. `quire coverage --strict`
  reports FR-058 at 0/6 and lists FR-058-AC-1 through AC-6 as unbacked among 59
  unbacked rows. This is the expected Planned state, and neither the PR nor the
  index claims coverage.
- The CG drafts consume `quire_contract_model::ExecutionPoint` through "the planned
  IR-1013 typed accessor", with `MissingExecutionPoint` on absence. That matches
  FR-058's output and ownership statement. Cross-repo note, not a finding here:
  the CG drafts phrase the lookup as per "selected V2 node", but FR-058 (correctly)
  keys it per claim occurrence. CG's own selection is a claim, so CG should
  pass the claim occurrence through.

## Verdict

**PASS with findings.** No high finding. One medium: the IR-side admission and
retention rule for the future carrier has no owning requirement. One low: the
Inspection-verified AC-6. Both are mergeable for a Planned spec draft, if the medium
is either fixed in-PR or recorded as a named follow-up.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-058 Behavior 6 reads "the admission-retained typed relation", and AC-6 expects a carrier "admitted and retained by IR". No IR requirement, here or as an FR-038 follow-up, owns that admission. Nothing states the reader rules: at most one point per claim occurrence, refusal of duplicate or conflicting points, a point on a non-claim occurrence, or point/clause-kind compatibility. Dependencies defers "wire and reader rules" to QSpec, so the singular "exact" point of Behavior 5 is undefined if two points are retained, and there is nothing for an implementer to refuse against | spec/checked_package/functional/FR-058-admitted-clause-execution-point.md:66-67; spec/checked_package/functional/FR-058-admitted-clause-execution-point.md:81; spec/checked_package/functional/FR-058-admitted-clause-execution-point.md:85-87 |
| FND-002 | low | FR-058-AC-6 is verified by `Inspection`, which `quire coverage --strict` reports as minting no source symbol. The row stays unbacked after implementation too, so FR-058 can never reach 6/6. It also conjoins a temporal clause ("no claimed implementation while the QSL-1082/QSpec carrier is absent") that is vacuous once the carrier lands. Precedent exists (FR-055-AC-4, FR-056-AC-7, FR-057-AC-1/9) | spec/checked_package/functional/FR-058-admitted-clause-execution-point.md:81 |

## Examined scope

- **FR-058** (examined): Description, Inputs, Outputs, Behavior 1-7, Dependencies; relationships to FR-038, QSpec FR-341/FR-342, CG FR-011.
- **FR-058-AC-1** (examined): four producer-origin points `startup`/`tick`/`adjust` pre/post returned exactly; synthetic package gives no producer credit.
- **FR-058-AC-2** (examined): two claims on one invariant node bound to `startup` and `tick` return distinct points; renaming changes the result; no kind/anchor default.
- **FR-058-AC-3** (examined): `UnknownClause`, `NotStateClause`, `MissingClaimOccurrence` in first-applicable order with supplied ids.
- **FR-058-AC-4** (examined): `MissingExecutionPoint` on absent point and on a point keyed to another package, clause or claim.
- **FR-058-AC-5** (examined): external CG-style consumer, exhaustive refusal match, no QSL import, body JSON or `export_bindings`; determinism over repeat and clone.
- **FR-058-AC-6** (examined): inspection for one QSpec-governed carrier, no QSL copy, no compat path, no premature claim.
- **TC-462** (examined): steps 1-4 and expected results cover AC-1 to AC-6. Producer-origin cases are UNRUN until QSL-1082. Fixtures are read from authoritative checkouts.
- **spec/spec.md** (examined): new FR-058/TC-462 paragraph; links resolve.
- **CG FR-011 / FR-002 / FR-004 drafts** (context_only): consumer of the accessor.

## New findings (disposition pass 1)

Checked at fix-round.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | low | The fix names a new IR reader code, `CheckedPackageRefusalCode::InvalidExecutionPointCarrier`, and AC-7/AC-8 hard-code it, but the same Outputs bullet says QSpec's carrier contract "must assign these cases to its closed diagnostic pairing before implementation". The existing `CheckedPackageRefusalCode` variants pair with QSpec diagnostic codes (snake_case, e.g. `invalid_package`, `ill_typed`). If QSpec pairs these cases with an existing code and cause, AC-7/AC-8 must be rewritten, or IR's code diverges from the QSpec vocabulary. The FR does not say which wins | spec/checked_package/functional/FR-058-admitted-clause-execution-point.md:49-53; spec/checked_package/functional/FR-058-admitted-clause-execution-point.md:105-106 |

## Dispositions

Round 1, checked against the repo (diff `..`; vs main only FR-058, TC-462 and six added lines in spec/spec.md; no existing FR/AC touched; QSL-1082 prerequisite and Planned status retained; `quire validate` 1380/1380 grammar-clean; strict coverage FR-058 0/8, 61 unbacked rows, AC-6 no longer reported as Inspection).

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | New "Admission and retention" section (lines 55-71). `CheckedPackageV2::read` retains at most one point per claim and refuses `DuplicatePoint` (equal or conflicting), `NonClaimPoint` and `IncompatiblePoint` with a kind table, revalidating at admission. Backed by AC-7, AC-8 and TC-462 step 4 |
| FND-002 | fixed | AC-6 is now a `Test`: a compile control plus a reader control with source bytes dropped and a source check. The vacuous temporal clause is removed, and strict coverage no longer reports AC-6 as Inspection-only |

Round 2, checked against the repo. The diff from is two added lines in the FR-058 Outputs; the PR body now says eight Planned ACs.

| FND | outcome | reason |
| --- | --- | --- |
| FND-003 | deferred | Deferred to QSL-1082 and the QSpec carrier contract. FR-058 Outputs now says "These Planned names, including AC-7 and AC-8, follow that QSpec pairing exactly when QSL-1082 lands; QSpec's assigned code and cause prevail", which settles precedence. The concrete code and cause are assigned when the carrier lands |
