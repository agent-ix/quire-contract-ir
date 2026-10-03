---
id: SR-1152
title: "spec review of PR 276 (flat v2 wire, no depth limit, FR-038-AC-114 to AC-118)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir@dd407ae54c8e813c9d6e8efb97d6d435d6947ec1; git diff origin/main...HEAD (base e6fc881e6315c9763d1783e152dc1ed59e070427): spec/assurance/AD-004, spec/checked_package (FR-038, FR-040, TC-048, matrix/tests.md); checked against merged QSpec at quire-specification origin/main 2f846f88bd646781acc4704da4e46a3b54bf0116 (FR-322, FR-341, FR-370, FR-440, TC-233, TC-303, TC-427, proposals/checked-package-v2/fixtures/adverse.json), crates/quire-contract-model at the same IR sha, and quire-contract-codegen origin/main 4f87c7c"
review_set: base
---
# SR-1152: spec review of PR 276

## Summary

Ticket: IR-495. Spec-only PR. It states the flat v2 wire of merged QSpec FR-322
"Body grammar" as strict wire validation ahead of identity recomputation, removes
the reader depth limit entirely, and adds planned FR-038-AC-114 to AC-118. It also
amends FR-038 AC-3, AC-26, AC-78, AC-86, AC-88 and AC-100, FR-040 prose and
FR-040-AC-10, AD-004 and the two matrices.

Measured at the reviewed sha:

- `make spec`: validate passes, 1 grammar finding (FR-014, baseline), strict
  coverage 23 unbacked rows (baseline). The diff touches only five spec files: no
  code, schema, manifest or test file.
- AC-114/115/116 against merged FR-322 "Body grammar", FR-341-AC-6,
  FR-370-AC-5/AC-12 and FR-440: the operator classes (`quire.op.function.call`,
  `quire.op.state.clause`, `temporal`, `temporal_formula`, `temporal_fairness`),
  the three positions (arguments, aggregate members, binding value), an aggregate
  in a Group's members, a binding as body root, the body-root misplacement
  refused `ill_typed`/`operator-ineligible` at the holding node, the nested `case`
  exception, and the two pre-order pointer examples
  (`.../details/{d}/operator`, `.../details/{d}/members/0/operator`) match the
  merged text. No QSpec text is copied verbatim beyond short pointers.
- The inferred rule, that an application of a class other than
  `temporal_formula`, `temporal_fairness` or `case` in a `details` term is
  `malformed_wire`, follows from merged FR-322. "A diagnostic's `details` are
  each a Member". No Member production is an application. A term outside the
  grammar refuses `malformed_wire` except for the two named exceptions. FR-370
  supports this: it says the three classes are reported "not as `malformed_wire`".
  This is acceptable as IR's requirement and needs no marking.
- The depth limit: FR-038 Inputs, Reading, the limit-charge list, AC-3, AC-26,
  AC-78, AD-004 (seam table row, open question, R-S6) and TC-048's procedure are
  amended. A grep of the whole `spec/` tree finds no other IR spec that names a
  `CheckedPackageReadLimits` depth member, `MAXIMUM_DEPTH`, a `Depth` limit kind
  or a seven-kind list. FR-031's "three limit kinds" are Kani outcome kinds and
  are unrelated. The one exception is the TC-048 cases row in `tests.md`
  (FND-008). The v1 limits of FR-019/FR-023 (`MAX_WIRE_JSON_DEPTH`,
  `MAX_SEMANTIC_DEPTH`) stay, and they collide with AC-117's scan (FND-002).
  quire-contract-codegen builds `CheckedPackageLimit::Depth` in
  `src/exact_scalar.rs:3366` (FND-012).
- AC-116 is consistent with FR-322 "Identity and validation" and FR-038's
  order-of-checks sentence: strict syntax and duplicate member, then canonical
  bytes, then decode and header, then body grammar, then identity. AC-118 fits
  AC-112. AC-112's "a list is absent or empty" fail condition refers to the
  `adverse.json` lists, so an expected-failure list empty of the five mutations
  is coherent.
- FR-040 prose and AC-10 match FR-341-AC-6. The nested clause is marked planned.
  AD-004's settled-STD-125 note is accurate and labels today's behaviour.
- New ACs are direct assertions with no "shall", carry the planned marker, and
  name no tracker state. The matrix range, the TC-048 trace list and the TC-048
  section agree. No crate edge is added. The text says the crate choice is
  undecided, but see FND-007 on "arena order".

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | AC-117's 100000-deep clause cannot be met with the parser FR-038 mandates, and it does not name which stage refuses. FR-038-AC-111 requires the package to be parsed through `serde_json`. Building a `serde_json::Value` recurses once per level: the default recursion limit is 128, and `disable_recursion_limit` needs `serde_stacker` to avoid overflow, which AC-117 bans. The order of checks puts syntax, duplicate-member, canonical-bytes and closed-schema decode ahead of the body grammar, so the grammar refusal "at the first value outside the body grammar" needs the whole 100000-deep document parsed with no call-stack recursion on a 256 KiB thread. A depth pre-scan that refused earlier would break the order of checks. A parser-limit failure is forbidden by "Reading" and carries no pointer under AC-24. The AC also does not require the document to be otherwise canonical and schema-valid, so an earlier refusal satisfies "refuses `malformed_wire`". State how the strict read parses unbounded nesting (an iterative parse), name the refusing stage, and require an otherwise valid canonical document | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2222, 148-167, 2216 |
| FND-002 | high | AC-117 and "Reading" ban things the same crate needs for v1. AC-117 says "the reader's sources and manifests hold no `MAXIMUM_DEPTH` or other `MAX_*DEPTH` constant, no `stacker`, no `serde_stacker`". Reading says "no constant of this repository caps a value a caller supplies by a nesting depth". The reader's crate, `quire-contract-model` (AC-111 calls its manifest "the manifest of the crate that holds the reader"), holds FR-019's `MAX_WIRE_JSON_DEPTH` (576) and `MAX_SEMANTIC_DEPTH` (256) and `MAX_EXPRESSION_DEPTH`. FR-019 and FR-023 keep these limits active ("the existing 576 wire-depth ... limits remain active"). Its manifest declares `stacker` and `serde_stacker`, which v1 `binding.rs`, `identity.rs` and `conformance.rs` use. As written, the AC and the Reading sentence contradict FR-019/FR-023, or force removing v1 limits. Scope both to the V2 reader (`checked_package/`) and say how the manifest clause applies | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2222, 155-157; spec/model/functional/FR-019-rust-library-interface.md:117-119; spec/model/functional/FR-023-executable-projection-binding.md:50-51 |
| FND-003 | medium | AC-117's positive case reads "a package of 100000 flat nodes". Merged FR-322-AC-41 and QSpec TC-427 BG-03 use a 100,000-deep expression (a 100,000-deep sum), which is a chain of nodes each reached by reference. 100000 unrelated nodes do not exercise the reference-chain walks: the closure, the dependency joins and the lowering walk. FR-038's "no walk recurses on the call stack" claim covers those walks. The AC also sizes only the node, edge and byte limits. The occurrence default (100000) and the work default (1000000) may decide the outcome. Use a 100,000-deep expression chain, lower it as well, and size every limit or name the defaults | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2222 |
| FND-004 | medium | The unchanged "Diagnostic details" paragraph contradicts the new flat-wire text. It puts the `details` application refusals in the temporal step, after placement and so after identity recomputation. It also says "An application of class `temporal` there is this reader's reading and not merged text", which reads as `ill_typed`/`operator-ineligible`. The new "The flat wire" section, FR-038-AC-115, the amended path row and AC-100 make that application `malformed_wire` at strict wire validation. The section's blanket "superseded" clause does not name this paragraph. Two implementers would read it differently. Amend the paragraph to the new rule and stage | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1465-1482, 1192 |
| FND-005 | medium | The stage of the nested-`case` refusal is an unmarked IR reading, and no AC pins it. Merged FR-440 "Reader joins" checks case placement, including the nested case, "at FR-322's operation step". FR-322's pre-order rule needs the nested case decided in the same walk as `malformed_wire`. The PR places it at strict wire validation (lines 579-595 and 1667) without saying this resolves a QSpec tension. AC-115 tests pre-order only in `details`, and AC-116 only for a nested non-`case` application. An implementation that keeps the nested case in the operation step passes every AC. Add a node-body case: a nested `case` at `arguments/0` with a nested call at `arguments/1`, refused at `arguments/0/operator`. Add a nested `case` with stale identities, refused `ill_typed` and not `stale-node-key`. Mark the reading | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2220, 579-595, 1667 |
| FND-006 | medium | AC-114 covers a subset of the grammar's violations and omits a case that merged FR-322-AC-40 names: "a binding of an aggregate inside an aggregate's members", that is, a Group member that is a binding whose value is a Group or Tuple. It also has no Tuple-stratum case, such as a binding as a member of a binding-value Tuple. FR-038 states the grammar "without copying" it, so the ACs are the only check that the reader enforces all five strata. Add the FR-322-AC-40 binding case, plus a Tuple-level violation | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2219 |
| FND-007 | medium | "The requirement is an iterative walk in arena order" uses an undefined term. "Arena order" appears nowhere else in the spec tree. The same section and "Reading" say "document pre-order", which decides which refusal is reported. "Arena" is the vocabulary of one particular walker implementation, and the text says it does not choose that dependency. Say "document pre-order" | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:602 |
| FND-008 | low | The TC-048 cases row of the matrix still lists "exact and one-over byte/depth/node/edge/occurrence/diagnostic/work limits". This contradicts amended FR-038-AC-3, which says no limit kind is a depth, and TC-048's amended procedure | spec/checked_package/matrix/tests.md:34 |
| FND-009 | low | AC-100 now ends a clause with "(merged QSpec FR-440 and FR-322, confirmed by the same ruling)". The PR removed the earlier "QSL ruling relayed 2026-10-03" that "the same ruling" referred to, so the reference dangles. The amended row also keeps "QSL confirmed 2026-10-03, QSpec text follow-up pending" for the `case`-node `details` reference, although merged FR-370-AC-12 now states it. Cite the merged text only | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2205 |
| FND-010 | low | Several statements describe the planned state in the present tense with no marker. FR-038 Inputs says the bounded default has "no member a nesting depth". Reading says `CheckedPackageReadLimits` has no depth member and there is "no `stacker`, `serde_stacker` or `on_stack_for`". AD-004's seam row gives the `bounded()` default with no depth. The code still has `depth: 128`, `MAXIMUM_DEPTH` and `on_stack_for`. Only the order-of-checks paragraph and the matrix carry the planned marker. Label these the way the order-of-checks paragraph is labelled | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:42-46, 150-167; spec/assurance/AD-004-checked-package-seam.md:52 |
| FND-011 | low | "Canonical encoding of the wire types" still says "a node `body` and a diagnostic `details` entry nest as deep as their input". The new Reading says every value an admitted package holds "nests to a depth the grammar fixes". Narrow the older sentence to an in-memory `Value` | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:339-341 |
| FND-012 | low | Removing the depth limit removes the `Depth` variant of `CheckedPackageLimit` and the `depth` member of `CheckedPackageReadLimits` (AC-3: "no limit kind is a depth"). Both are public types that codegen consumes: quire-contract-codegen `src/exact_scalar.rs:3366` builds `CheckedPackageLimit::Depth`. FR-038 never names the variant removal, and AD-004 records no breaking change across the seam to codegen | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2111; spec/assurance/AD-004-checked-package-seam.md:52-53 |

## Verdict

Changes needed. The merged-text reading is accurate. AC-114/115/116 say what merged
FR-322, FR-341, FR-370 and FR-440 say. The details-class inference is derived from
FR-322, and the depth removal is complete across FR-038, AD-004 and TC-048. Two
highs block merge, both in AC-117:

- FND-001: state how the strict read parses arbitrarily deep input without
  call-stack recursion while still using serde_json's number parse (AC-111). Name
  the refusing stage, and require an otherwise valid canonical document.
- FND-002: scope the "no `MAX_*DEPTH`, no `stacker`" scan and the "no constant of
  this repository" sentence to the V2 reader, so they do not contradict
  FR-019/FR-023.

The mediums (FND-003 to FND-007) should be fixed in this PR. They are wording or
AC-coverage changes with no code impact.

## New findings (disposition pass 1)

Reviewed at agent-ix/quire-contract-ir@753a3c449843fa11674ff68b0e19eec575edef3a. It is one commit on dd407ae with the same base, e6fc881, which is still main, and it changes four spec files. `make spec`: validate passes, 1 grammar finding (FR-014, baseline), strict coverage 23 unbacked rows (baseline). The delta names no crate and adds no dependency edge. "Arena order" is gone from the spec tree.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-013 | medium | "Reading" now says "this specification fixes no outcome for input nested past the strict parse's own limit", and no AC covers such input. That is weaker than merged FR-322, which refuses a body outside the grammar "as `malformed_wire`, never as a limit". As written, the sentence allows an `incomplete` outcome, a panic or a stack overflow on untrusted input. The removed sentence that the parser's nesting cap "never decides the outcome" leaves nothing in its place. serde_json's default recursion limit of 128 returns an error before it recurses deeper, and every in-grammar package parses well within that limit. So the spec can fix that such input refuses `malformed_wire` (with no pointer, as AC-24 says for malformed JSON), is never `incomplete` and does not overflow a 256 KiB stack, and an AC can check it with a 100000-deep document. If the owner wants this left open, record it as an owner decision rather than leaving it unspecified | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:160-167, 2236 |
| FND-014 | low | The new nested-`case` vectors pin a reading that merged FR-440 contradicts, and the AC rows do not say so. AC-115's case (nested `case` at `arguments/0`, nested call at `arguments/1`, refused at `arguments/0/operator`) and AC-116's case (nested `case` with stale identities, refused `ill_typed`, not at an identity check) follow FR-322's pre-order rule. FR-440 "Reader joins" decides the nested `case` at the operation step, which would give `malformed_wire` at `arguments/1` and an identity refusal respectively. The prose at line 617 marks this as this reader's reading, but neither AC row does, unlike the "(an IR reading)" marking in AC-97 and AC-103. AD-004's QSpec routing table has no row asking QSpec to reconcile the two stages. Mark both rows and add the routing row | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2234-2235, 612-618; spec/assurance/AD-004-checked-package-seam.md:232-238 |

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 753a3c4 |
| FND-002 | fixed | 753a3c4 |
| FND-003 | fixed | 753a3c4 |
| FND-004 | fixed | 753a3c4 |
| FND-005 | fixed | 753a3c4 |
| FND-006 | fixed | 753a3c4 |
| FND-007 | fixed | 753a3c4 |
| FND-008 | fixed | 753a3c4 |
| FND-009 | fixed | 753a3c4 |
| FND-010 | fixed | 753a3c4 |
| FND-011 | fixed | 753a3c4 |
| FND-012 | fixed | 753a3c4 |
| FND-013 | fixed | 54650b5 |
| FND-014 | fixed | 54650b5 |
