---
id: SR-20261103150658
title: gap-analysis of fresh QSL execution-point evidence
type: SpecReview
analysis: gap-analysis
scope: agent-ix/quire-contract-ir; PR 868; Makefile, tests/conformance_qsl_exact_producer/,
  FR-058-AC-1 and FR-058-AC-6
review_set: subset
---

## Summary

Reviewed IR-1003 PR 868, covering the whole FR-058-AC-1 and FR-058-AC-6 criteria and the existing Exact producer lane affected by helper extraction. No findings. Runtime applications: quire-contract-ir / quire-contract-model 0.1.0 and qsl-replay 0.1.0; review tooling quire 0.38.1 with engine 0.51.0 and quoin 0.29.1.

## Verdict

**PASS** for this scoped evidence slice. The lead retains the final aggregate gate. No repository-wide coverage, complete TC-462 qualification, or proof claim is made.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Scope

- examined: FR-058-AC-1 (spec/checked_package/functional/FR-058-admitted-clause-execution-point.md): For one QSL-produced, IR-admitted V2 package with accepted bindings at `Initialization { name: startup }`, `Handler { name: tick }`, `Pre { operation: adjust }`, and `Post { operation: adjust }`, the public accessor returns each exact `ExecutionPoint` variant and nonempty QSpec point name for the selected clause id and authentic claim occurrence. A fresh producer result and the authoritative QSpec carrier are required; a synthetic package alone gives no producer-origin credit.
- examined: FR-058-AC-6 (spec/checked_package/functional/FR-058-admitted-clause-execution-point.md): A public-consumer compile control accepts only the admitted package, clause id and claim occurrence and cannot supply a caller point or QSL binding. An IR reader control consumes the authoritative QSpec node-body member from a fresh QSL package, retains the point after source bytes are dropped, and a source check finds no IR-owned QSL binding copy or alternate compatibility path.
- context_only: FR-352-AC-9 (spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md): A QSL-side selected-model identity producer supplies `SelectedExactIdentities` alongside a checked package; an external caller passes it to the public projection API. The IR join accepts identities only for matching admitted selected-document, declaration form and original IR declaration or field identities. Absent, conflicting, foreign-document and wrong-form entries return the named typed outcome; a caller-supplied bare `NodeKey`, `EffectiveId`, `MemberId`, or node-name pair cannot enter the projection as an authoritative witness.

Examined Makefile's explicit compile-then-native execution and serialization, tests/conformance_qsl_exact_producer/main.rs, the new runtime helper execution_points.rs, and the SourceIdentity constructor migration in producer.rs. Read TC-462 as context for the producer-origin and consumer boundary, without claiming its unrelated procedures complete. Read the actual QSL public compile implementation and IR execution_point_accessor.rs as implementation context. No applicable AssuranceProfile was found in the repository spec tree.

## Whole-criterion alignment

FR-058-AC-1: the runtime helper invokes the genuine public qsl_replay::compile_package_with_execution_points entry with newly authored native source, accepted bindings and the owning QSL model document read by reference. Source inspection confirms that entry traverses QSL checking and emission, rather than fabricating a package. One emitted package is strictly admitted with its selected document evidence. All four authored clauses are independently selected by retained source regions, never by returned point; each selection requires exactly one claim and verifies that occurrence on its retained node. Each accessor result must equal the independently specified exact Initialization(startup), Handler(tick), Pre(adjust), or Post(adjust). The host validates every fresh state-clause body against QSpec's current StateClauseBody schema by reference and checks the exact four carrier values. No external fixture or schema is copied into the repository.

FR-058-AC-6: consume accepts only public package, clause and owner-bound occurrence references and directly calls the public accessor. The fresh compiled bytes, JSON value, caller binding table, selected document, evidence and source string are dropped before repeated reads. Two separate external compiler controls reject the extra caller point (E0061) and QSL binding table (E0308). They require exit 101, exactly one compiler error, the intended binary and source file, the expected code, and a diagnostic naming clause_execution_point. Provisioning or unrelated dependency errors cannot satisfy these checks; the real positive helper succeeds first. The AST source check requires the exact admitted-term accessor input count and call vocabulary. Reviewer inspection confirms the current accessor reads only admitted nodes and typed operation.execution_point(), with no QSL binding copy, JSON decode, export_bindings, supplied fallback or compatibility overload. The AST check is deliberately supplementary to the fresh runtime checks and the explicit source-check requirement.

The shared consumer setup retains the owner lock seed as ephemeral scratch state, patches the producer and consumer to one local model crate, verifies that single model in Cargo metadata, uses explicit offline/locked commands after resolution, and preserves invoking compiler selection. Unique test labels separate scratch trees. The Make recipe completes outer Cargo before native execution and serializes the two native consumers, preventing competing helper Cargo builds against their owned target. The source-identity migration follows the actual current QSL constructor; the existing Exact producer control and its four negative probes pass after extraction. No production API, dependency, repository lock, specification method, workflow, or gate is weakened.

## Coverage

Computed quire matrix independently binds both FR-058-AC-1 and FR-058-AC-6 to fresh_qsl_four_points_survive_source_drop_at_the_public_accessor. Existing qsl_ir1052 bindings also appear; this review's new qualification rests on the freshly exercised helper, not those older tags. Reverse inspection finds no new production behavior or source stubs. The new helper is covered by the explicit native producer lane; it is copied to runtime compiler scratch and is not a separately tagged test function. The candidate report has 78 unbacked rows, 32 retired rows and zero status lies; unrelated strict gaps remain. No comparison to stale local main coverage is used. Specification and criterion population are unchanged by this diff. No optional repository-wide semantic analysis was run; the dispatched whole-criterion alignment above was assessed. Specification review and sub-analyses are inapplicable because no specification or plan changed.

Plan completion: not assessed

## Validation

Inspected resume-current/producer-native-final.log: both native producer tests pass, including the existing Exact control and all four of its compiler probes. Inspected producer-region-selection.log: the new four-point test passes independently. Inspected clippy-final.log and clippy-receipt.log: successful workspace check of the touched conformance target, warnings denied. The runtime helper is compiled and executed by that native test; outer Clippy does not lint generated scratch programs. Independently ran rustfmt --check on all three Rust sources and git diff --check; both pass. Independently computed the matrix with the lead-specified quire executable. No Cargo was launched by this reviewer, as dispatched.

Earlier exploratory producer failures, the source-identity compile failure, and the initial region-selection failure remain in the external receipts. The current source selects a containing authored clause line rather than requiring a region to start at the clause name, and still requires a unique authentic claim. The successful final logs validate the corrected source, rather than treating any earlier failure as positive evidence.
