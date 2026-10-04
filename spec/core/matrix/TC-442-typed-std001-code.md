---
id: TC-442
title: "Std001Code is built only by validation, serializes as the bare string and spells every registered code once"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-044
    type: verifies
---
# TC-442: Std001Code is built only by validation, serializes as the bare string and spells every registered code once

## Description

Verify FR-044-AC-1 through FR-044-AC-3: the form `Std001Code::new` accepts and
refuses, the bare-string wire form and its validating deserialization, and the
registered code set. The `KaniOutcome` side of the type is FR-030-AC-6, verified
under TC-223.

## Test Procedure

Call `Std001Code::new` with a registered code, a 64-byte code, and each of the
empty string, a 65-byte string, a string with an upper-case letter, a hyphen, a
leading digit, a leading or trailing underscore, a doubled underscore, a space,
a non-ASCII letter and a one-mebibyte string, and read each result. Serialize
`Std001Code::KANI_VACUOUS_PROOF` and compare the bytes to the quoted code.
Deserialize the quoted code, `"Bad"`, `""`, `7` and `null`. Compare
`Std001Code::REGISTERED` to the fifteen codes FR-044-AC-3 lists, written out in
the test, compare each constant's text to its lower-case name, convert every
`DiagnosticCode::ALL` entry, and call `is_registered` on a registered code, a
diagnostic code and `kani_corpus_identity_collision`.

## Expected Results

The two valid inputs return a code equal to the input. Every other input
returns `Std001CodeError` with code `invalid_code_form` and no value, with no
panic. The serialization is the quoted code, the valid string deserializes to an
equal code, and the four other values fail with no value. `REGISTERED` equals the
fifteen codes in order, every constant matches, every diagnostic code converts
unchanged, and `is_registered` is true for the first two and false for the
third.

## Status

Planned. No test is tagged for this case and no `code` module exists.
