---
id: SR-12016
title: "IR-832 gap analysis of IR PR #427"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-038-consume-checked-package-v2.md (AC-258..261, Typed parameter declaration), spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md, spec/checked_package/matrix/tests.md, spec/model/functional/FR-019-rust-library-interface.md, crates/quire-contract-model/src/checked_package/v2/parameter_declaration.rs, crates/quire-contract-model/src/checked_package/v2/structural.rs, tests/it/checked_package_v2_parameters.rs, tests/it/checked_package_v2_model_fields.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: references
  - target: ix://agent-ix/quire-contract-ir/TC-048
    type: references
---

# IR-832 gap analysis of IR PR #427

## Summary

Ticket: IR-832, PR quire-contract-ir#427. This is a planless gap analysis; plan completion was not assessed. Authority is FR-038-AC-258..261 and the TC-048 section as merged in spec PR #425 (origin/main), with QSL FR-092 "Parameter nodes" read-only from quire-spec-language origin/main.

Computed matrix (quire 0.36.2, `quire matrix --strict`):

- On origin/main: 55 untagged and 1 tagged only by ignored tests.
- At head: 51 untagged and 1 tagged only by ignored tests.
- The untagged set at head is the base set minus exactly FR-038-AC-258, 259, 260 and 261.
- `quire coverage --strict` reports 43 unbacked rows and 0 contradicted on both base and head. Both commands exit 1 on both sides; that is the pre-existing baseline.
- `quire validate` over spec/plan/reviews: 938/938 docs clean.

Hand mutants were run in a throwaway worktree with `it -- checked_package_v2` and model `--lib checked_package`. The unmutated control failed 0 on both base and head.

**Shared admission check.** Each mutant was applied to origin/main's inline `parameter_defect` and to head's `parameter_binding`:

| Mutant | Base: killed by | Head: killed by |
| --- | --- | --- |
| leading-zero level | `tc_048_a_malformed_parameter_node_refuses` | the same test, plus the new MalformedBody unit test |
| type form ignored | same as above | same as above |
| name literal kind/type ignored | same as above | same as above |
| extra aggregate members accepted | same as above | `tc_048_a_malformed_parameter_node_refuses` only |
| non-identifier name accepted | same as above | not run (see below) |

- The extra-members case is the one the accessor unit list does not cover. It is still caught at admission.
- The non-identifier mutant was not run on head: the mutation string matched four sites, so it was not applied. The new unit test has a non-identifier case, `"1x"`, so it would be caught.
- **Conclusion:** existing admission tests catch the shared-check mutants exactly as before.

**Accessor and operand mutants (head):**

| Mutant | Result | Killed by |
| --- | --- | --- |
| name/level swap | killed | 4 tests |
| shared-node return (any parameter id returns the first parameter's declaration) | killed | 3 tests |
| form check dropped | killed | 2 tests |
| UnknownNode reported as NotParameter | killed | 2 tests |
| MalformedBody reported as NotParameter | killed | the unit test |
| leading zeros trimmed from level (`"10"`, `"0"`) | killed | 3 tests |
| operand vector reversed | killed | AC-261 test, plus 3 scalar tests |
| operand ordinal off by one | killed | scalar tests AC-159..164 only, not the AC-261 test |
| tag check dropped | survived | none |

The tag-check mutant is equivalent on any admitted package, because `parameter` exists only in the closed `ValueForm` vocabulary.

## Reviewed scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| FR-038-AC-258 | examined | For an admitted `value`/`parameter` node, `parameter_declaration` returns `Ok` whose `node` equals the supplied id, whose `name` equals the node's admitted `"name"` text literal and whose `level` equals the canonical decimal text of its admitted `"level"` integer literal; for parameters named `a` at level `0` and `b` at level `1` it returns exactly `("a", "0")` and `("b", "1")`, and a level of `10` is returned as `"10"` and not read as an integer. |
| FR-038-AC-259 | examined | `parameter_declaration` over an id absent from the graph refuses `UnknownNode`; over an existing `value`/`literal`, an `application` node and a `function` node it refuses `NotParameter` for each; over a parameter node whose `body` a crate-internal post-admission mutation changed to lack the `"name"` binding, to give `"name"` a non-text value, to give `"name"` a non-identifier value, to type the name literal at a node that is not `scalar_type`/`text`, and to give `"level"` the non-canonical value `"010"`, it refuses `MalformedBody` for each. |
| FR-038-AC-260 | examined | Two admitted parameter nodes named `x` at levels `0` and `1` have distinct ids, and `parameter_declaration` returns `level` `"0"` for one and `"1"` for the other with `name` `x` for both ... The result type has no member naming a function, owner or declaration, which a compiled field-by-field construction of `CheckedParameterDeclaration` shows. |
| FR-038-AC-261 | examined | ... calling `parameter_declaration` on each `GraphChild` id returned by `scalar_application_operands` yields `a` then `b` in operand ordinal order, and the same application with its arguments swapped in a separately admitted package yields `b` then `a`. An `InlineLiteral` operand carries no `CheckedNodeId`, so the type of `CheckedScalarOperandChild::InlineLiteral` offers no id to pass to the accessor. |
| TC-048 "Typed parameter declaration" | examined | The `MalformedBody` mutations are crate-internal unit tests beside the accessor, since they need the private wire. |
| QSL FR-092 "Parameter nodes" | context_only | body: `aggregate{[binding "name" = the name as a text literal typed at the text scalar node (T3), binding "level" = the binder's level as an integer literal typed at the `Integer` node (T2)]}`. |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-261's last sentence is false as written, and its test cannot fail on it. `CheckedScalarOperandChild::InlineLiteral` is `{ application: CheckedNodeId, occurrence, ordinal }`, so it does carry a `CheckedNodeId`, namely the containing application's, and that id can be passed to the accessor. The test only asserts `matches!(&operands[1].child, InlineLiteral { .. })`, and the `..` would match any field set. Restate the AC as "carries no operand node id; its `application` id names the containing application, which the accessor refuses `NotParameter`". Then assert `parameter_declaration(&application) == Err(NotParameter)`, and destructure the variant without `..`. The sentence came in through the spec-review round 1 fix (SR-12003 FND-003). | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4508; tests/it/checked_package_v2_model_fields.rs:793-813 |
| FND-002 | low | The `MalformedBody` unit tests, and the unit test also tagged AC-258, run on a hand-built package that was never admitted (`kinds: Vec::new()`, node `3` typed at a bare `integer` scalar). AC-259 says "a crate-internal post-admission mutation", and FR-038-AC-140 says module tests can build a `CheckedPackageV2` from an admitted one. The test passes only because the accessor ignores `kinds` and re-decodes from the wire, which ties it to SR-12015 FND-001. This matches the scalar_operands AC-162 precedent, and the healthy admitted read is covered by tests/it for AC-258, so the AC wording is met in practice. It is still not met literally, and the AC-258 tag on the unit test adds no admitted evidence. Either start the fixture from an admitted package, or say in TC-048 that the fixture is a never-admitted package of admitted shape. | crates/quire-contract-model/src/checked_package/v2/parameter_declaration.rs:123-171 |

## Coverage

Bindings were checked against the test bodies:

- `tc_048_parameter_declaration_returns_name_and_level_text` covers AC-258. It checks a/0, b/1 and c/"10", plus `node`, with the expected values built independently of the accessor.
- `tc_048_parameter_declaration_refuses_unknown_and_non_parameter_nodes` covers AC-259's public half. It uses an unknown id, a literal, an application and a function node, all on an admitted package.
- `tc_048_post_admission_body_mutations_refuse_malformed_body` covers AC-259's MalformedBody half: all 5 AC cases plus 8 more.
- `tc_048_parameter_declaration_reads_a_healthy_body_and_refuses_other_nodes` is tagged AC-258 and AC-259, on the unit fixture (see FND-002).
- `tc_048_parameters_of_one_name_differ_by_level_and_carry_no_function` covers AC-260. The ids are distinct, the levels are "0" and "1", and the struct is built and then destructured without `..`, so a new field fails compilation.
- `tc_048_parameter_declarations_follow_scalar_operand_order` covers AC-261. Both argument orders are tested, and the `Int[0, 9]` range is asserted as (0, 9). The InlineLiteral clause is weak (see FND-001).
- No tag is on a test that does not assert its AC.
- The FR-019 row matches the compiled exports: tc_058 passes.

## Verdict

Mergeable after FND-001. FND-001 is a spec-text and test fix inside this PR: restate the AC-261 sentence and add one assertion. FND-002 is low and can be fixed or accepted with a TC-048 note.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1: AC-261 now reads "An `InlineLiteral` operand carries no operand `CheckedNodeId`: the only id its `CheckedScalarOperandChild::InlineLiteral` holds is `application`, the containing application's, and `parameter_declaration` over that id refuses `NotParameter`." The test destructures `InlineLiteral { application: carried, occurrence: at, ordinal }` without `..`, asserts `(carried, at, ordinal) == (&application, &occurrence, 1)` and asserts `parameter_declaration(carried) == Err(NotParameter)`. A mutant that accepts any known kind as a parameter is now killed by this test. TC-048 has the matching sentence. The FR-038 matrix row text changed only from "🚧... are PLANNED/UNRUN" to "✅... are implemented with tagged TC-048 tests" (a character diff of the row confirms nothing else changed), which matches the AC table and tests.md. |
| FND-002 | fixed | Round 1: the fixture now carries `kinds` decoded from the wire, and both its doc comment and TC-048 say it is a never-admitted package of admitted shape. This is the option the finding offered, so I accept the note in place of an admitted fixture. The rationale the note gives is wrong; that is FND-003 below. |
| FND-003 | fixed | Round 2: TC-048 no longer claims that a module test cannot admit. It now presents the never-admitted fixture as a choice that follows the `scalar_application_operands` precedent, and it says that a module test can admit a package, as the composite operand tests do (composite_operands.rs:1289-1300). The remaining clause ("would need its own keyed graph for no added check") is a judgement I accept: the accessor reads only the body and the admitted kinds, and the admitted path is covered by the tests/it AC-258..261 tests. The fixture is unchanged, and the text is now true. |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | low | The new TC-048 sentence gives a reason that is false: "because a module test cannot derive the structural keys an admitted read requires". `composite_operands.rs`'s own module tests admit a package in-crate. Their `package()` calls `CheckedPackageV2::read` and panics unless the result is `Admitted`, with the comment "Every corruption below starts from a genuinely reader-admitted package". FR-038-AC-140 says the same. The note's description of the fixture is accurate. Only its stated reason is wrong, and it would mislead the next author into thinking an admitted in-crate fixture is impossible. Drop the "because" clause, or reword it as a choice, for example "kept, as in the scalar_operands precedent, rather than admitting in-crate as composite_operands does". | spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:2260-2262; crates/quire-contract-model/src/checked_package/v2/composite_operands.rs:1289-1300 |
