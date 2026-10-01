---
id: SR-643
title: "scope-boundary review of PR 236 (IR-320 subsystem boundaries)"
type: SpecReview
analysis: scope-boundary
scope: "agent-ix/quire-contract-ir@92945cd2e400cdd7940e68db7aab21c3291b7395; spec/spec.md Subsystem Registry, spec/{core,model,conformance,checked_package,output_mapping,kani,bridge}/**, src/lib.rs, src/bin/, src/kani/, crates/quire-contract-model/src/"
review_set: subset
---
# SR-643: scope-boundary review of PR 236

## Summary

Ticket: IR-320. This review checks the seven subsystems against ADR-0056 rules 3 and 4 and against the code at head.

- Core holding `identity` and `limits`: justified. `identity.rs` defines `Diagnostic`, `PackageId`, `Clause`, `Requirement` and `ContractPackage`, which are consumed by wire, binding, conformance and checked_package. Core's in-repo edges to model (FR-012 to FR-014/016/017, STD-001 to FR-013..015/030) are all `references`, not `depends_on`, so rule 4 holds. The IR-320 ticket text put identity and limits in model, but core is the better fit under rule 4. Keep core.
- FR-028 in model: acceptable. It constrains the model crate's cycle-free graph, and the root crate is a consumer of that graph.
- FR-019-AC-5 and TC-058 in model: correct owner.
- No predicate, temporal or ecosystem_model subsystem: correct. That code was deleted (#210), and no live requirement is left there.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The "Bridge" subsystem owns "the root crate excluding `kani` and the conformance binary". At head that is the single line `pub use quire_contract_model::*;` in `src/lib.rs`, which FR-039-AC-1 requires deleting. Its two requirements govern other code: every public item FR-039 lists is a `kani` module item (title "Expose the root crate's bounded-Kani interface"), and FR-037 bounds Kani outcomes. The name revives `src/bridge`, which #210 deleted. Once FR-039 is implemented, this subsystem owns no code | spec/spec.md:118 |
| FND-002 | medium | FR-036 sits in checked_package, whose registry row assigns it to `quire-contract-model::checked_package`. FR-036 says "the provider shall negotiate". It settles items against the `kani-bounded/1` capability matrix (FR-029-AC-2's `unsupported` disposition), and its own Status says the codegen provider negotiates and emits. No requirement places provider negotiation in the target-neutral model crate. 5 of the 17 unbacked ACs, plus TC-045, are parked in a subsystem that will not implement them | spec/spec.md:115 |

## Finding Detail

- FND-001: preferred fix: fold FR-037, FR-039 and TC-055 into `spec/kani/`, and make the Kani row own the whole `quire-contract-ir` root crate except the conformance binary. That matches FR-039's own title and removes a subsystem whose target state is empty. Alternative: rename the subsystem `root` (`spec/root/`). Either way, the change is cheapest in this move-only PR, because doing it later means another restructure.
- FND-002: move FR-036 and TC-045 to `spec/kani/`, next to FR-029's capability matrix. Alternatively, keep FR-036 in checked_package and reword the registry Role so it says only the `ContractPackage` input side is IR's, with negotiation owned by codegen. Confidence is medium. The first option fits FR-036's text better.

## Verdict

The core, model, conformance, output_mapping and kani boundaries are sound. Two medium boundary calls need a decision. Fix both in this PR, or accept each with a stated reason before merging.
