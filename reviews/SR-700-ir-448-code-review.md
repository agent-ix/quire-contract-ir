---
id: SR-700
title: "code review of PR 244 (IR-448 FR-154 doc comments)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir@e94752093bd5ed619945964b9a6a035e48af5597; crates/quire-contract-model/src/checked_package/v2/model_members/tests.rs"
review_set: subset
---
# SR-700: code review of PR 244

## Summary

Ticket: IR-448. Reviewed head e947520 against origin/main 968ba9b. The Rust part of the
diff changes three doc comments in `model_members/tests.rs`. Each comment opened with
`FR-154:`. The trace scan read that as a trace to a row this repo does not declare, and
printed three "`FR-154` matches no declared row" warnings. The change touches comments only,
so no test changes behaviour. Before and after `make spec`, the three warnings are gone and
the 17 unbacked rows are unchanged. `make fmt-check` passes. The `Tracing: TC-048,
FR-038-AC-28` lines are unchanged.

The new wording says FR-154 is a QSL requirement. That is false. FR-154 ("Admit the domain
model from a domain package") is in agent-ix/quire-specification, at
`spec/functional/type-model/FR-154-admit-domain-package-model.md`. This repo's own FR-344 has
the relationship target `ix://agent-ix/quire-specification/FR-154`. The repo's idiom for that
spec is `QSpec FR-NNN`, as in `v2/frame.rs:1` and `v2/vocabulary.rs:366`. QSL means
quire-spec-language, a different repository. I tested `/// QSpec FR-154: ...` in a scratch
copy of the head. Quire did not read it as a trace (no FR-154 warning), and the count stayed
at 17 unbacked rows.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The three reworded doc comments attribute FR-154 to QSL ("In QSL's FR-154 (a QSL requirement, not a row of this repo)"). FR-154 is a quire-specification (QSpec) requirement, as FR-344's relationship `ix://agent-ix/quire-specification/FR-154` shows. The citation now points readers to the wrong repository. Use the repo idiom `/// QSpec FR-154: ...`. A scratch probe shows the scan does not read that form as a trace | crates/quire-contract-model/src/checked_package/v2/model_members/tests.rs:258,294,374 |
| FND-002 | low | Each reworded first line is now 123 to 128 characters long, over the repo's `max_width = 100` (rustfmt.toml). rustfmt does not wrap comments, so fmt-check passes anyway. Reflow the paragraphs. With the FND-001 wording the lines are short enough | crates/quire-contract-model/src/checked_package/v2/model_members/tests.rs:258,294,374 |

## Verdict

Not mergeable as is, because of FND-001. The comments are behaviour-neutral, but the guardrail
asks for truthful wording, and the new attribution is false. The fix is to change three lines
to `QSpec FR-154:` and reflow them.

## Dispositions

Round 1, reviewed at 4941351784736182f05385b670f7860889f04cdf (fix commit 4941351).

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 4941351 |
| FND-002 | fixed | 4941351 |
