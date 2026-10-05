---
id: TC-228
title: "CheckedPackage V2 re-derives the keys of the gated forms and reads QSpec's regenerated fixtures"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: verifies
---
# TC-228: CheckedPackage V2 re-derives the keys of the gated forms and reads QSpec's regenerated fixtures

## Description

Verify FR-038-AC-131, FR-038-AC-132, FR-038-AC-133, FR-038-AC-135 and
FR-038-AC-150 (IR-627): the criteria of the anonymous structural node key rule
(TC-226) that no test can discharge yet. AC-131 and AC-132 (the forms with no
preimage the reader derives, and declared nodes) are gated on IR-627-Q1 to Q4,
and AC-150 (in-group re-derivation) on IR-630 and QSL-638; AC-133 and AC-135
read QSpec's three positive fixtures, `adverse.json` and
`dependency-selection-vectors.json` carrying derived keys, which QSpec owns
(IR-627-Q5, QSL-635).

## Test Procedure

AC-131: for each gated form, change one body value of a node and keep its
`node_id`; the reader refuses `invalid_package`/`stale-node-key` at the node.
AC-132: the vectors or types the reader is checked against come from the
source IR-627-Q2 names and are never copied into this repository; the test
fails closed when the source is absent. AC-133 and AC-135: read the QSpec
fixtures from the checkout `QUIRE_SPECIFICATION_DIR` names, never copied, with
derived keys and with one placeholder key restored on an undeclared
derived-shape node, and apply `adverse.json` over the regenerated base.

## Expected Results

Each gated-form mutation refuses `invalid_package`/`stale-node-key` at the
node. The three QSpec fixtures admit with derived keys and refuse with a
placeholder; every `adverse.json` entry reaches its recorded outcome and none
is refused `stale-node-key`.

## Status

Planned. No test is written. AC-131 and AC-132 await the answers to IR-627-Q1
to Q4; AC-133 and AC-135 await QSpec's regenerated fixtures (IR-627-Q5,
QSL-635).
