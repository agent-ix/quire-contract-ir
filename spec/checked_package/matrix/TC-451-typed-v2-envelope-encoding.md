---
id: TC-451
title: "Typed checked-package V2 envelope encoding"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-051
    type: verifies
  - target: ix://agent-ix/quire-contract-ir/FR-019
    type: verifies
---
# TC-451: Typed checked-package V2 envelope encoding

## Description

Verify FR-051-AC-1 through FR-051-AC-4 through the public model-crate interface, canonical encoder and strict V2 reader. These controls are planned for the structural-identity cutover. QSL producer adoption remains separate.

## Test Procedure

Compile an external default-feature consumer that supplies a non-empty package name, typed lock and graph, source correspondence, capability report and diagnostics to `encode_checked_package_v2` with explicit `quire_canonical::Limits`. Compare its output with the canonical encoding of IR's private wire in a unit test. Inspect the seven required top-level members and the optional `refinement_step_rows` member when supplied. Compile negative probes importing the private wire from the model crate and the encoder from the root crate; check that an earlier signature taking a package id or identity preimage does not compile.

Encode with the exact canonical byte limit and one byte below it; then supply an empty name and a canonical-invalid typed value. Every failing call returns a typed error without bytes. Read the valid bytes through `CheckedPackageV2::read` with required evidence and compare the admitted name, structural node ids, full bodies and node order. Remove a required top-level member, add an unknown member and duplicate one member independently. Each mutation returns its matching typed refusal and no admitted package.

Inspect the private generic envelope declaration: one top-level field list serves owned read, raw-term intake and typed borrowed encode. Fixed-depth members use `Writer::serialize`; arbitrary-depth terms use iterative `Encode`. Encode empty, nested and deeply nested bodies and a deep diagnostic detail on a small stack. Compare the public encoder's complete bytes with the private wire at equal limits, including present and absent refinement-step rows and optional node fields. Find no cloned body, projection, identity derivation, admission, second envelope schema or producer byte-limit constant. Compare default-feature exports with FR-019's public inventory. The integrated producer RSS reduction remains PLANNED under NFR-006 and TC-455 until a QSL candidate is measured.

## Expected Results

The output is the exact closed FR-322 V2 envelope. The compile probes enforce the intended public boundary, the reader admits valid structural identities, malformed mutations return exact typed refusals, and canonical failures return no bytes. QSL producer adoption and integrated memory evidence remain separate.
