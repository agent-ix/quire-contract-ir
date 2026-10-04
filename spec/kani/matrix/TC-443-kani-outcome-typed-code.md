---
id: TC-443
title: "A Kani outcome and its error carry a typed STD-001 code"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-030
    type: verifies
---
# TC-443: A Kani outcome and its error carry a typed STD-001 code

## Description

Verify FR-030-AC-6: `KaniOutcome.code` and `KaniOutcomeError.code` are
`Std001Code` (FR-044), the non-success constructor takes one and refuses a
string, and the code serializes as the same JSON string and is checked on read.
It is split from TC-223 so that the strict coverage gate does not count
FR-030-AC-4 and FR-030-AC-5, which have no test, as backed by a test of AC-6.

## Test Procedure

Build the outcome for a zero-check proof, read its `code` and serialize it. Ask
the non-success constructor for a `proved` and for a `counterexample` outcome
and read the error's code and the requested kind and cause it names. Replace the
serialized code with `"Bad-Code"` and deserialize. Compile probes that pass a
`&str` and a `String` as the code of the non-success constructor, each paired
with the call that passes a `Std001Code`.

## Expected Results

The zero-check outcome's code equals `Std001Code::KANI_VACUOUS_PROOF` and its
serialization holds the member `"code":"kani_vacuous_proof"`. Each error's code
equals `Std001Code::KANI_OUTCOME_INVALID` and names the requested kind and cause.
The `"Bad-Code"` outcome fails to deserialize. Both probes fail to compile and
the paired call compiles.

## Status

Partly implemented: `tc_443_the_outcome_and_its_error_carry_a_typed_std001_code`
in `tests/it/kani_shared.rs`, and two `compile_fail` doctests with a passing one
on `KaniOutcome::non_success`, which `make test` runs (`cargo test -p
quire-contract-ir --doc`). FR-030-AC-6's clause that reads the error's code for a
`proved` request with a count of zero is planned with FR-030-AC-4: no such
request exists, so the test reads the error through the non-success request for
a `proved` outcome, and the clause is not counted as met.
