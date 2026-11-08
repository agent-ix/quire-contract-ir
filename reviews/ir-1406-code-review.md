---
id: SR-2476
title: "Code review of quire-contract-ir PR 947 external native join locus sources"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; PR 947 head 2f73ecc6ca9c1be4de6582d54f083958e5c5c924 diff against origin/main c838196ce: crates/quire-contract-model/src/checked_package/v2/native_join_budget.rs, crates/quire-contract-model/src/checked_package/v2/runtime_type_projection.rs, tests/it/native_join_budget.rs"
review_set: subset
---

## Summary

Ticket: IR-1406. PR: quire-contract-ir#947. Code review with the rust-review
lane folded in. Contract: FR-358 statement (external `NativeJoinLocusSource`
variants) and FR-358-AC-10, with FR-361's locus table and AD-010 as context.

The PR adds `AdapterRow(usize)`, `AuthorityRow`, `PreparationSelection(usize)`,
`PreparationOutput(usize)` and `MonitorAuthority` to the public
`NativeJoinLocusSource` enum (`Clone, Copy, Debug, Eq, PartialEq`, no serde
derive, not `#[non_exhaustive]`, same as the existing variants). Each doc
comment matches the FR-358 statement's meaning: adapter row being converted
into an IR-owned row, preparer charges while building the same-emission
authority row, selection comparison by position in the preparer's comparison
order, owned output item allocated after a native read, and IR's own
authority inspections before occurrence resolution.

`accounted_charge`, `resume`, `first_unpaid` and `first_unpaid_locus` are
source-agnostic: they store and return the caller's `NativeJoinLocus` (a
`Copy` value) as given, so no change is needed there and the new sources are
carried unchanged. The only exhaustive match over the enum is
`SelectedTypeDeclarationsError::denied_node`; the new sources map to `None`.
FR-358 says the external variants "do not change any native read's own loci"
and that the locus is non-owning; none of the new sources indexes the
package graph or the caller's tuple/source-unit rows, so "no node" is the
correct resolution and matches the resolver's rule that it never searches for
an identity after a denied charge. The other uses of the enum in the crate
(nominal_reads, exact_projection, selected_type_closure) construct variants
and do not match on them.

Focused check: `cargo test --features conformance-cli --test it
native_join_budget` at the head (result in Verdict).

## Verdict

**PASS with one low finding.** The type change is correct, minimal and
idiomatic. The `denied_node` doc comment was not extended to mention the new
sources (FND-001).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | `denied_node`'s doc comment still says only "Package allocations and unresolved declaration request coordinates have no resolved node"; it does not say that external (`AdapterRow`, `AuthorityRow`, `PreparationSelection`, `PreparationOutput`) and `MonitorAuthority` loci also resolve to no node, though the match now returns `None` for them | crates/quire-contract-model/src/checked_package/v2/runtime_type_projection.rs:334-337 |
