---
id: SR-987
title: "code review of PR 232 (TC-222 tests)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir@122c21f6cacb25222b35b6629d19ed90e541d81a; tests/it/checked_package_v2_adr002_members.rs, tests/it/main.rs"
review_set: subset
---
# SR-987: code review of PR 232 (TC-222 tests)

Former id: SR-625 (cited by the marker on IR-448).

## Summary

Ticket: IR-448. Code review with the rust-review lane over the three new TC-222 tests that back FR-344-AC-1 to AC-3. Every case was mutation-tested against the reader in `crates/quire-contract-model/src/checked_package/v2/mod.rs`, and each mutant was reverted.

- M1: an unknown `node_tag` falls back to `model`. The AC-1 test goes red.
- M2: an unknown `semantic_form` falls back to `namespace`. The AC-1 test goes red on its second case.
- M4: `validate_body` accepts any `relation`/`population` body. The AC-2 test goes red.
- M3: `#[serde(deny_unknown_fields)]` is removed from `CheckedSemanticNodeV2`. The AC-3 test **stays green**.

The trace convention (doc lines plus `#[trace]`) matches the repo. The helpers reuse `tests/it/support/checked_package.rs`. Clippy, fmt and the full test suite pass under `make ci`.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The AC-3 test never exercises the node gate TC-222 names: it only checks the refusal path with `ends_with("/subsets")`, and the refusal it observes is in the mirrored identity projection | tests/it/checked_package_v2_adr002_members.rs:462-482 |

## Finding Detail

- FND-001: `mutated()` calls `refresh_identity`, and that call copies the edited node into `identity_preimage.identity_projection`. `CheckedPackageWireV2` decodes `identity_preimage` before `semantic_graph`, so the refusal actually observed is `unknown_member` at `/identity_preimage/identity_projection/14/subsets` (and `/14/redefines`). It is not at the node member that FR-344-AC-3 describes. The loose `path.ends_with("/{member}")` check accepts either location. Removing `deny_unknown_fields` from `CheckedSemanticNodeV2` (M3) leaves the test green. The closed decode of the `semantic_graph` node object, which the TC-222 case 3 prose names as the mechanism, therefore has no oracle.

  Measured fix: add the member after `refresh_identity`, so that it is not mirrored. That case refuses `UnknownMember` at exactly `/semantic_graph/nodes/14/subsets`, and under M3 it becomes `MalformedWire`. Assert the exact pointer `/semantic_graph/nodes/{at}/{member}` for that case. Either keep the mirrored case with its own exact projection pointer, or drop it.

## Scope

- tests/it/checked_package_v2_adr002_members.rs `tc_222_an_unrecognized_tag_or_form_refuses_at_the_graph_gate`, examined: real oracle; M1 and M2 each kill it.
- tests/it/checked_package_v2_adr002_members.rs `tc_222_a_population_body_outside_the_closed_term_grammar_refuses_at_the_body`, examined: real oracle; M4 kills it.
- tests/it/checked_package_v2_adr002_members.rs `tc_222_an_extra_node_member_refuses_as_unknown_member_in_the_closed_decode`, examined (FND-001).
- tests/it/main.rs module registration, examined: clean.
- Binding TC-222 to FR-344-AC-1, AC-2 and AC-3, examined: each tag names the AC its test covers.

## Verdict

Changes requested, one medium. The AC-1 and AC-2 tests are real oracles. The AC-3 test does pass on real behaviour: the package is refused and never admitted. But it pins the wrong gate and cannot detect the loss of the node's closed decode. The fix is a few lines, and it was measured to work.

## Dispositions

Round 1, reviewed at 276749f984690d9aad26b28347369e32e5845508. The mutation was re-run: with `deny_unknown_fields` removed from `CheckedSemanticNodeV2`, the AC-3 test now fails (`MalformedWire` != `UnknownMember` at `/semantic_graph/nodes/14/subsets`). With the reader restored it passes, and `make ci` runs 183/183 integration tests green.

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 276749f |
