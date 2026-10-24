---
id: SR-17013
title: "IR-347 PR 557 gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir#557@59d855c0418db8f707c2084d5644dad2b8427b86 vs origin/main 7075edc; spec/kani/matrix/tests.md, spec/kani/matrix/TC-055-root-crate-public-interface.md, spec/kani/functional/FR-039-root-crate-public-interface.md, spec/kani/functional/FR-031-bounded-kani-dispatch-and-terminal-map.md, src/kani/, tests/it/"
review_set: subset
---
# SR-17013: IR-347 PR 557 gap analysis

## Summary

Ticket: IR-347. Plan completion: not assessed. The reviewer ran `make spec` (quire validate plus
`quire coverage --scope . --strict`) at the frozen head.

- Validation reports no error.
- Coverage fails strict on 60 unbacked rows. All of them are outside this diff: FR-052, FR-350,
  FR-057, FR-045, FR-055, FR-056 and similar. The PR's rows read FR-039 4/4, FR-037 1/1,
  FR-031 1/1 and FR-029 3/3.
- 0 contradicted statuses.

Original IR-347 acceptance was checked against the source:

- **Remove arithmetic, collection and graph lowerings and exports: met.** The modules, `pub use`
  lines and suites are deleted. No `lower_*`, `*Lowering` or request type remains under `src/`.
- **Own no independent Kani corpus: met.** No Kani corpus fixture or lowering suite remains. The
  only `tests/fixtures/kani-*` file is the concrete-playback text, and nothing in `src/` or
  `tests/` reads it.
- **Keep bounded three-family dispatch and the shared profile, firewall and outcome: met.**
  `tc_042_dispatch_rejects_cross_family_or_duplicate_ownership` and the TC-055 corpus route
  `DefinednessArithmetic`, `ObjectsReferencesGraphs` and `CollectionsQueries`.
- **TC-055 exact inventory, forbidden module and source checks, negative compile probes: met.**
- **TC-055 typed no-panic handling: partly met.** See FND-001 and SR-17011 FND-001.

Examined units: FR-039-AC-1, FR-039-AC-2, FR-039-AC-3, FR-039-AC-4, FR-037-AC-6, FR-031-AC-1
and TC-055 (all examined).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The matrix marks FR-039-AC-1 through AC-4 ✅ and TC-055 ✅ ("no-panic negative corpus implemented"). FR-039-AC-4 requires no panic path "under the negative corpora of TC-041 and TC-042". The only test bound to AC-4 runs neither corpus. TC-041 has no Kani negative corpus to run (see SR-17014 FND-001), and the TC-042 negative tests are not bound to AC-4 or TC-055. The ✅ status reports more than the bound evidence shows. Without narrowing the AC, either bind the TC-042 negative tests (`kani_shared.rs:92`, `:135`, `:230`, `:336`, `:431`, and the TC-443 malformed-wire case) to FR-039-AC-4 and TC-055, or extend the TC-055 corpus to cover them | spec/kani/matrix/tests.md:17, spec/kani/matrix/tests.md:28 |

## Verdict

**PASS WITH FINDINGS.** The IR-347 deletion scope is complete, and it leaves no production code
without an owning requirement. The one gap is that the ✅ status for FR-039-AC-4 depends on
corpora the bound test does not run.

## Dispositions

Round 1. Checked against fix commit `c28a120bb41f1dea550006fc55b79dd692419bc7`. `make spec` validation is clean. Strict coverage still lists the same 60 unbacked rows, all outside the diff, and 0 contradicted statuses. FR-039 is 4/4.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The FR-039 and TC-055 matrix rows and TC-055 Status now read 🚧 partial. AC-1 through AC-3 are backed. AC-4 has the TC-042 no-panic negatives bound to it, and the unresolved TC-041 reference is named. No overclaim remains (spec/kani/matrix/tests.md:17, :28; spec/kani/matrix/TC-055-root-crate-public-interface.md:46) |
