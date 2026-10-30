---
id: SR-2026102934
title: "Code review — fresh QSL producer readback"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; PR 706 Cargo.toml, Makefile, tests/conformance_qsl_ir1013_ir1052.rs; FR-058-AC-1/6, FR-067-AC-7"
review_set: subset
---

## Summary

Reviewed PR 706 at 48d4e2d3 against the three named criteria and QSL's Switch source. The external tests had a missing source input, and the four-point control asserted a different operation name from FR-058-AC-1.

## Verdict

**FAIL at original review** — the two findings below prevented producer-origin evidence at the reviewed head. Both findings were fixed in later PR revisions; runtime and AC acceptance remain unverified pending QSL #1852.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Both tests required `examples/switch-transition/switch.native.in`, absent from the selected QSL checkout; `producer()` failed before invoking the emitter. QSL owned `switch.native`. | tests/conformance_qsl_ir1013_ir1052.rs:52 |
| FND-002 | high | FR-058-AC-1 required Pre and Post `adjust`, but the producer input and assertions used `set`; a passing test could not establish the literal four-point criterion. | tests/conformance_qsl_ir1013_ir1052.rs:140 |

## Scope

- FR-058-AC-1, examined: For one QSL-produced, IR-admitted V2 package with accepted bindings at `Initialization { name: startup }`, `Handler { name: tick }`, `Pre { operation: adjust }`, and `Post { operation: adjust }`, the public accessor returns each exact `ExecutionPoint` variant and nonempty QSpec point name for the selected clause id and authentic claim occurrence. A fresh producer result and the authoritative QSpec carrier are required; a synthetic package alone gives no producer-origin credit.
- FR-058-AC-6, examined: A public-consumer compile control accepts only the admitted package, clause id and claim occurrence and cannot supply a caller point or QSL binding. An IR reader control consumes the authoritative QSpec node-body member from a fresh QSL package, retains the point after source bytes are dropped, and a source check finds no IR-owned QSL binding copy or alternate compatibility path.
- FR-067-AC-7, examined: IR-owned tests use the authoritative QSpec TC-233 transition package and mutation definitions directly from the selected QSpec checkout, and a QSL-1163-authored source/claim when available; they copy no QSpec or QSL schema, fixture or binary into IR. The QSL-authored positive case exposes pre-state Boolean inputs, post-state Boolean result and actual transition subject through the public IR carrier.

## Review Notes

The test invokes the QSL emitter executable with a fresh source and selected QSL model, then admits returned bytes through the public IR reader. The transition test checks the typed application, Boolean subject, one Boolean input, and Boolean post clause. No copied fixture, frozen package bytes, alternate accessor, or compatibility shim appeared in the original diff. No applicable AssuranceProfile was found. No builds or full gates were run, as directed.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | At a8a51059, both tests read QSL's existing `examples/switch-transition/switch.native`; `producer()` checked that same path. |
| FND-002 | fixed | At a8a51059, the test derived an adjusted Switch model and source at runtime, passed Pre/Post `adjust` to the QSL emitter, and asserted exact Pre/Post `adjust` from the public accessor. |

The new `make conformance-qsl-ir1013-ir1052` target passes both required environment variables to the named Cargo test target; a `make -n` check confirmed the command. Runtime remains unrun. This disposition does not claim AC acceptance.

### Disposition round 2

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | At 2d80b3a4, the four-point test reads QSL-owned `examples/switch-transition/adjust.native` and the transition test reads `switch.native`; neither uses the obsolete `.native.in` path. The `adjust.native` fixture is pending in QSL #1852. |
| FND-002 | fixed | At 2d80b3a4, the harness reads QSL's unchanged selected model, passes QSL-owned `P1` Pre and `Q2` Post `adjust` bindings with appended Init/Handle clauses, and asserts exactly the four required accessor results. The QSL fixture is pending, so runtime and AC acceptance remain unverified. |
