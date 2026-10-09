---
id: TC-444
title: "Private numeric type and range are paired by construction"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-014
    type: verifies
---
# TC-444: Private numeric type and range are paired by construction

## Description

Inspect the private expression checker representation for FR-014-AC-9. Each
successfully checked integer or rational operand must carry its concrete type
and same-kind interval range as one typed value.

## Test Procedure

Inventory every private constructor and mutation of checked numeric operands
in `crates/quire-contract-model/src/expression.rs`, including literal,
reference, access, arithmetic, negation, range-refinement, and shared child
construction paths. Inspect the types accepted and returned by the unary and
binary numeric operator dispatch and checker functions. For each integer and
rational path, determine whether Rust's type checker rejects construction or
passage of a numeric operand with an absent range or the other numeric kind's
range. Compare the public expression signatures and invalid-operand diagnostic
codes and spans with FR-014-AC-1.

## Expected Results

The private integer case bundles `IntegerType` with integer interval ranges,
and the private rational case bundles `RationalType` with rational interval
ranges. Every checked-numeric constructor and unary and binary operator
signature consumes or produces those paired cases. Inspection fails if a
numeric type and an independent optional or cross-kind range can be assembled
or passed in well-typed Rust, even if a runtime match later refuses it. The
public signatures and invalid-operand diagnostics remain unchanged.

## Status

Planned.
