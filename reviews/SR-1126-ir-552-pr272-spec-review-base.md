---
id: SR-1126
title: "spec review of PR 272 (FR-038 merged-text prose after QSpec #181/#182)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir@29cdb1495b4a623e25df532705553da1e03d6112; git diff origin/main...HEAD (base 12ba8ad694511d273621c4bb1cc9b58bee464d55): spec/checked_package/functional/FR-038-consume-checked-package-v2.md prose hunks (refusal-locus table rows, Merged QSpec text, case outcomes, Diagnostic details, union recursion leaf); checked against quire-specification origin/main 396493c4 (FR-322 Structural leaf walk, FR-370, FR-440) and QSpec PRs #181, #182, #183 via gh"
review_set: base
---
# SR-1126: spec review of PR 272, FR-038 prose

## Summary

Ticket: IR-552. The PR replaces "QSL ruling relayed" and "QSpec text follow-up
pending" wording in FR-038 with citations of merged QSpec text, naming PRs #181
and #182.

Measured:

- QSpec #181 (merged 2026-10-03T12:43Z, f39c93f) and #182 (merged
  2026-10-03T14:35Z, e3a3e71) are both merged, as the PR says.
- FR-322 "Structural leaf walk" at QSpec origin/main lists a
  `{path: p + ["recursion:<d>"], laws: [], mode: null}` entry when text is
  reachable. Its examples cover a record (`Node`), a tuple (`Cell`) and a union
  (`Chain`), and over `Option<Chain>` the entry is `recursion:1`. #181 wrote
  "lists no entry for it" and #182's diff replaces that wording with the
  recursion entry. The rewritten paragraph at FR-038 line 1786 is accurate.
- FR-370-AC-12 at origin/main says a `details` reference to a
  `temporal`/`formula`, `temporal`/`fairness` or `expression`/`case` node refuses
  `ill_typed`/`operator-ineligible` at `/diagnostics/entries/{e}/details/{d}`,
  and that `details` references to a union or a union value are admitted. That
  matches FR-038 lines 1396-1402.
- QSpec #183 ("IR-495 rulings: nested temporal applications are
  malformed_wire; pre-order refusal pointer") merged at 2026-10-03T15:26Z,
  before this PR opened at 19:01Z. It changed FR-322, FR-370 and FR-370-AC-12.
  The rewritten prose does not cite it (FND-001).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The rewrite cites "merged QSpec text (#182)" for the case outcomes and the details rules, and "Merged QSpec text" names only #181 and #182. It misses QSpec #183, merged before this PR opened. #183 makes a nested `temporal_formula`, `temporal_fairness` or `temporal` application `malformed_wire` under FR-322 Body grammar. It narrows FR-370's holding-node rule to body-root misplacement and misplaced references. It adds the document pre-order pointer to FR-370-AC-12. The kept parenthetical at line 1381, "the misplaced-application rule of FR-370 refuses at the holding node instead, so the two differ", no longer describes merged FR-370 for nested applications. The paragraph is now labelled merged text, so it reads as agreeing with QSpec when it does not | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1344, 1377-1385 |
| FND-002 | medium | The table row at line 1321 ("an application of the four classes in a `details` term (merged QSpec FR-370, #182)") and the bullet at line 1403 now label as merged QSpec text a rule that covers four operator classes: `temporal`, `temporal_formula`, `temporal_fairness` and `case`. Merged FR-370 (the details placement bullet, "one of the three operator classes above", and FR-370-AC-12) names three: `temporal_formula`, `temporal_fairness` and `case`. A `temporal` class application in `details` was previously labelled a relayed QSL ruling and is still not merged QSpec text, so the provenance is now wrong for that class | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1321, 1396, 1403-1408 |
| FND-003 | low | After the relayed-ruling wording was removed, two phrases still point at "the ruling" with no antecedent: "which the ruling does not name but merged FR-370-AC-12 states" (line 1399) and "this is the ruling's pointer" (line 1381). A reader cannot tell which ruling is meant, or whether it is merged | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1381, 1399 |

## Verdict

Changes needed. The recursion-leaf rewrite and the details-reference rules
check out against QSpec origin/main. Two things need fixing. FND-001: account
for QSpec #183, either by citing it and fixing the stale parenthetical, or by
saying explicitly that the nested-application paragraph is IR-495's to
reconcile. FND-002: keep `temporal` out of the merged-text attribution, or show
where QSpec merged it. Nothing in the diff copies a QSpec file into this
repository.

## Dispositions

Round 1, reviewed at agent-ix/quire-contract-ir@cbcdd758c34ce335b39f4684cf20dff9ca32cd38 (fix commit cbcdd75 on 29cdb14; main 12ba8ad). Checked against QSpec origin/main 396493c4, where #183 and #184 are merged (gh).

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | cbcdd75 |
| FND-002 | fixed | cbcdd75 |
| FND-003 | fixed | cbcdd75 |
