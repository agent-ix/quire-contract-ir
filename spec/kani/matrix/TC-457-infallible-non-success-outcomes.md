---
id: TC-457
title: "External consumers construct non-success Kani outcomes infallibly"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-030
    type: verifies
---
# TC-457: External consumers construct non-success Kani outcomes infallibly

## Description

Verify [FR-030](../functional/FR-030-bounded-kani-domain-and-outcomes.md)
AC-8 and AC-9 from outside the `kani` module. This case covers the new
infallible route, not the still-planned proof-count, field-privacy and broad
cause-validation work of TC-223 or downstream codegen adoption.

## Test Procedure

Use the public `quire_contract_ir::kani::KaniOutcome` import in an external
integration test. Assign the direct return from each of `refused`,
`invalid_input`, `incomplete_input`, `timed_out`, `resource_exhausted`,
`cancelled`, `checker_unavailable`, `backend_unavailable` and `vacuous_proof`
to an explicitly typed `KaniOutcome`. For the six code-taking methods use a
valid consumer-owned unregistered code built through
`quire_contract_model::std001_code!`. Supply distinct source identities and
contexts containing the selected profile and bounds. For `cancelled`, run
both `Requested` and `Deadline`.

Assert each expected kind, exact cause, exact source/context and absent
Boolean claim before and after JSON serialization/deserialization. Inspect
the serialized cancellation kind to distinguish the two sources and inspect
the fixed-cause methods' serialized codes. For coded outcomes, assert that
`is_registered()` remains false and the supplied code bytes survive unchanged.

Pair passing public-signature examples with compile-fail probes that attempt
to pass `KaniOutcomeKind::Proved` or `Counterexample` as a kind to this route,
pass a cause argument to each fixed-cause method, omit `cancelled`'s required
source, and pass a `&str` or a `String` as a coded method's cause. Each probe
must fail because the intended argument type or arity is unavailable, rather
than an unrelated unresolved import or missing fixture. Keep the existing
public fallible `non_success` regression controls for `Proved` and
`Counterexample`, inspecting their exact requested kind/code and error code.

## Expected Results

All nine methods return `KaniOutcome` directly, with the selected kind and
exact supplied context. The three fixed causes are `kani_checker_absent`,
`kani_backend_absent` and `kani_vacuous_proof`; the six coded methods retain
the supplied unregistered code. No returned outcome has a Boolean claim.
Both observed cancellation sources round-trip exactly. Every negative probe
fails to compile while its paired intended call compiles. The two broad
Boolean-kind requests still return `kani_outcome_invalid` without an outcome.

## Status

Implemented by the external integration tests in
`tests/it/kani_outcome_construction.rs` and the paired passing/compile-fail
doctests on the nine public `KaniOutcome` methods. The native tests assert
exact construction and wire behavior; the doctests check type and arity
refusals. The public constructors do not themselves observe SUCCESS checks or
cancellation events and provide no new Boolean proof or runtime-producer
evidence.
