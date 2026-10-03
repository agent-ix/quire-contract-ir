---
id: SR-1014
title: "code review (with rust-review lane) of PR 266, IR-274 part A: v2 identity digests through quire-canonical"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir@476e9b418e69aaaf7e796e3923e3d8321a812d50; git diff origin/main...HEAD (merge base ebea678 = current main): Cargo.toml, Cargo.lock, crates/quire-contract-model/src/checked_package/{common.rs,shared.rs}, v2/{encode.rs,identity.rs,lower.rs,lower/ceiling_tests.rs,mod.rs,model_members.rs,model_members/tests.rs,operations.rs,state.rs}, tests/it/*; read-only consumers quire-contract-codegen@origin/main, quire-driver@c0b2f31, quire-spec-language@eddbdc54"
review_set: subset
---
# SR-1014: code review of PR 266

## Summary

Ticket: IR-274 (part A). Reviewer-only; nothing was edited in the repository.
Spec: FR-038 "Every identity digest is computed through quire-canonical" and
FR-038-AC-89..95, as merged in #264.

Measured:

- **Gates on the PR head**, in a detached worktree outside the repository:
  `make fmt-check`, `lint`, `test`, `deny`, `cargo-audit` and `audit-unsafe`
  all exit 0. `make spec` exits 2 with the expected baseline: validate passes,
  1 grammar finding (FR-014), and `--strict` reports 23 unbacked rows and 0
  contradicted.
- **Digests are byte-identical.** I recorded every node key, `package_id`,
  lowered `ir_id`, lowered `package_id` and the full lowered package bytes on
  origin/main (ebea678), and again on the PR head. I used five admitted
  fixtures: the PR's three, plus the model-members `package_over` package
  (structural, model-declaration and dispatch application keys) and the
  complete-v1 `mixed_fixture`. The `fixtures.txt`, `model_members.txt` and
  `mixed.txt` dumps are identical, and all five `.bytes` files compare equal
  with `cmp`. The PR's `RECORDED_*` constants equal the values I measured on
  main. The dimension-term order is unchanged by construction: `DimensionTerm`
  and `CheckedNodeId` differ only at the digest, and the domain is fixed.
- **The dependency is right.** `quire-canonical` is a first-party git
  dependency on `branch = "main"` with `features = ["serde_json"]`, and there
  is no rev or tag pin. The lockfile records b4bb97a5, which is the merge
  commit of quire-canonical #7 and its current main. `deny.toml` already
  allow-lists the source. Nothing is vendored: no `fn dismantle`, no walker
  over `Value` and no `impl Encode for Value` remain in `crates/`.
- **AC-91's part-A deletions are done.** `digest_json`, `digest_bytes`,
  `value_to_vec`, `write_value` and `write_number`, and `dismantle` are gone,
  and drops go through `quire_canonical::drop_value`. The lowering `expect` is
  gone, and so is the `u64::MAX` cap in `NominalIdentityPreimage::digest`.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The selected model document is parsed a second time into a `serde_json::Value` after the digest (`strict_json_value(bytes)`), against FR-038's "It is not parsed a second time into a `serde_json::Value`, so there is one reader for the document". The two readers can disagree, for example on a document `quire_canonical::read` accepts and `strict_json_value` refuses, and that disagreement surfaces as `byte-digest-mismatch` after a digest that matched. Either read the declarations from the `quire_canonical::Document`, or route a spec amendment that allows the second parse after the digest | crates/quire-contract-model/src/checked_package/v2/model_members.rs:1070 |
| FND-002 | medium | When the lowered package is over the ceiling, every record is replaced with `Failed`, but `CompleteContractPackageV2` still carries every lowered and dependency node: `lowered()` and `dependencies()` return them, and only `encoding` is `None`. A caller then sees all records `failed` beside a package that holds those nodes as lowered. That breaks the type's own contract ("Builds the call's package from its `lowered` records alone"; "A refused request contributes nothing to it"). Clear `lowered`/`dependencies` when encoding fails, or build the package after the record replacement | crates/quire-contract-model/src/checked_package/v2/lower.rs:377 |
| FND-003 | medium | Cross-repo breakage of the public API, with no lockstep change. (a) `CheckedPackageRefusal` gains the public field `document_pointer`. The struct is not `#[non_exhaustive]`, so every struct literal breaks. quire-contract-codegen origin/main builds one in production code at `src/oracle/scalar/mod.rs:3368`, and quire-driver at `tests/drive.rs:1516`. (b) `NominalIdentityPreimage::digest` now takes `limit_bytes` and returns `Result`. Codegen's `tests/checked_package_support/base.rs:58-60` calls `.digest()` with no argument. (c) `CompleteContractPackageV2::canonical_bytes()` and `package_id()` now return `Option`. No consumer uses them today: codegen reads `source_package_id()` only, and QSL is unaffected. Codegen depends on IR at `branch = "main"`, so its next lock refresh will not compile. Driver pins rev 48ab5dc and breaks when that pin moves. The field itself is specified (AC-93), so the change is right, but it needs a codegen follow-up queued before or with the merge | crates/quire-contract-model/src/checked_package/shared.rs:313 |
| FND-004 | low | Encoder refusals on derived keys are swallowed with `.ok()` / `if let Ok`: `ModelOwners` skips a declaration whose key is refused, `slot_type` types nothing, `state.rs` and `check_model_member` compare against `None`. The refusal then surfaces as a different code (`missing_declaration`, signature or eligibility refusals). FR-038 says an encode refusal while reading refuses "`invalid_semantic_graph` at the node for a node key". It is only reachable when a model document's identity and node strings bring the ~260-byte structural preimage over `limits.bytes`, but it is an error swallowed rather than returned | crates/quire-contract-model/src/checked_package/v2/model_members.rs:858 |
| FND-005 | low | `CheckedPackageV2`'s `PartialEq` (and `Debug`) now include the retained read limit `bytes`. Two admissions of identical bytes under different `limits.bytes` compare unequal. This public semantics change is not in the spec. Equality was content equality before. If the limit must ride along, keep it out of `eq` or say so in FR-038 | crates/quire-contract-model/src/checked_package/v2/mod.rs:410 |

## Verdict

Changes requested. The identity move itself is correct. Digests, keys and
lowered bytes are byte-identical on five fixtures, measured independently.
The dependency is clean, and the deletions AC-91 asks of part A are complete.
FND-001 and FND-002 are behaviour defects to fix in this PR. FND-003 needs a
codegen (and later driver) follow-up queued before or with the merge.

Rust-review lane: no `unsafe`, no new `unwrap`/`expect` in production code,
and integer conversions are checked (`i128::try_from` for counts,
`saturating_*` in the exponent parse). The 2^53 text classifier
`exceeds_2_pow_53` is correct for every case I traced: integer, fraction,
exponent, saturating exponent and leading or trailing zeros
(`9007199254740992.5` refuses, `9007199254740992.0` admits). The history
carries a WIP commit (3f1b6ab) and a merge commit (76e1963) that holds the
substantive change. That is acceptable only for a squash merge. The net diff
has no formatting churn beyond import reflow forced by the deletions, and no
artifacts.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-006 | medium | The byte-limit failed record does not match the amendment in quire-contract-ir #267 at its current head ec21b9c. That draft says "Only the canonical-bytes refusal is a `bytes` failure", and any other encoder refusal "is not folded into a `failed` record with an invented `consumed`: lowering returns it as the encoder's own refusal". `required_bytes` does two things against that. It maps every `Error::Limit` to `bytes` without checking `limit.kind` is `CanonicalBytes`. It also folds every non-limit error into `Failed { Bytes, consumed: ceiling + 1 }`. The code conforms to the merged FR-038, which says only "`failed`". If #267 lands as written, three things must change: check the kind, add a path for a non-limit encoder refusal (a record variant or an error), and assert `consumed == required` in the tests. If #267 adopts the code's folding instead, nothing changes here | crates/quire-contract-model/src/checked_package/v2/lower.rs:284-289 |

## Dispositions

Round 1, reviewed at 249acbcd028536586d0e8162b1a15ddfc59e75a9 (one squashed commit on ebea678). These are measured
in a detached worktree outside the repository, since removed. All gates exit 0
(`make spec` 2 at the baseline: 1 grammar finding, 23 unbacked, 0
contradicted). The 5-fixture digest dump and all five lowered `.bytes` files
are identical to ebea678.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 249acbcd028536586d0e8162b1a15ddfc59e75a9 |
| FND-002 | fixed | 249acbcd028536586d0e8162b1a15ddfc59e75a9 |
| FND-003 | accepted-no-change | The change is specified (AC-93's `document_pointer`), and `#[non_exhaustive]` would break every external literal anyway. The PR body records the breakage: codegen `src/oracle/scalar/mod.rs`, codegen `tests/checked_package_support/base.rs`, driver `tests/drive.rs`. `Failed.limit_kind` compiles in codegen (`..` patterns), but codegen maps every `Failed` to `LoweringWorkExhausted`, so a byte failure is mislabeled there. A Linear follow-up ticket for codegen and driver must exist before merge; the PR body alone is not a tracker entry |
| FND-004 | fixed | 249acbcd028536586d0e8162b1a15ddfc59e75a9 |
| FND-005 | fixed | 249acbcd028536586d0e8162b1a15ddfc59e75a9 |

### Round 2

Round 2, reviewed at 31ba6f5f7443e3cb01258f55bb66bd84669b2290 (one commit on ebea678). These are measured in a
detached worktree outside the repository, since removed. All gates exit 0
(`make spec` 2 at the baseline: 1 grammar finding, 23 unbacked, 0
contradicted). The 5-fixture digest dump and all five lowered `.bytes` files
are identical to ebea678.
`required_bytes` now matches the final amendment #267 (head 8adb938): only
`LimitKind::CanonicalBytes` records `required`, and every other refusal records
`limit + 1`, saturating. Mutants: the canonical-bytes count replaced by
`limit + 1` is killed, and a wrapping (non-saturating) `limit + 1` is killed.
Mapping every limit kind to `required` survives. That is the object-buffer
refusal, which needs a buffer over 4 GiB; #267 and the matrix row both state it
is untested.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-006 | fixed | 31ba6f5f7443e3cb01258f55bb66bd84669b2290 |
