---
id: SR-11631
title: gap-analysis and test-oracle strength of IR-742 IEEE reader admission
type: SpecReview
analysis: gap-analysis
scope: agent-ix/quire-contract-ir; branch code/ir742-reader-admission; diff origin/main...HEAD;
  crates/quire-contract-model/src/checked_package/v2/operations.rs, tests/it/checked_package_v2_ieee_admission.rs, spec/checked_package
review_set: subset
---

## Summary

Ticket: IR-742. Method: quoin gap-analysis (planless; "Plan completion: not
assessed") with test-oracle strength folded in. PR: none yet. Computed matrix:
`quire coverage --scope .` reports 452/515 rows backed at origin/main
and 454/515; FR-038 224/236 to 226/236; `--strict` 49 unbacked rows
at both. `quire matrix` lists FR-038-AC-222 as tagged by
tests/it/checked_package_v2_ieee_admission.rs:89 and :177, and FR-038-AC-223 by
:284 (both untagged at base). No new production code lacks an owning
requirement: `check_ieee_member_result` traces to FR-038-AC-223 and the FR-038
normative IEEE bullet.

Mutants were run in a throwaway worktree against the three tagged tests:

| Mutant | Result |
| --- | --- |
| accept either IEEE width in `literal_defect` | killed (both AC-222 tests) |
| skip `result == member` | killed |
| skip the `member_family` fit | killed |
| ignore the conversion operand family refusal | killed (same-width case) |
| refuse at `operation/member/declaration` instead of `result_type` | killed |
| refuse with `operation-member-mismatch` cause | killed |
| check only the first of two nested literals | **survived** (also survives every `checked_package_v2` integration test) |
| accept a Float32 literal under Float64 only when nested | killed |
| accept a Float64 literal under Float32 only at a standalone body root | killed |
| drop the check for `to_float64` | killed |
| remove the `check_ieee_member_result` call | killed |

## Verdict

**FAIL (coverage gaps; no false trace).** The three tagged tests assert
exact typed code, cause, pointer and node locus through the public reader on
freshly derived identities, and every tag is truthful. AC-223 is strongly
pinned. Two gaps remain: the FR-038 normative literal rule is wider than
AC-222's opposite-width cases and its familyless-type branch is both untested
and admitted (SR-11630 FND-001), and no test places a mismatched literal after
another literal in one body.

## Examined scope

```yaml
- id: FR-038-AC-222
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Retaining each value kind and changing only its declared `type` to the opposite IEEE width, while re-deriving the affected node and package identities, refuses `ill_typed`/`operator-ineligible` at that literal's `type`, including in an operand position whose catalogued family would otherwise accept the declared type. Neither mutation is admitted or refused merely at an enclosing operand.
- id: FR-038-AC-223
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: For each identity, changing `result_type` and the enclosing application's root-node `semantic_type` together to the source-width node while retaining the valid destination `type_argument` ... refuses `ill_typed`/`operator-ineligible` at the application's `result_type` by the `member` result form; changing `type_argument`, `result_type` and that root-node `semantic_type` together to the source-width node likewise refuses there by the catalogued `member_family` constraint.
- id: FR-038/IEEE-literal-type-and-catalog-result
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Independently of an enclosing operand position, a `float32_bits` literal requires a declared `type` in the `float32` family, and a `float64_bits` literal requires one in the `float64` family. A mismatch refuses `ill_typed`/`operator-ineligible` at that literal's `type`.
- id: TC-048/IEEE-literal-and-catalog-result-admission
  path: spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md
  role: context_only
  excerpt: Place each in an otherwise eligible argument position so an enclosing operand-family rejection cannot stand in for the literal's own kind/type check.
```

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The FR-038 normative IEEE literal rule requires a declared type in the `float32`/`float64` family, but no test covers a declared type that resolves to no family; that branch is admitted at HEAD (root cause SR-11630 FND-001). A control with a familyless declared type, standalone and as an argument, would pin it. | tests/it/checked_package_v2_ieee_admission.rs:86-119 |
| FND-002 | medium | Test-oracle gap: the AC-222 operand test mutates only `arguments/0`, so a mutant that checks only the first literal in a body survives the IEEE tests and the whole `checked_package_v2` suite. A case with the mismatched literal at `arguments/1` (after a valid literal) kills it. | tests/it/checked_package_v2_ieee_admission.rs:148-152 |

## Dispositions

Round 1, verified (`git diff origin/main...HEAD`).

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | New `tc_048_ieee_bits_reject_every_familyless_declared_type` (tagged TC-048, FR-038-AC-222) covers 66 cases (11 familyless node kinds x two IEEE kinds x standalone and both argument positions) with exact code, cause, pointer and node locus; it fails against the old `continue` (mutant killed). |
| FND-002 | fixed | New `tc_048_ieee_literal_width_checks_both_arguments` puts the mismatched literal at arguments/1 after a valid one, and both-wrong at arguments/0; the "check only the first of two nested literals" mutant is now killed. Full re-run of 14 mutants: 13 killed; 1 survivor (fail open on catalog-shape drift) is equivalent under the compiled-in catalog. Coverage 454/515, strict unbacked 49 (== main), AC-222 tagged at :90, :218, :284, :320 and AC-223 at :407. |
