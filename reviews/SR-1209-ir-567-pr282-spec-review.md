---
id: SR-1209
title: "PR #282 FR-034-AC-7 amendment spec review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir@75f880ea12a818f63282971e557e580759da8711; spec/output_mapping/functional/FR-034-assemble-output-package-atomically.md, spec/output_mapping/matrix/tests.md, spec/tests.md (git diff origin/main...HEAD, base 543cd8a8b6a5fe2a1974617869824a75f6c38629)"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-034
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/FR-032
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-033
    type: references
  - target: ix://agent-ix/quire-contract-ir/TC-043
    type: references
---
# SR-1209: PR #282 FR-034-AC-7 amendment spec review

## Summary

Ticket: IR-567. PR agent-ix/quire-contract-ir#282, a spec-only PR.

The diff rewrites FR-034-AC-7, adds sentences to two FR-034 Behavior bullets
(the canonical-identity bullet and the decimal-string bullet), and changes the
FR-034 and TC-043 rows of the output-mapping matrix and the Output mapping row
of `spec/tests.md`.

Sub-analyses covered in this one artifact: ambiguity, contradiction,
testability and atomicity, EARS/form, and trace/matrix consistency. The
clause-by-clause test check is in SR-1210.

Each claim was read against `crates/quire-contract-model/src/output_mapping.rs`
at the reviewed sha:

- Admission (`admit_controlled`, lines 2251-2290) builds `RequestIdentityMaterial`,
  encodes it with `quire_canonical::to_vec` under `maximum_request_bytes`, and
  keeps only `request_bytes`. It computes no digest.
- `identity_bytes` (2643-2651) is the single place a ceiling is chosen.
- Record and package identities are SHA-256 of their canonical bytes.
- `MappingLimits::new` (625-660) refuses any zero member with `zero_limit`.
- `OutputByteRegion::new` (1049) admits `start` 0 when `start < end`.
- `RequestResourceShape` leaves out `maximum_request_bytes`. The package
  material carries all seven limits.

The whole `spec/` tree was grepped for a stale request digest or zero-limit
clause. None remains:

- FR-032 Behavior and FR-032-AC-6 say "request identity material" and claim no
  digest.
- FR-034-AC-3 ("every zero ... limit is classified") agrees with FR-032-AC-2
  and STD-003 `zero_limit`.
- FR-034-AC-6 is unchanged and consistent.
- FR-034 has no AC-8.

## Verdict

**APPROVE WITH NITS** (one medium and one low finding). The amendment is true to
the code and removes both defects IR-567 names: the unbuildable zero limit and
the request digest that does not exist. It contradicts nothing in FR-032,
FR-033 or FR-034-AC-1..6.

Leaving out a limit of `1` is acceptable. AC-7 no longer claims a bottom-of-range
limit. `serialize_decimal` is one serializer shared by `MappingLimits` and
`OutputByteRegion`, so the region bounds `"0"` and `"1"` exercise the
small-value spelling.

`quire validate` passes, with 1 grammar finding (FR-014, unchanged).
`quire coverage --strict` reports 23 unbacked rows and 0 contradicted statuses,
none of them in output mapping.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-034-AC-7 is one ~170-word compound of four independent assertions: (a) the record and package digests equal written-out text, (b) the request material equals written-out text, (c) six u64 spellings, (d) u64::MAX and u64::MAX-1 both admit with distinct material. Clause (c) pairs a list of values with a list of strings by position, and "including" leaves the region set open. A test can satisfy one part and the row still reads green. Split it into AC-7 (a+b), a new AC (c) naming each value and its string per member, and a new AC (d). | spec/output_mapping/functional/FR-034-assemble-output-package-atomically.md:104 |
| FND-002 | low | The new Behavior sentence calls the canonical bytes "the request identity", but admission throws them away and keeps only their length. No request identity is held or exposed, and AC-7, FR-032 and the matrix call the same thing "request identity material". The added sentences are also descriptive ("is", "computes no digest") inside a "shall" bullet list. Suggested wording: "The request identity material is encoded only to meter it: admission shall keep its length as `request_bytes` and shall retain neither the bytes nor a digest of them." | spec/output_mapping/functional/FR-034-assemble-output-package-atomically.md:58-63 |

## Dispositions

Round 1 was reviewed at 111d164c2ee950ceaf44a6732a3da64363f2e5df.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 111d164. AC-7 is split three ways. AC-7 now covers the identities and the request text and length. AC-8 lists each value and its string member by member: the request material has 40, 1100, 130, 4200, 36 and 77777, plus 18446744073709551615; the package material has 9007199254740993, the same six and its ceiling; the record regions are 0/1 and 9007199254740993/18446744073709551615. AC-9 covers u64::MAX and u64::MAX-1, which both admit with different material. |
| FND-002 | fixed | 111d164. The sentence is now two "shall" bullets. The first reads "The admission step shall encode the request identity material only to meter it, keep its length as `request_bytes`, and retain neither the bytes nor a digest of them." The second reads "The assembler shall derive each record identity and the package identity as the SHA-256 digest of its canonical bytes." |
