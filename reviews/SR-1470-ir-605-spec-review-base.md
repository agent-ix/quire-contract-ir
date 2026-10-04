---
id: SR-1470
title: "spec review of PR 291 (typed STD-001 code, FR-044)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir@c109f7eaba2d15f2d8d526f0b0f9dfdfc240a0dd; git diff origin/main...HEAD (base 6fb6e974efd6b9a9c74515ee7e07df250fa4aacf, current main): spec/core/functional/FR-044, STD-001, spec/core/matrix (TC-442, tests.md), spec/kani/functional (FR-030, FR-039), spec/kani/matrix (TC-223, tests.md), spec/model/functional/FR-019, spec/assurance/AD-006, spec/spec.md, spec/tests.md; checked against src/ and crates/quire-contract-model/src at the same sha and quire-contract-codegen origin/main f3ece43 (src and spec, read only)"
review_set: base
---
# SR-1470: spec review of PR 291

## Summary

Ticket: IR-605. Base review of a spec-only PR that adds FR-044 (`Std001Code`
and `Std001CodeError` in a new `code` module of `quire-contract-model`),
registers thirteen bounded-Kani cause codes and `invalid_code_form` in STD-001,
and makes `KaniOutcome.code` and `KaniOutcomeError.code` that type
(FR-030-AC-6).

Measured at the reviewed sha:

- Codes emitted by IR src today (literals outside the family lowerings):
  `kani_identity_invalid`, `kani_population_incomplete`, `kani_bound_invalid`,
  `kani_bound_exhausted`, `kani_population_invalid`, `kani_reference_invalid`
  (`src/kani/abi.rs:92-152`), `kani_capability_request_invalid`,
  `kani_capability_missing` (`src/kani/profile.rs:140,152`), `kani_proved`,
  `kani_counterexample`, `kani_vacuous_proof` (`src/kani/outcome.rs:116-152`).
  That is eleven of the thirteen registered cause codes; the other two,
  `kani_solver_absent` and `kani_backend_absent`, are emitted nowhere today and
  are the merged, planned FR-030-AC-5. The condition and kind of each STD-001
  row match the code site.
- `kani_outcome_kind_invalid` (`src/kani/outcome.rs:170`) is correctly left
  unregistered: merged FR-030-AC-5 already replaces that path with a
  `KaniOutcomeError` coded `kani_outcome_invalid`, merged STD-001 never listed
  it, and the repository has zero tags, so the new stability rule ("never
  removed while a release names it") is not engaged.
- The lowering codes STD-001 excludes (`kani_dispatch_*`, `kani_arithmetic_*`,
  `kani_definedness_*`, `kani_collection_*`, `kani_graph_*`) are emitted only
  from `src/kani/arithmetic.rs`, `collections.rs` and `objects.rs`, as stated.
- Partition: STD-001 at head has 60 code rows. `DiagnosticCode` spells 45 of
  them and `REGISTERED` (FR-044-AC-3) the other 15; no row is in neither and no
  `DiagnosticCode` is unregistered. The AC-3 list is in byte order.
- Form rule `[a-z][a-z0-9_]*`, 1 to 64 bytes, no trailing `_`, no `__`: all 60
  STD-001 codes conform (longest 35 bytes), and so does every `kani_*` literal
  in codegen's src (longest `kani_witness_multiple_assertions_refused`, 40).
- Newtype versus enum: codegen mints its own codes into `KaniOutcome` through
  `KaniOutcome::non_success` (`kani_profile_input_mismatch`,
  `kani_corpus_dependency_invalid`, `kani_corpus_serialization_failed`,
  `kani_corpus_identity_collision`;
  `src/kani/generate/corpus/bounded_kani_corpus.rs:401-524`), and after IR-347
  the lowering codes become codegen's too. A closed IR enum would force those
  into IR's registry, so a form-only newtype is the sound choice for the
  `KaniOutcome` field. FND-001 is about what that choice means for QSL.
- No compatibility layer: no `From<String>`/`From<&str>`, no deprecated alias,
  no dual field; the break is stated plainly in FR-044 "Break" and FR-030.
- Strict coverage: 23 unbacked at origin/main, 29 at head; the six added rows
  are exactly FR-044, FR-044-AC-1, AC-2, AC-3, FR-030-AC-6 and TC-442 (diffed
  row by row). `quire validate` over spec/plan/reviews is clean for the PR's
  files.

Examined (role examined): FR-044 statement, Contract, Behavior sections and
AC-1 to AC-3; STD-001 Description, Typed Code Form, Bounded Kani Cause Codes;
FR-030 new paragraph, AC-6 and Status; FR-039 Error surface; FR-019 serde
paragraph and Public items row; AD-006 seam bullet and Decision C; TC-442;
TC-223 edits. Context only: merged FR-030-AC-4/AC-5, AD-006 G-4, codegen
FR-030 (`spec/kani/functional/FR-030-ir-outcome-terminal-map.md`) and NFR-005.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | `Std001Code` states form, not membership, and FR-044 says codegen builds its own codes with it, yet FR-044's Dependencies says QSL's `DeclineCode` takes this type for its `Ir` arm, and the QSL ruling recorded in codegen's FR-030 says a `Declined` code is carried "in the registry that issued it". Codegen's `kani_profile_input_mismatch` and `kani_corpus_*` today, and the lowering codes (`kani_dispatch_unowned` etc.) after IR-347, ride `KaniOutcome.code` and would land in `DeclineCode::Ir`, attributed to IR's registry. The name `Std001Code` makes the same membership claim the type refuses to check. FR-044 must say what the `Ir` arm carries (an IR-family code from IR's or codegen's registry, told apart by `is_registered`) or carry the issuer; it cannot be left to QSL-351 | spec/core/functional/FR-044-typed-std001-code.md:73-87, 124-127 |
| FND-002 | medium | The only way to build an unregistered code from a literal is the fallible `Std001Code::new`. Codegen's corpus refusals (`bounded_kani_corpus.rs:401,445,514,522`) and, until IR-347, IR's own lowering sites pass literals that are always valid, so each must either `.expect()` (codegen NFR-005 forbids generation panics) or carry an unreachable error path. FR-044 should specify a compile-time path, e.g. `new` as a `const fn` so a consumer declares `const` codes checked at build time | spec/core/functional/FR-044-typed-std001-code.md:60-71 |
| FND-003 | medium | The Contract invariant "a registered code is spelled by one constant" and Stability's "a registry row has one constant and the reverse" are false for 45 of STD-001's 60 rows: the `DiagnosticCode` codes have no `Std001Code` constant and are spelled by `DiagnosticCode` variants via `From<DiagnosticCode>`. Restrict both statements to codes outside `DiagnosticCode`, or say a row has one constant or one `DiagnosticCode` variant | spec/core/functional/FR-044-typed-std001-code.md:29, 98-104 |
| FND-004 | low | Order against IR-347 is not stated. If the IR-605 code change lands first, the lowerings still in IR emit their codes through `non_success`, which now takes `Std001Code`, and `KaniProviderRecord.cause: String` (`src/kani/outcome.rs:88`, cloned from `code` at :110) still exists; neither FR-044 nor FR-030 says how those are typed in the interim. One sentence naming the order (or the interim form) is enough | spec/core/functional/FR-044-typed-std001-code.md:106-111; spec/kani/functional/FR-030-bounded-kani-domain-and-outcomes.md:49-60 |

## Verdict

Changes needed, no high finding. The core claims test true: every code
STD-001 registers matches IR src or a merged planned AC, the partition with
`DiagnosticCode` is exact, the form rule fits every registered and every
codegen code, `kani_outcome_kind_invalid` is correctly superseded, there is no
compatibility layer, and the coverage delta is exactly the six new planned
rows. FND-001 to FND-003 should be fixed in this PR because they fix the type's
contract that QSL and codegen build against; FND-004 can be a sentence here or
the first decision of the code PR.

Author's open questions: the QSL `Ir` arm typing is FND-001 and belongs in this
PR. Order against IR-347 is FND-004 (state it here, low). Whether `kani_proved`
and `kani_counterexample` should carry a code (`Option<Std001Code>`): no change
needed now; both are emitted today, and an `Option` would change the wire form
(`null`), contradicting "the wire form does not change", so note it for a later
ticket. Codegen fallout: the constructor shape is FND-002 and belongs here; the
call-site edits are codegen's code PR.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-005 | low | The rewritten Contract invariant says a `Std001Code` "is built only by a validating constructor, a registered constant or std001_code!", but "Registered codes" keeps the infallible `From<DiagnosticCode>`, a fourth construction path that is none of the three. Add `From<DiagnosticCode>` to the invariant | spec/core/functional/FR-044-typed-std001-code.md:29, 110-112 |

## Dispositions

Round 1, reviewed at cace2678410ee69ae71e656e26f80541ba2e31e4.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | cace267 |
| FND-002 | fixed | cace267 |
| FND-003 | fixed | cace267 |
| FND-004 | fixed | cace267 |

Round 2, reviewed at cf90862eda642d465b90d4cc13fa3f51bfbade57.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-005 | fixed | cf90862 |
