---
id: SR-1208
title: "PR #281 FR-346 spec text review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir@98ed90df11a41f8c76aa1b4146b31916d56b4bf7; spec/checked_package/functional/FR-346-admit-abstraction-relation-body.md, spec/checked_package/matrix/TC-225-checked-package-v2-abstraction-relation-body.md, spec/checked_package/matrix/tests.md, spec/tests.md (git diff origin/main...HEAD)"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-346
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/TC-225
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/FR-040
    type: references
---
# SR-1208: PR #281 FR-346 spec text review

## Summary

Ticket: IR-509. PR agent-ix/quire-contract-ir#281.

The spec diff in this code PR:

- removes 🚧 from FR-346-AC-1 through AC-10;
- appends to FR-346-AC-2 "at `/semantic_graph/nodes/{n}/body` as FR-040-AC-10
  has it" for the state-step body-root refusal;
- rewrites the Body shape prose from "with the same code and locus" to "with
  the same code and cause", adding which pointer each step uses;
- updates the TC-225 Status, the FR-346 and TC-225 matrix rows and the
  `spec/tests.md` index row.

Read against FR-040-AC-10, the FR-038 "Frame bodies" and flat-wire sections,
and merged QSpec FR-451, FR-450, FR-342 and FR-353 (quire-specification
origin/main, read-only).

## Verdict

**APPROVE WITH NITS** (two low findings).

The AC-2 pointer change is correct. FR-040-AC-10 puts a clause application at
the root of another node at the holding node. The existing TC-056 test
`tc_056_a_state_clause_application_stands_only_as_a_clause_body_root` and
`state.rs` `validate_placement` already refuse at `/semantic_graph/nodes/{n}/body`.
Merged FR-346-AC-2 therefore contradicted shipped, tested FR-040 behaviour,
and the amendment removes the contradiction without changing any reader
behaviour. The matrix status flips are backed (see SR-1207). `make spec`
grammar is 1 finding (FR-014), as at base.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | A merged acceptance criterion (FR-346-AC-2) and its Body shape prose are amended inside a code PR. The amendment is right, but under spec-first it lands without its own spec review, and the PR body records it only as "one text clarification". It should be accepted explicitly, or split into a spec PR. | spec/checked_package/functional/FR-346-admit-abstraction-relation-body.md:116-122,257 |
| FND-002 | low | The member step says "its locus is the entry's path (IR reading)". A locus in this reader is a node key, not a path. The code and tests use the entry's `type`/`population`/`context` node key as the locus of every member refusal (abstraction.rs module doc), and FR-346 never states that IR reading. The target and uniqueness steps do state their node-key loci. | spec/checked_package/functional/FR-346-admit-abstraction-relation-body.md:200-203 |

## Dispositions

Round 1 was reviewed at 0768afb609da09d3a82043b1fd63a1780b0a6afa.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 0768afb. The PR body now has a "Spec corrections in this PR" section that lists four FR-346 text changes for review: the AC-2 state-step `/body`, `digest_domain_mismatch` at the key, the member-step locus, and the declared-parameter-name source. The FR-346 diff from origin/main matches that list plus the 🚧 status glyph removals. There is no unlisted text change. All four are reviewed here and hold. |
| FND-002 | fixed | 0768afb. Step 4 now says: "its path is that entry's. IR reading: its locus is the node key of the entry's `type`, `population` or `context` target, as the target and uniqueness steps have it." |
