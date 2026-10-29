---
id: SR-15600
title: "IR-9 PR 514 string-edge consumer specification review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; IR-9; spec/core/functional/FR-056-consume-authoritative-string-edge-scan.md; spec/core/matrix/TC-460-string-edge-consumer-gate.md; spec/spec.md"
review_set: base
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-056
    type: references
  - target: ix://agent-ix/quire-contract-ir/TC-460
    type: references
---

## Summary

Independent non-author review of PR #514 at exact against IR main. The new FR and TC correctly distinguish controlled fixture checks from the required clean result on both real source roots, preserve the separate FR-028 dependency boundary, require the neutral ix-trace marker and authoritative QSL command, and state the scanner's syntactic limits. The core matrix and index have not been extended to declare the new FR/TC and their planned coverage.

## Scope

- FR-056 and all eight criteria: examined for intake boundary, marker authority, strict rules, failure propagation and actual-source completion.
- TC-460: examined for real-command mutation, positive/negative controls and the distinction between fixture success and repository success.
- `spec/spec.md`: examined for registry links.
- `spec/core/matrix/tests.md` and `spec/tests.md`: context only, inspected for the new requirement/test mapping.
- QSL origin/main FR-064 and `qsl-string-edge` command/error implementation; ix-trace-rs origin/main FR-004: context only, to verify the claimed external contracts.

## Verdict

**CONDITIONAL** — The consumer boundary and adverse controls are sound, but the declared matrix mapping is incomplete. The 236-hit red source inventory remains an implementation blocker for FR-056-AC-8, not a defect in this honest planned-status specification. No Rust/code review or execution gate applies to this Markdown-only PR.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Add FR-056, its AC-1 through AC-8 mapping to TC-460, and planned/unrun status to the core test matrix; add the core subsystem's FR-056 entry to the top-level matrix index. The core StR-003 row currently omits FR-056 despite the new `traces_to` relationship. A registry link in `spec/spec.md` alone does not supply the repository's declared matrix coverage. | spec/core/matrix/tests.md:14; spec/core/matrix/tests.md:19; spec/core/matrix/tests.md:40; spec/tests.md:12; FR-056 |

## Review notes

Changed-document `quire validate --scope.` passed 3/3 grammar-clean at the exact reviewed head; `git diff --check` passed. The computed `quire matrix --format tsv` reports FR-056-AC-1 through AC-6 and AC-8 as `untagged` and AC-7 as `method-without-symbol`, appropriate for a planned spec before implementation, but it does not replace the missing human-declared core matrix rows. QSL's current command source maps clean/violation/error to exits 0/1/2 and distinguishes a missing executable from scanner-returned exit 2; FR-056-AC-5 reflects that distinction. The existing measured red scan was reported by the author on IR-9 and was not rerun by this reviewer. No Cargo tests, `make ci`, Kani, replay or mutation gates were run.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | At exact, `spec/core/matrix/tests.md` adds FR-056-AC-1 through AC-8 mapped to TC-460 with planned/unrun status, adds TC-460 and the StR-003 FR-056 edge; `spec/tests.md` adds FR-056 to the core index and keeps actual-source-clean credit planned. The original SR-15600 copied into `reviews/` is byte-identical to this reviewer-held artifact before the disposition appendix. Changed-document Quire validation passes 6/6; diff-check passes. The red real-source inventory remains an implementation condition, not a spec finding. |
