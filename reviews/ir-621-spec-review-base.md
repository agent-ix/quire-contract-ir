---
id: SR-14332
title: spec-review-base review of IR-621 infallible outcome specification
type: SpecReview
analysis: base
scope: agent-ix/quire-contract-ir; spec/core/functional/FR-044-typed-std001-code.md,
  spec/kani/functional/FR-030-bounded-kani-domain-and-outcomes.md, spec/kani/functional/FR-039-root-crate-public-interface.md,
  spec/kani/matrix/TC-457-infallible-non-success-outcomes.md, spec/spec.md
review_set: subset
---

## Summary

Ticket: IR-621. Independent review of PR #487. The new AC ids are unique and method-only Test cells name no TC. TC-457 has a real verifies edge to FR-030, and the specification index links it. Existing requirement stakeholder/dependency edges remain unchanged. The API table names all nine methods, six caller-code signatures and three exact registered constants. AC-4/5 and the live producer portion of AC-7 remain explicitly unclaimed. The negative procedure requires each intended signature to pass and prohibits unrelated import/fixture failures, making compile-fail controls discriminating. Full spec validation completed: 100/100 grammar-clean requirement documents; no grammar finding. No hash/pin or file-tracking contract was introduced.

## Verdict

**PASS** — Specification is ready for its coding stage; no runtime implementation acceptance is asserted.

## Examined Scope

```yaml
scope:
- id: FR-030-AC-8
  path: spec/kani/functional/FR-030-bounded-kani-domain-and-outcomes.md
  role: examined
  excerpt: Test |
- id: FR-030-AC-9
  path: spec/kani/functional/FR-030-bounded-kani-domain-and-outcomes.md
  role: examined
  excerpt: Test |
- id: FR-030-infallible-1
  path: spec/kani/functional/FR-030-bounded-kani-domain-and-outcomes.md
  role: examined
  excerpt: 'The root crate shall expose the following inherent `KaniOutcome` methods
    to

    consumers outside the `kani` module. Every method returns `KaniOutcome`

    directly, with no `Result`, optional outcome, panic for an impossible kind,

    or caller-supplied `KaniOutcomeKind`.'
- id: FR-030-infallible-2
  path: spec/kani/functional/FR-030-bounded-kani-domain-and-outcomes.md
  role: examined
  excerpt: '| Method | Arguments before `source_id`, `context` | Outcome kind | Cause
    |

    | --- | --- | --- | --- |

    | `refused` | `code: Std001Code` | `Refused` | Exact supplied code |

    | `invalid_input` | `code: Std001Code` | `InvalidInput` | Exact supplied code
    |

    | `incomplete_input` | `code: Std001Code` | `IncompleteInput` | Exact supplied
    code |

    | `timed_out` | `code: Std001Code` | `TimedOut` | Exact supplied code |

    | `resource_exhausted` | `code: Std001Code` | `ResourceExhausted` | Exact supplied
    code |

    | `cancelled` | `source: KaniCancellationSource`, `code: Std001Code` | `Cancelled
    { source }` | Exact'
- id: FR-030-infallible-3
  path: spec/kani/functional/FR-030-bounded-kani-domain-and-outcomes.md
  role: examined
  excerpt: 'Each method takes `source_id: impl Into<String>` and

    `context: impl Into<String>` as its final two arguments. The outcome shall

    retain both strings exactly. The six methods taking a code shall retain any

    valid `Std001Code`, including a consumer-owned unregistered code, without

    substitution or a new registry-membership check

    ([FR-044](../../core/functional/FR-044-typed-std001-code.md)). The three

    fixed-cause methods shall take no cause-code argument. No method in this

    route can request `Proved` or `Counterexample`, and every returned outcome

    shall have `boolean_claim() == None`.'
- id: FR-030-infallible-4
  path: spec/kani/functional/FR-030-bounded-kani-domain-and-outcomes.md
  role: examined
  excerpt: '`vacuous_proof` records a caller-observed zero-SUCCESS-check result; it
    does

    not observe checks or classify an arbitrary profile refusal. Likewise,

    `cancelled` records the supplied observed event source and does not infer or

    observe cancellation. The consumer remains responsible for supplying the

    actual event and source/profile/bound context. This API adds neither a live

    Kani producer nor a QSL terminal mapper.'
- id: FR-030-infallible-5
  path: spec/kani/functional/FR-030-bounded-kani-domain-and-outcomes.md
  role: examined
  excerpt: 'The infallible methods shall use the same internal outcome-construction
    path

    as the existing fallible `non_success` method; they introduce no alternate

    wire envelope or compatibility layer. `non_success` remains the fallible

    entry point for callers holding a broad `KaniOutcomeKind`. This addition does

    not relax its existing rejection of `Proved` and `Counterexample`, and it

    does not claim that the planned AC-4/AC-5 constructor and field-validation

    work is implemented. In particular, adding fixed-cause infallible methods

    does not itself make invalid broad `non_success` cause requests refuse.

    '
- id: FR-030-AC-4
  path: spec/kani/functional/FR-030-bounded-kani-domain-and-outcomes.md
  role: context_only
  excerpt: Test (TC-223) |
- id: FR-030-AC-5
  path: spec/kani/functional/FR-030-bounded-kani-domain-and-outcomes.md
  role: context_only
  excerpt: Test (TC-223) |
- id: FR-030-AC-7
  path: spec/kani/functional/FR-030-bounded-kani-domain-and-outcomes.md
  role: context_only
  excerpt: Test (TC-449) |
- id: FR-044-infallible
  path: spec/core/functional/FR-044-typed-std001-code.md
  role: examined
  excerpt: 'The six code-taking public infallible non-success constructors of

    [FR-030](../../kani/functional/FR-030-bounded-kani-domain-and-outcomes.md)

    shall accept and retain this same `Std001Code`, including a consumer-owned

    unregistered code. Their other three fixed-cause constructors select existing

    registered constants and take no code argument. This addition changes neither

    the code type''s form nor its registry or wire contract.


    '
- id: FR-039-infallible
  path: spec/kani/functional/FR-039-root-crate-public-interface.md
  role: examined
  excerpt: 'An item''s own public fields, variants and inherent methods are part of
    the

    item and are not listed separately. This includes the nine public infallible

    non-success constructors on `KaniOutcome` specified by

    [FR-030](./FR-030-bounded-kani-domain-and-outcomes.md), AC-8 and AC-9.

    `NonSuccessKind` remains private; this addition exports no new outcome-kind

    type. These constructors accept typed, already selected inputs rather than

    untrusted wire input, and their signatures return `KaniOutcome` directly.


    '
- id: TC-457-Description1
  path: spec/kani/matrix/TC-457-infallible-non-success-outcomes.md
  role: examined
  excerpt: 'Verify [FR-030](../functional/FR-030-bounded-kani-domain-and-outcomes.md)

    AC-8 and AC-9 from outside the `kani` module. This case covers the new

    infallible route, not the still-planned proof-count, field-privacy and broad

    cause-validation work of TC-223 or downstream codegen adoption.

    '
- id: TC-457-Test-Procedure1
  path: spec/kani/matrix/TC-457-infallible-non-success-outcomes.md
  role: examined
  excerpt: 'Use the public `quire_contract_ir::kani::KaniOutcome` import in an external

    integration test. Assign the direct return from each of `refused`,

    `invalid_input`, `incomplete_input`, `timed_out`, `resource_exhausted`,

    `cancelled`, `checker_unavailable`, `backend_unavailable` and `vacuous_proof`

    to an explicitly typed `KaniOutcome`. For the six code-taking methods use a

    valid consumer-owned unregistered code built through

    `quire_contract_model::std001_code!`. Supply distinct source identities and

    contexts containing the selected profile and bounds. For `cancelled`, run

    both `Requested` and `Deadli'
- id: TC-457-Test-Procedure2
  path: spec/kani/matrix/TC-457-infallible-non-success-outcomes.md
  role: examined
  excerpt: 'Assert each expected kind, exact cause, exact source/context and absent

    Boolean claim before and after JSON serialization/deserialization. Inspect

    the serialized cancellation kind to distinguish the two sources and inspect

    the fixed-cause methods'' serialized codes. For coded outcomes, assert that

    `is_registered()` remains false and the supplied code bytes survive unchanged.'
- id: TC-457-Test-Procedure3
  path: spec/kani/matrix/TC-457-infallible-non-success-outcomes.md
  role: examined
  excerpt: 'Pair passing public-signature examples with compile-fail probes that attempt

    to pass `KaniOutcomeKind::Proved` or `Counterexample` as a kind to this route,

    pass a cause argument to each fixed-cause method, omit `cancelled`''s required

    source, and pass a `&str` or a `String` as a coded method''s cause. Each probe

    must fail because the intended argument type or arity is unavailable, rather

    than an unrelated unresolved import or missing fixture. Keep the existing

    public fallible `non_success` regression controls for `Proved` and

    `Counterexample`, inspecting their exact requested kind/code and error'
- id: TC-457-Expected-Results1
  path: spec/kani/matrix/TC-457-infallible-non-success-outcomes.md
  role: examined
  excerpt: 'All nine methods return `KaniOutcome` directly, with the selected kind
    and

    exact supplied context. The three fixed causes are `kani_checker_absent`,

    `kani_backend_absent` and `kani_vacuous_proof`; the six coded methods retain

    the supplied unregistered code. No returned outcome has a Boolean claim.

    Both observed cancellation sources round-trip exactly. Every negative probe

    fails to compile while its paired intended call compiles. The two broad

    Boolean-kind requests still return `kani_outcome_invalid` without an outcome.

    '
- id: TC-457-Status1
  path: spec/kani/matrix/TC-457-infallible-non-success-outcomes.md
  role: examined
  excerpt: 'Planned under IR-621. No source implementation or tagged test is claimed
    by

    this specification change. The public constructors do not themselves observe

    SUCCESS checks or cancellation events and provide no new Boolean proof or

    runtime-producer evidence.

    '
- id: spec-outcome-index
  path: spec/spec.md
  role: examined
  excerpt: TC-222 verifies the FR-344 refusals and TC-223 the outcome constructors
    and TC-443 the typed code of an outcome. [TC-457](kani/matrix/TC-457-infallible-non-success-outcomes.md)
    specifies the planned external infallible non-success constructor controls.
```

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Plan completion: not assessed

No Cargo, rustfmt, metadata, Clippy, native test or Kani invocation was run. No runtime gate receipt is claimed. No applicable AssuranceProfile or required review-selection profile was found. Optional whole-repository semantic gap analysis was not requested and was not run. Dependency analysis was not separately selected: no prerequisite edge changes; TC-457 adds only a verifies edge. Object review, risk-complexity, security and architecture evaluation are not implicated by this bounded API specification. Criterion-strength/Jev was not selected.
