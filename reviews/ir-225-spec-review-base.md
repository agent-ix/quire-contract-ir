---
id: SR-11670
title: IR-225 base spec review of catalogue-driven member-result admission
type: SpecReview
analysis: base
scope: agent-ix/quire-contract-ir@39627960464edcc4138c29d20c18dbbcbefc603a (PR 408, diff vs origin/main c2b2a8b, merge-base 055ad1f); spec/checked_package/functional/FR-038-consume-checked-package-v2.md,
  spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md, spec/checked_package/matrix/tests.md
review_set: subset
---

## Summary

Ticket: IR-225. Base checklist over the three-file, additions-only spec diff that adds the "Catalogued member results" rule to FR-038, the new FR-038-AC-247 (PLANNED/UNRUN), its TC-048 step and a tests.md row. Every claim was checked against the code at the PR head (`check_ieee_member_result`, crates/quire-contract-model/src/checked_package/v2/operations.rs:519-574), the catalogue locked in Cargo.lock (quire-verification-contracts e4195bd, contracts/checked-operation-catalog-v1.json), QSpec FR-322 (quire-specification origin/main e29b34b, "Operation families, groups and constraints") and the IR test fixtures. Review-only; no edits, no builds.

## Verdict

**CONDITIONAL**: one high, two medium and one low finding. The defect premise holds and the generalization matches upstream semantics, but the rule as worded refuses shapes this FR explicitly defers, and it says nothing about the existing fixtures it would newly refuse.

What holds:

- Defect premise confirmed by reading. `check_ieee_member_result` returns `None` for every identity except `quire.op.ieee.to_float32` and `quire.op.ieee.to_float64` (operations.rs:529-534). `quire.op.ieee.to_rational` therefore gets no result check. Its operand check only requires the `ieee_float` family on operand 0, so all 8 Float32/Float64 x {rational/rational, float/float, float/rational, rational/float} member/result pairs pass. This agrees with the leader's probe.
- The generalization matches the catalogued semantics. QSpec FR-322 defines the `member` result form as "the type the member names". It says the application's `result_type` "must have the derived family and, when the derived result names a type, be that same type node", and that `member_family` means "the member's type family is the named family or a member of the named group". For all 21 `result: member`/`member: type_argument` entries, "result_type equals the type-argument declaration" is therefore the upstream rule, not an IR invention. No entry has a wrapper, option or refinement result that differs from its type argument. `quire.op.reference.upcast` has a `Reference<T>` type argument (FR-322: "names a `Reference<T>` node, whose family is `reference`"), and `operand_family` maps `composite_type/reference` to `reference` (operations.rs:980-990). The narrowing entries (`numeric.narrow`, `rational.narrow`) take the target bounded domain itself as both type argument and result. FR-038:3150-3154 already requires that for `numeric.narrow`, and every IR fixture does it. `collection.fold` and `collection.reduce` have no `member_family`, and the AC's "where the entry has" covers them.
- The TC-048 lists of 21 type_argument entries and 5 other member entries match the locked catalogue exactly (scripted count: 21 and 5, same identities and kinds).
- QSpec's positive fixtures carry only two in-scope applications (`collection.sum.decimal` in positive-operation-identities.json). Both use `member.declaration` equal to `result_type`, so the conformance suite stays green.
- The five excluded kinds (`field`, `position`, `element`, `relationship_end`, `operation`) are the catalogue's only other `result: member` entries. "Retain their member-specific result rules" is accurate as a no-change statement.
- FR-038-AC-247 is a unique id: `git log --all -S FR-038-AC-247` finds only 3962796, and AC-246 (#407) is the main maximum. PLANNED/UNRUN is honest: `quire matrix` reports AC-247 `untagged`. The TC step re-derives node keys, references, identity projection and package identity. It matches the root `semantic_type` to the candidate `result_type`.
- AC-222 and AC-223 are byte-identical. The diff is 47 insertions and 0 deletions.
- No hash, digest or pin.
- Gates. `quire validate`: 876/876 docs at the head, 880/880 on the clean merge with main (`git merge-tree`, no conflict). `quire coverage --strict`: 49 unbacked at both base 055ad1f and the head, 50 at main and 50 at the merge. `quire matrix` untagged counts: 58 at base, 59 at the head and at main, 60 at the merge. The 59 at main includes AC-246; the 60 at the merge includes AC-246 and AC-247.
- Item (f), the 49. The tests.md row is not what keeps AC-247 out of the unbacked list. The verification cell `Test (TC-048)` does that. Strict coverage accepts a cited test case as backing. AC-232 through AC-235 (`Test (TC-048)`) and AC-132 (`Test (TC-226)`) are untagged and likewise absent from the unbacked list. AC-246 uses a bare `Test` cell and is flagged, which is why #407 went 49→50, even though AC-246 also sits in a TC-048 tests.md row. The author's explanation is inaccurate, but the effect follows the repo's existing convention and is honest: the matrix still shows AC-247 untagged, and the row reads "PLANNED/UNRUN (IR-225); no tagged implementation test yet". No wording change is needed. Readers should use the matrix's untagged count, not the strict total, for planned work.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The new `member_family` clause ("that declared type SHALL satisfy the catalogue's family") contradicts FR-038's own deferral at 3571-3581. That paragraph says the reader does not claim admission semantics for `scalar_type`/`compound_unit` quantities, `bounded_domain`/`model_population` or `composite_type`/`alias` until FR-322 rules. The reader resolves no family for these shapes (`operand_family` returns `None` for `CompoundUnit` and `Alias`, operations.rs:979,991), and today's operand checks skip an unresolved family (operations.rs:900-914). The new clause gives them a meaning: an unresolved family does not satisfy `quantity`/`integer`/`collection_kind`, so the result is a refusal. Failure: QSL FR-094 lowers `convert(v, metre / second)` as `quire.op.quantity.convert` whose type_argument and result are the `compound_unit` node. That package is admitted at main and refuses `ill_typed`/`operator-ineligible` at `result_type` after IR-225, before the upstream ruling. The same happens to `collection.sum.quantity` over a compound unit and to `collection.size`/`count` into an alias of Integer. Either scope the family clause to declared types whose family resolves (unresolved family: the equality rule only, as the deferral requires), or route the shapes through the pending FR-322 ruling explicitly and add the case to AC-247. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3526-3536,3571-3581,4326 |
| FND-002 | medium | AC-247 and the bullet name `operation.member.declaration` but do not say what happens when a type_argument member has no `declaration`. The reader does not check the member shape for `type_argument` (`member_is_well_formed` returns true, temporal.rs:419-426), so it now admits that case. Eleven fixture sites in four IR test files spell the member `{"kind": "type_argument", "type_argument": X}` with no `declaration`: tests/it/checked_package_v2_operand_families.rs:703,995,1027 (AC-210, AC-213); tests/it/checked_package_v2_recursive_leaves.rs:896,981 (AC-230 `collection.convert`); tests/it/admitted_result_context.rs:118,512,527,741,844 (TC-447, `numeric.narrow`/`numeric.convert`); tests/it/admitted_guard_context.rs:159 (TC-450). Failure: implemented as written, the entry-driven check finds no declaration and refuses, so these currently-admitted packages fail their `admits` assertions. The coder then either migrates the fixtures (correct, since QSpec fixtures and FR-038:3152 use `declaration`) without a spec mandate, or adds a fallback that reads the `type_argument` key, an unrequested compatibility path. State the outcome for an absent or malformed `declaration` (code, cause and pointer), and say in the TC step that existing IR fixtures move to the FR-322 `declaration` spelling. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3526-3536,4326; spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:349-366 |
| FND-003 | medium | The generalization, which is the risky half, is not falsifiable by the required cases. The adverse public-reader cases are all `quire.op.ieee.to_rational`. For the other 18 entries the TC says only "apply the same check" and "confirm by catalogue inspection that the check is selected by `result: member` ... rather than an IEEE operation-name allowlist". Inspecting the catalogue cannot show how the code selects the check. Failure: an implementation that extends today's allowlist to {to_float32, to_float64, to_rational} passes every required AC-247 case and every existing test. Meanwhile `quire.op.numeric.narrow`, `quire.op.reference.upcast` or `quire.op.collection.size` with a mismatched `result_type` stay admitted, contrary to "selected by the catalogue entry rather than the operation name". Require at least one non-IEEE adverse case in AC-247 and TC-048: a mismatched `result_type` on, for example, `numeric.narrow` and `reference.upcast`, plus one `member_family` refusal on a non-IEEE entry. Alternatively make the selection an Inspection obligation over the code with that verification method. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4326; spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:362-366 |
| FND-004 | low | The new bullet leaves out the IEEE bullet's "the published validation order and resource accounting unchanged". It also does not place the check among the other application checks. Today the IEEE check runs after operands and before `check_inner_result`, `check_mode_type` and the member checks (operations.rs:795-884). The generalization adds a refusal ahead of `operation-mode-type-mismatch` for `quantity.convert`, `numeric.convert_rounding`, `ieee.from_exact` and `collection.sum.*`. Failure: for a package with both a wrong `result_type` and a wrong rounding mode on `quantity.convert`, the first refusal is unspecified, and two conforming readers may disagree on the reported code (`ill_typed` vs `invalid_package`). State the position, for example "after operand checks and before the mode-type check, as for the IEEE conversions". | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3523-3536 |

## Coverage

Examined the new FR-038 bullet (3526-3536), the adjacent IEEE bullet (3512-3525), the deferral paragraph (3571-3581), the `numeric.narrow` closed-literal rule (3147-3158), AC-213, AC-222, AC-223 and AC-247, TC-048:349-381 and tests.md row 65. Coverage rules: AC-247 has a TC (TC-048), its tests.md row lists it, and the status is PLANNED/UNRUN in all three files. No hash or pin use.

## Examined units

```yaml
scope:
- id: FR-038-AC-247
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: "PLANNED/UNRUN (IR-225). Through `CheckedPackageV2::read`, every catalogued operation with result form `member` and member kind `type_argument` admits only when the application's `result_type` equals that member's declared type node and, where the entry has a `member_family` constraint, the declared type satisfies that catalogue family; this check is selected by the catalogue entry rather than the operation name."
- id: FR-038
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: "Catalogued member results. For every catalogued entry whose result form is `member`, the entry's member kind selects the result rule. When that kind is `type_argument`, the application's `result_type` SHALL name exactly the type node named by `operation.member.declaration`; when the entry has a `member_family` constraint, that declared type SHALL satisfy the catalogue's family."
- id: FR-038-AC-222
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: "Implemented (IR-742). Through `CheckedPackageV2::read`, a `float32_bits` literal declared at a Float32 type and a `float64_bits` literal declared at a Float64 type each admit in otherwise valid packages."
- id: FR-038-AC-223
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: "Implemented (IR-742). Through `CheckedPackageV2::read`, valid `quire.op.ieee.to_float32` over Float64 and `quire.op.ieee.to_float64` over Float32 admit when each `type_argument` and `result_type` name the same correctly typed destination node."
- id: FR-038-AC-213
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: context_only
  excerpt: "Implemented (IR-484). `quire.op.quantity.convert` over a declared-unit quantity and a selected quantity `type_argument` admits mode `{kind: rounding, value: exact}`."
- id: TC-048
  path: spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md
  role: examined
  excerpt: "PLANNED/UNRUN (IR-225; FR-038-AC-247). Build fresh otherwise valid `quire.op.ieee.to_rational` applications for both Float32 and Float64 source operands ... Confirm by catalogue inspection that the check is selected by `result: member` and `member: type_argument`, rather than an IEEE operation-name allowlist; apply the same check to the other catalogued type-argument member results, including non-IEEE conversions."
bindings: []
```

## Dispositions

Round 1; reviewed 9e608222751efbe09fe78c8260e3c5bcec143292 (diff 3962796..9e60822; still based on 055ad1f; merges cleanly with origin/main c2b2a8b).

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | - |
| FND-002 | fixed | - |
| FND-003 | fixed | - |
| FND-004 | fixed | - |
| FND-005 | fixed | - |
| FND-006 | fixed | - |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-005 | low | The TC-048 AC-250 step says to "Exercise the existing member-specific result cases for `field`, `position`, `element`, `relationship_end` and `operation`", and AC-250 claims those five kinds keep their own rules. No IR test or fixture builds `quire.op.tuple.project` (`position`) or `quire.op.collection.index` (`element`): grep of tests/ and crates/ finds none. The reader has no result rule for those kinds either (operations.rs:818-828 maps them to `None`). For two of the five kinds there is therefore no existing case to exercise. Failure: AC-250's exclusion is reported as tested when only `record.project`, `model.navigate` and `model.dispatch_call` cases run. A regression that applies the type-argument equality rule to `position`/`element` entries would pass. Either add one admitted `tuple.project` and one `collection.index` package whose member declaration differs from `result_type`, or narrow the claim to the kinds that have cases. | spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:383-386; spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4356 |
| FND-006 | low | AC-248's sentence "A `model_population` does not gain a settled population family merely because the generic bounded-domain walk reaches its set base" does not pin an outcome, and no TC case exercises it. The FR bullet is clear: for this check `model_population` has no ruled family, so only equality applies, even though `resolve_family` forwards it to `set`. AC-248 instead only denies a `population` family. Failure: an implementation that uses the walk's `set` family refuses `quire.op.collection.size` or `count` with a `model_population` declaration and equal result (`set` does not fit `integer`). The FR bullet requires that package to be admitted, yet AC-248 and its TC still pass. State the outcome in AC-248 (family check skipped for `model_population`) and add one case, or drop the sentence. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3539-3542,4354 |

## Disposition round 1 verdict

**CONDITIONAL** (two new low findings; every original finding fixed).

- FND-001 fixed. The family clause now applies only "When the entry has a `member_family` constraint and the reader resolves a family for the declared type". An unresolved family leaves only the equality check. `compound_unit`, `alias` and `model_population` are named as having no ruled family under the FR-322 deferral, and `quantity.convert` into a `compound_unit` is explicitly preserved (AC-248). The wording is right for each shape:
  - `compound_unit` and `alias`: `operand_family` returns `None` (operations.rs:979,991), and `check_mode_type` skips an unresolved family (2704-2706), so the AC-248 compound-unit admission is achievable.
  - `model_population`: the author's note is accurate. `resolve_family` forwards the bounded domain to its `Set<Reference<T>>` base and yields `set`, so the FR's "for this check ... no ruled family" is a deliberate override of the walk, which the implementation must special-case. The AC side of this is FND-006.
- FND-002 fixed. The bullet closes the shape as `{kind: type_argument, declaration: NodeId}` and says "There is no alternate spelling". A missing, malformed, extra or `type_argument`-keyed member refuses `invalid_package`/`operation-member-mismatch` at `operation.member` (AC-249). That code, cause and pointer are the ones the reader already uses for member defects: `member_mismatch` at `member_or_operation("member")`, operations.rs:731-758, and AC-84. AC-249 and the TC step direct the eleven fixtures to move to `member.declaration`, with "no alternate read".
- FND-003 fixed. AC-250 requires public-reader result-mismatch refusals for `quire.op.numeric.narrow` and `quire.op.quantity.convert`. Both are real admitted fixtures with the right catalogue entries. numeric.narrow (type_argument, `member_family: exact_numeric`) is in tests/it/admitted_result_context.rs:110-122 and admitted_guard_context.rs:152-166. quantity.convert (type_argument, `member_family: quantity`, mode rounding) is in tests/it/checked_package_v2_operand_families.rs:983-1008. An allowlist of the IEEE identities now fails AC-250.
- FND-004 fixed. The bullet places the check after the wire, identity, member-shape and operand checks and before the `result_inner`, state-clause, model-member, mode-type and leaf checks. That matches the code order at operations.rs:780-897; the relationship-member and function-call checks between them apply to no type_argument entry. A two-defect `quantity.convert` refuses at `result_type` first, and resource accounting is unchanged.
- AC-247 through AC-251 are unique (`git log --all -S` finds only 3962796 for AC-247 and 9e60822 for AC-248 to AC-251). All are PLANNED/UNRUN in FR-038, TC-048 and tests.md. The evidence cases do not overlap: equality is to_rational float/rational and rational/float plus ordering; family is to_rational float/float, compound_unit and a non-quantity destination; shape is the member spelling; non-IEEE is narrow and convert; catalogue shape is AC-251. AC-222, AC-223 and #407's AC-246 are byte-identical on the merged tree.
- Gates at 9e60822: `quire validate` 876/876 grammar-clean; `quire coverage --strict` 49 unbacked; `quire matrix` 63 untagged, with AC-247 through AC-251 all untagged. On the clean merge with origin/main c2b2a8b: 880/880, 50 unbacked (only AC-246 is new against base, as at main) and 64 untagged. The five new rows use `Test (TC-048)`, so strict coverage counts them as backed by the cited TC, as in round 0. The matrix shows them untagged, consistent with PLANNED/UNRUN.

## Disposition round 2 verdict

Round 2 reviewed 5918af852bbdbe600181906bd4b71be371902762, which is 9e60822 plus fix commit 05014ba plus a merge of origin/main c2b2a8b. The net change against main is `git diff c2b2a8b..5918af8`: three spec files, 131 insertions and 2 deletions. The FND-005 and FND-006 rows in `## Dispositions` above were added in this round.

**PASS** (no open findings, no new findings).

- FND-005 fixed. AC-250 now states that "valid `tuple.project` and `collection.index` packages with member declarations distinct from their `result_type` remain admitted". The TC-048 step names the existing cases for the other three kinds and requires the code PR to add one admitted public-reader package each for `tuple.project` (`position`) and `collection.index` (`element`) "since neither kind has an existing IR case", so applying the equality rule to either kind must fail the check.
  - The named files exist and build those identities: tests/it/checked_package_v2_model_fields.rs has `record.project`; tests/it/checked_package_v2_model_members.rs has `model.navigate` and `model.dispatch_call`.
  - The new packages can be admitted. operations.rs:818-828 is unchanged and maps `Position` and `Element` to no member check, and `member_is_well_formed` accepts both kinds.
- FND-006 fixed. AC-248 now pins the outcome: "An otherwise valid `quire.op.collection.size` whose `type_argument` declaration is a `model_population` node and whose `result_type` equals that declaration admits: this result check skips the unresolved family even though the generic bounded-domain walk reaches the `Set<Reference<T>>` base and reports `set`". A matching TC-048 case and tests.md wording were added.
  - This is now falsifiable against the code. `resolve_family` forwards `bounded_domain` through `semantic_type` (operations.rs:1247-1325) and returns `set`. `family_fits("set", "integer")` is false, so an implementation that uses the walk refuses and fails AC-248.
  - It agrees with the FR bullet, which says `model_population` has no ruled family for this check.
- Earlier fixes still hold on the merged tree.
  - FND-001 to FND-004 text is unchanged since 9e60822, apart from the AC-248 and AC-250 additions above.
  - AC-213, AC-222, AC-223 and #407's AC-246 rows are byte-identical to c2b2a8b.
  - AC-247 to AC-251 each occur once. `git log --all -S` finds AC-247 only in 3962796 and AC-248 to AC-251 only in 9e60822.
  - There are no conflict markers in spec/ or reviews/.
  - No code or test file changed (no diff under crates/ or tests/).
- Gates at 5918af8: `quire validate` 880/880 grammar-clean; `quire coverage --strict` 50 unbacked, of which the only one beyond the original 49 is main's AC-246; `quire matrix` 64 untagged, including AC-246 to AC-251.
