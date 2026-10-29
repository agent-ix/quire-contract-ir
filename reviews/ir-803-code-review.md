---
id: SR-11664
title: "IR-803 typed composite operator code review (with Rust review)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; PR #410 PR diff.: crates/quire-contract-model/src/checked_package/v2/composite_operands.rs, crates/quire-contract-model/src/checked_package/v2/mod.rs, crates/quire-contract-model/src/lib.rs, tests/it/checked_package_v2_composite_operands.rs, FR-038, TC-048, spec/checked_package/matrix/tests.md, FR-019"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/TC-048
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/FR-019
    type: reviews
---

# IR-803 typed composite operator code review (with Rust review)

## Summary

Ticket: IR-803. PR: quire-contract-ir#410 against (merge base equals origin/main). One pass of code-review with the rust-review lane folded in, scoped to `git diff origin/main...HEAD` (8 files). Authority: FR-038-AC-246 and the FR-038 accessor contract as merged by spec PR #407. Gates run in a throwaway worktree with QUIRE_SPEC_LANGUAGE_DIR at quire-spec-language origin/main: `cargo test --locked --workspace --test it checked_package_v2` 448 passed; `cargo test --locked -p quire-contract-model --lib v2` 171 passed (the author reported 170; the diff adds no unit test, so the difference is the filter, not the change); `cargo fmt --all -- --check` clean; `cargo clippy --locked --workspace --all-targets -- -D warnings` clean; `quire validate` 887/887 grammar-clean; `quire coverage --strict` 49 unbacked, 0 contradicted (main: 50; the only row removed is FR-038-AC-246).

## Verdict

**PASS** with two low findings. Two hand-reasoned checks and eleven scratch mutants confirm the behaviour.

- **Public type.** `CheckedCompositeOperator` is a closed `Eq | Ne` enum. It has no `#[non_exhaustive]` and derives `Clone, Copy, Debug, Eq, PartialEq`, the same derives as the fieldless sibling `CheckedUnsupportedCompositeOperand`. The type and both variants carry rustdoc.
- **Re-exports.** The enum is re-exported from `v2/mod.rs` and from the crate root `lib.rs`, in alphabetical position like its siblings. The 12-line `lib.rs` hunk is a rustfmt reflow caused by one insertion. A sorted token diff of the `lib.rs` re-export list against main shows exactly one added identifier, `CheckedCompositeOperator`, and nothing removed or renamed.
- **FR-019 table.** The FR-019 public-items row adds the name in sorted order (`Operands` < `Operator`), and nothing else changed in that row.
- **Mapping.** The identity is still read from the node that was already resolved under the single `graph.node` charge. The read comes after the UnknownNode, NotApplication and MissingOccurrence checks, and the uncatalogued-identity UnknownOperator check stays ahead of it. The only `_` arm returns `IneligibleOperator`, so the mapping has no default value. No new lookup, `meter.charge` or index access was added. `consumed_work` is unchanged from main: the pre-existing exact-work oracles (tests/it/checked_package_v2_composite_operands.rs:632-633 for both eq and ne, :1087) are untouched by the diff and pass. A symmetric extra-charge mutant and a second-lookup mutant both fail them.
- **Rust lane.** No `unwrap`, `expect`, indexing or `as` cast was added on wire data, no `#[allow]` was added, and there is no `unsafe`.
- **CG consumer.** CG's composite builder can map `Eq` to `Equal` and `Ne` to `NotEqual` from the returned value through the crate-root re-export, with no JSON read and no second lookup. The type is `Copy` and exhaustively matchable. It does not derive `Hash`, which the builder does not need for a two-way map.

Mutants run against `cargo test --test it checked_package_v2_composite_operands`:

- Every one of these mutants was killed: constant Eq, constant Ne, swapped mapping, wildcard default to Eq, extra charge on both arms, extra charge on the ne arm only, second `graph.node` lookup, operator taken from another application in the graph, and operator derived from the occurrence.
- The `#[non_exhaustive]` mutant and the extra-variant mutant both fail to compile `inspect_public_fields` with E0004.
- No mutant survived.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The private `EligibleOperator` enum and the second match that converts it one-to-one into `CheckedCompositeOperator` add a type and a match that carry no behaviour. A single match on the identity, with `CheckedCompositeOperator::Eq`/`Ne` arms and the `_ => return Err(IneligibleOperator)` refusal, is equally exhaustive. A later reader may assume the two enums can diverge. | crates/quire-contract-model/src/checked_package/v2/composite_operands.rs:46-49, 406-418 |
| FND-002 | low | The accessor rustdoc still reads "Returns authored domains of `structural.eq` or `structural.ne` operands in argument order". It does not mention the typed operator that the result now carries, although FR-038 makes returning the operator with the operands part of the accessor contract. | crates/quire-contract-model/src/checked_package/v2/composite_operands.rs:362 |

## Evidence

- Token diff: `diff <(git show origin/main:crates/quire-contract-model/src/lib.rs | tr -s ',\n' '\n' | sort) <(tr -s ',\n' '\n' < crates/quire-contract-model/src/lib.rs | sort)` prints only `> CheckedCompositeOperator`.
- Refusal order at composite_operands.rs:380-414 is: UnknownNode, NotApplication, MissingOccurrence, UnknownOperator (uncatalogued), IneligibleOperator (catalogued but not eq/ne). The wildcard-default mutant is killed by `tc_048_external_error_consumer_retains_authentic_node_and_application_loci`, which uses `boolean.and`.
- The scratch mutant logs are kept outside the repo, and the review worktree was restored clean (`git status --porcelain` empty) after the run.

## Examined scope

```yaml
scope:
- id: FR-038-AC-246
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: "For a reader-admitted quire.op.structural.eq application, composite_application_operands returns CheckedCompositeOperator::Eq together with its operands; for an otherwise equivalent reader-admitted quire.op.structural.ne application it returns CheckedCompositeOperator::Ne together with its operands."
- id: FR-038
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: "Selecting the typed operator from that identity shall add no work charge and shall leave consumed_work, the existing refusal variants, and operand order unchanged."
- id: FR-019
  path: spec/model/functional/FR-019-rust-library-interface.md
  role: examined
  excerpt: "checked_package (V2 composite operands) public types row, now including CheckedCompositeOperator."
- id: TC-048
  path: spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md
  role: context_only
  excerpt: "For AC246, admit otherwise equivalent structural.eq and structural.ne applications through the public reader, with the same ordered operand children and an authentic occurrence in each."
```

## Dispositions

Round 1, against fix (`git diff origin/main...HEAD`, which changes only composite_operands.rs). Re-run gates: `cargo test --workspace --test it checked_package_v2` 448 passed, model `--lib v2` 171 passed, fmt and clippy clean, `quire validate` 887/887, strict coverage 49 unbacked / 0 contradicted. Mutants re-run on the new head and killed: swapped mapping and wildcard-default-to-Eq (by the AC-246 and boolean.and AC-181 tests), extra charge and second lookup (by the exact-work tests at:632-633 and:1087). The refusal order and `consumed_work` are unchanged, and the fix round adds no finding.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | `EligibleOperator` removed; one match on the identity yields `CheckedCompositeOperator::Eq`/`Ne`, and its only `_` arm refuses `IneligibleOperator` (composite_operands.rs:402-410) |
| FND-002 | fixed | Rustdoc now reads "Returns authored domains in argument order with their typed operator: `Eq` for `structural.eq` or `Ne` for `structural.ne`." (composite_operands.rs:357-358) |
