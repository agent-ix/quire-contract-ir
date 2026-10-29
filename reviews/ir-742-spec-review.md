---
id: SR-11632
title: spec-review of IR-742 status wording for FR-038-AC-222 and AC-223
type: SpecReview
analysis: base
scope: agent-ix/quire-contract-ir;; spec/checked_package/functional/FR-038-consume-checked-package-v2.md,
  spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md, spec/checked_package/matrix/tests.md
review_set: subset
---

## Summary

Ticket: IR-742. Method: spec-review limited to the changed status lines of FR-038 and TC-048. PR: none yet. Each AC row differs from origin/main only in its
status prefix (`PLANNED/UNRUN (IR-742).` to `IMPLEMENTED (IR-742; tagged reader
tests passed).`); the criterion text after the prefix is byte-identical. The
TC-048 paragraph and section status, and the tests.md row status and heading,
change the same way; the tests.md "Required cases" cell is unchanged.
`quire validate --scope . 'spec/**/*.md' 'plan/**/*.md' 'reviews/**/*.md'
--summary` exits 0 (873/873 grammar-clean); `quire coverage --strict` reports
49 unbacked rows at base and head; `quire matrix` shows both ACs tagged at head
and untagged at base.

## Verdict

**PASS with one low finding.** The status change is truthful to the AC rows'
own text: the tagged tests pass and cover both opposite-width literal cases and
all AC-223 cases. The wording is inconsistent with neighbouring rows, and the
FR-038 normative bullet these rows extend is still not fully met by the code
(SR-11630 FND-001), which is a code defect, not a wording one.

## Examined scope

```yaml
- id: FR-038-AC-222
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: IMPLEMENTED (IR-742; tagged reader tests passed). Through `CheckedPackageV2::read`, a `float32_bits` literal declared at a Float32 type and a `float64_bits` literal declared at a Float64 type each admit in otherwise valid packages.
- id: FR-038-AC-223
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: IMPLEMENTED (IR-742; tagged reader tests passed). Through `CheckedPackageV2::read`, valid `quire.op.ieee.to_float32` over Float64 and `quire.op.ieee.to_float64` over Float32 admit when each `type_argument` and `result_type` name the same correctly typed destination node.
- id: TC-048/IEEE-literal-and-catalog-result-admission
  path: spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md
  role: examined
  excerpt: IMPLEMENTED (IR-742; tagged reader tests passed). Build otherwise valid V2 packages using the published `quire.op.ieee.to_float32` and `quire.op.ieee.to_float64` entries
- id: TM/IEEE-admission-coverage
  path: spec/checked_package/matrix/tests.md
  role: examined
  excerpt: '| FR-038 | FR-038-AC-222, FR-038-AC-223 | TC-048 | ✅ IMPLEMENTED (IR-742; tagged reader tests passed) |'
```

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | Status vocabulary differs from the neighbouring rows (`Implemented (IR-484).`) and embeds a run claim ("tagged reader tests passed") that no gate checks and that goes stale on the next change; `Implemented (IR-742).` matches the file's convention and leaves the evidence to the computed matrix. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4313-4314 |

## Dispositions

Round 1, verified (`git diff origin/main...HEAD`).

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | The AC-222/223 rows, the TC-048 section and the tests.md row now read `Implemented (IR-742).`, matching the neighbouring rows. The criterion text after the prefix is byte-identical. The TC-048 overview sentence "IR-742's tagged reader tests passed" is unchanged and matches the IR-800 sentence beside it, so it is the file's convention, not a defect. `quire validate` exits 0. |
