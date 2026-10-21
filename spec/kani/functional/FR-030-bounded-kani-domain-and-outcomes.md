---
id: FR-030
title: "Bind bounded Kani domains and non-Boolean outcomes"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/FR-029
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-015
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-044
    type: depends_on
---
# FR-030: Bind bounded Kani domains and non-Boolean outcomes

## Description

The bounded-Kani boundary shall use a typed finite model-domain/snapshot/population ABI and a common typed outcome envelope. The bounded-Kani boundary shall not hide invalid or incomplete populations through Kani assumptions.

## Inputs

A selected `kani-bounded/1` profile, finite typed model declarations, explicit population and snapshot records, model/domain/collection/graph/resource bounds, and a checked native clause.

## Outputs

Either a validated bounded execution input and a typed outcome, or a typed pre-execution refusal with no partial proof artifact. A construct the profile cannot interpret is settled `unsupported` at negotiation (FR-029) and never reaches this boundary.

## Behavior

The ABI carries the exact model identity, closed typed universes, object identities, ordered duplicate-preserving collection values, reference targets, snapshot and invocation identities, completeness declarations, and every selected bound. Bounds derive only from the selected profile and exact reviewed model/domain declarations; host widths, observed values, and backend defaults are not bounds. The input checker validates complete populations before harness construction: duplicate identities, wrong object types, dangling/foreign references, snapshot/invocation mismatch, impossible creation/deletion deltas, duplicate decoded fields, oversized values, and a claimed-complete but invalid population are `invalid_input`. An incomplete population or unavailable required observation is `incomplete_input`, not an empty universe.

The common envelope has distinct terminal kinds `proved`, `counterexample`, `refused`, `invalid_input`, `incomplete_input`, `unavailable`, `timed_out`, `resource_exhausted`, `cancelled`, and `inconclusive`. Only `proved` carries a Boolean-success claim; only `counterexample` carries a Boolean-failure claim. Every other kind carries a stable code, source/profile identity, bound/resource context, and cause as applicable, and carries no Boolean result. A definedness failure remains the existing typed partial-operation result and cannot become a Kani counterexample or proof by coercion.

A `KaniOutcome`'s kind, cause code, SUCCESS check count, source identity and
context are private fields read through accessors, and an outcome is built
only through its validated constructors, so no struct literal outside the
`kani` module can build one. The constructors enforce these kind and cause
rules:

- An `unavailable` outcome carries exactly one of two cause codes:
  `kani_checker_absent`, when the run finds no checker satisfying the selected
  profile's capability negotiation, or `kani_backend_absent`, when the run
  finds no Kani backend satisfying it.
- An `inconclusive` outcome carries exactly the cause code
  `kani_vacuous_proof`.
- A `proved` outcome carries a SUCCESS check count of at least one.

The `cancelled` kind carries one required `KaniCancellationSource` value:
`Requested` when the caller-owned cancellation handle records a caller request,
or `Deadline` when the frontend that owns the timer cancels that handle at its
deadline. Contract IR owns this typed outcome carrier and its validation.
Codegen's bounded Kani launcher/execution boundary (`src/kani/run/launch.rs`
and `src/kani/run/execute.rs`) owns observing any cancellation of a Kani run:
it must receive source-bearing control from the caller or timer owner, retain
the first event that actually stopped the run, and pass that observed source
when constructing an IR cancelled outcome. Codegen currently has no such
event reader or `Cancelled` outcome producer; its launcher reports an elapsed
execution ceiling as `TimedOut`. Adding the source-bearing producer and its
QSL terminal map is a prerequisite of IR-701, not an existing behavior or an
IR model dependency. Until that producer exists, IR-701 cannot claim a
requested/deadline cancellation mapping from a live Kani run. Neither
Contract IR nor a consumer derives the origin from `code`, `source_id`,
`context`, elapsed time, a generic timeout, or a failed charge. The source is
part of the `Cancelled` variant of `KaniOutcomeKind`, so every cancelled
outcome has exactly one source and no other kind has one. `TimedOut` remains a
distinct outcome for a run that exceeded its execution timeout without a
recorded cancellation event. The type and outcome live in the root crate's
`kani` module without a dependency on QSL or quire-exact; codegen translates
the two source variants into QSL's `CancelCause` at its FR-030 consumer seam.

On serialization, a cancelled outcome's `kind` is
`{"cancelled":{"source":"requested"}}` or
`{"cancelled":{"source":"deadline"}}`; all other kind spellings and outcome
members retain their existing form. Deserialization refuses the old unit
`"cancelled"`, a missing or unfamiliar cancellation source, and a source
member on a non-cancelled outcome (for example,
`{"kind":"timed_out","source":"deadline"}`). The outcome reader rejects
unknown members. There is no default origin or alternate legacy wire form.

A `KaniOutcome`'s code, and a `KaniOutcomeError`'s, is a `Std001Code`
(FR-044), exported by `quire-contract-model`, and not a `String`. The
non-success constructor takes a `Std001Code`, and a `proved` or `counterexample`
outcome carries `kani_proved` or `kani_counterexample`. The codes IR raises are
registered in STD-001 ("Bounded Kani Cause Codes") and spelled by `Std001Code`
constants, so a consumer matches `Std001Code::KANI_CHECKER_ABSENT` and not a
literal. The outcome serializes its code as the same JSON string it serialized
before. A consumer that builds an outcome with a code of its own, as
codegen's corpus refusals do, builds that code with `std001_code!`; the type
checks its form and not its registration or issuer (FR-044), so `code` alone does
not say which registry issued a code, and `is_registered()` says only whether
STD-001 registers it. This replaces the free-string cause
code: it is a prerelease break with no compatibility layer, and an outcome
deserialized with a malformed code is refused.

A constructor asked to build an outcome that breaks one of these rules, or to
build `proved` or `counterexample` through the non-success constructor,
returns a typed `KaniOutcomeError` with the code `kani_outcome_invalid`,
registered in STD-001, and no outcome; it never substitutes another kind or cause.

### Public infallible non-success constructors

The root crate shall expose the following inherent `KaniOutcome` methods to
consumers outside the `kani` module. Every method returns `KaniOutcome`
directly, with no `Result`, optional outcome, panic for an impossible kind,
or caller-supplied `KaniOutcomeKind`.

| Method | Arguments before `source_id`, `context` | Outcome kind | Cause |
| --- | --- | --- | --- |
| `refused` | `code: Std001Code` | `Refused` | Exact supplied code |
| `invalid_input` | `code: Std001Code` | `InvalidInput` | Exact supplied code |
| `incomplete_input` | `code: Std001Code` | `IncompleteInput` | Exact supplied code |
| `timed_out` | `code: Std001Code` | `TimedOut` | Exact supplied code |
| `resource_exhausted` | `code: Std001Code` | `ResourceExhausted` | Exact supplied code |
| `cancelled` | `source: KaniCancellationSource`, `code: Std001Code` | `Cancelled { source }` | Exact supplied code |
| `checker_unavailable` | None | `Unavailable` | `Std001Code::KANI_CHECKER_ABSENT` |
| `backend_unavailable` | None | `Unavailable` | `Std001Code::KANI_BACKEND_ABSENT` |
| `vacuous_proof` | None | `Inconclusive` | `Std001Code::KANI_VACUOUS_PROOF` |

Each method takes `source_id: impl Into<String>` and
`context: impl Into<String>` as its final two arguments. The outcome shall
retain both strings exactly. The six methods taking a code shall retain any
valid `Std001Code`, including a consumer-owned unregistered code, without
substitution or a new registry-membership check
([FR-044](../../core/functional/FR-044-typed-std001-code.md)). The three
fixed-cause methods shall take no cause-code argument. No method in this
route can request `Proved` or `Counterexample`, and every returned outcome
shall have `boolean_claim() == None`.

`vacuous_proof` records a caller-observed zero-SUCCESS-check result; it does
not observe checks or classify an arbitrary profile refusal. Likewise,
`cancelled` records the supplied observed event source and does not infer or
observe cancellation. The consumer remains responsible for supplying the
actual event and source/profile/bound context. This API adds neither a live
Kani producer nor a QSL terminal mapper.

The infallible methods shall use the same internal outcome-construction path
as the existing fallible `non_success` method; they introduce no alternate
wire envelope or compatibility layer. `non_success` remains the fallible
entry point for callers holding a broad `KaniOutcomeKind`. This addition does
not relax its existing rejection of `Proved` and `Counterexample`, and it
does not claim that the planned AC-4/AC-5 constructor and field-validation
work is implemented. In particular, adding fixed-cause infallible methods
does not itself make invalid broad `non_success` cause requests refuse.

A `proved` outcome carries the number of SUCCESS checks the run's obligation completed with, and that number is at least one. When a Kani run reports a proof whose obligation completed with zero SUCCESS checks, the bounded-Kani boundary shall record `inconclusive` with the cause code `kani_vacuous_proof` and no Boolean claim, never `proved`; that classification is the one shared rule every caller that classifies a Kani run routes through.

The harness constrains only values after the ABI has established that the finite population and relationships are valid. It shall never use `assume` to discard invalid, incomplete, unavailable, or over-bound input; those inputs return their earliest typed result with originating source identity and no partial lowering, oracle, proof, or replay artifact.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-030-AC-1 | Every model, population, snapshot, identity, reference, collection, and resource bound is explicit, exact, and validated before harness construction. | Test (TC-042) |
| FR-030-AC-2 | Invalid and incomplete populations, unavailable observations, and resource limits retain distinct typed outcomes and are never removed by an assumption. | Test (TC-042) |
| FR-030-AC-3 | No invalid, refused, incomplete, unavailable, timed-out, exhausted, cancelled, or inconclusive outcome carries Boolean success; earliest failures emit no partial artifact. | Test (TC-042) |
| FR-030-AC-4 | A `proved` outcome built from a run with three SUCCESS checks carries the count three; a run with zero SUCCESS checks yields `inconclusive` with cause `kani_vacuous_proof`, carries no count and no Boolean claim; and a request for a `proved` outcome with a count of zero returns `KaniOutcomeError` with code `kani_outcome_invalid` and no outcome. | Test (TC-223) |
| FR-030-AC-5 | An `unavailable` outcome built for an absent checker carries cause `kani_checker_absent` and one built for an absent backend carries `kani_backend_absent`, each with no Boolean claim; a request for an `unavailable` outcome with any other cause code, an `inconclusive` outcome with a cause other than `kani_vacuous_proof`, or a `proved` or `counterexample` outcome through the non-success constructor returns `KaniOutcomeError` with code `kani_outcome_invalid` and no outcome; and a struct-literal `KaniOutcome` outside the `kani` module fails to compile. | Test (TC-223) |
| FR-030-AC-6 | The `code` of the outcome built for a zero-check proof equals `Std001Code::KANI_VACUOUS_PROOF` and its serialization holds the member `"code":"kani_vacuous_proof"`; the `code` of the `KaniOutcomeError` for a `proved` outcome with a count of zero equals `Std001Code::KANI_OUTCOME_INVALID`; a call that passes a `&str` or a `String` as the code of the non-success constructor fails to compile; and a serialized `KaniOutcome` whose `code` is `"Bad-Code"` fails to deserialize. | Test (TC-443) |
| FR-030-AC-7 | The IR outcome constructor accepts the observed caller-request or timer-owner deadline source as `Cancelled { source: Requested }` or `Cancelled { source: Deadline }`; each serializes and deserializes with that exact `kind` source and no Boolean claim. Construction without a source fails to compile. Deserialization refuses a unit `"cancelled"` kind, a missing or unknown source, and an origin on a non-cancelled kind. Every existing non-cancelled kind retains its former wire spelling. The future CG Kani producer reads the event that actually stopped the run; an execution timeout without that event remains `TimedOut`, and reversing the recorded event reverses the constructed cancellation source. | Test (TC-449) |
| FR-030-AC-8 | An external consumer calls each of the nine public infallible methods with an explicit `KaniOutcome` return type. The six coded methods retain a consumer-owned unregistered `Std001Code`; `checker_unavailable`, `backend_unavailable` and `vacuous_proof` carry their exact fixed causes. Every outcome retains distinct supplied source/context strings, has the specified kind and no Boolean claim, and survives a serialization/deserialization round trip with exactly the same kind, code, source and context. Both cancellation sources remain distinct in the wire form. | Test |
| FR-030-AC-9 | External compile-fail probes cannot pass `KaniOutcomeKind::Proved` or `Counterexample` as a kind to the infallible route, cannot supply a cause argument to its three fixed-cause methods, cannot omit the cancellation source, and cannot pass a `&str` or `String` as a coded method's cause. Each probe is paired with a passing public call of the intended signature. Existing fallible `non_success` requests for `Proved` and `Counterexample` still return `KaniOutcomeError` naming the exact requested kind and cause and build no outcome. | Test |

## Dependencies

FR-044 defines `Std001Code`. FR-015 defines partial-operation semantics. FR-029 selects the capability and ABI revisions that interpret this boundary.

## Status

Implemented through the shared finite ABI/outcome foundation and three semantic
lanes. Invalid, incomplete, unavailable and over-bound inputs retain typed
non-Boolean outcomes; this status does not claim an undeclared model or domain.
AC-6 is partly implemented (TC-443, IR-605): `KaniOutcome.code` is a `Std001Code`
that serializes as the bare string and is checked on read, `non_success` takes
one (a string does not compile), and it returns `KaniOutcomeError` (code
`kani_outcome_invalid`, naming the requested kind and cause) for a `proved` or
`counterexample` request. Its clause that reads the error's code for a `proved`
request with a count of zero is planned with AC-4, because no such request
exists; the TC-443 test reads the error through the non-success request instead.
AC-4 and AC-5 are planned (TC-223, no test): `KaniOutcome`'s fields are public, a
`proved` outcome carries no check count and no `proved` request takes one, and
`non_success` accepts any cause code for `unavailable` and `inconclusive`, so the
`CapabilityDisposition::Inconclusive` arm of the lowerings, which carries the
profile's own code, still builds an `inconclusive` outcome that AC-5 will forbid.
AC-7 remains incomplete: the IR carrier has a required typed cancellation
source, exact wire form and refusal controls tagged to TC-449. The CG Kani
launcher has no source-bearing cancellation event reader or IR
cancelled-outcome producer; IR-701 owns the live-event and reversal control.

AC-8 and AC-9 are planned under IR-621; [TC-457](../matrix/TC-457-infallible-non-success-outcomes.md)
defines their public-consumer controls. The existing private `NonSuccessKind`
and `raise` do not yet provide this public route. IR-621 does not claim AC-4,
AC-5, the live cancellation producer in AC-7, or downstream codegen adoption.
