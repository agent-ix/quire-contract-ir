---
id: FR-043
title: "Hold one lock entry per first-party crate, checked by one owned tool"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/FR-028
    type: references
  - target: ix://agent-ix/quire-contract-ir/AD-007
    type: references
---
# FR-043: Hold one lock entry per first-party crate, checked by one owned tool

## Description

The repository's `Cargo.lock` SHALL hold at most one entry for each first-party crate, where
a first-party crate is a lock entry whose source is under the agent-ix GitHub organisation
or whose name another entry of that source carries.

`make deny` SHALL fail when the lock holds a second entry for a first-party crate, whether
the second entry differs in version, in git revision or in source kind (git or registry).

`make deny` SHALL run that check through one owned tool.

This repository SHALL NOT carry a copy of the check's logic.

This is AD-007's invariant G-4 for IR, following the owner's ruling on AD-007 O-4
(first-hand, 2026-10-04, recorded on IR-581: replace the three copies with one owned tool).
Third-party crates are outside the requirement: IR's lock holds third-party duplicates today,
and the check does not refuse them.

## Inputs

The repository's `Cargo.lock` and `deny.toml`.

## Outputs

A pass, or a failure that names the crate and the number of lock entries it holds.

## Behavior

Two entries of one first-party crate build the crate twice with types that do not unify;
AD-007 G-3 (every first-party git edge is `branch = "main"` with no `rev`) is the rule that
keeps one revision, and this check is what catches a lock that breaks it. The check applies
to every first-party crate without a list of their names, so a first-party crate added later
is checked from the day it is added.

The tool is the one AD-007 O-4 names. The AD recommends a new small tool repository whose
binary reads the raw `Cargo.lock` once for all callers; it also records cargo-deny
configuration as an alternative with its costs (the check then judges the cfg-resolved graph
rather than the raw lock, cannot condition an exception on a crate's source, and must run
with `-D unmatched-skip` so that an unused `skip` entry fails). The criteria below hold for
either; only the mechanism of AC-1 and AC-4 changes. The check is run by `make deny`.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-043-AC-1 | `make deny` runs the owned one-copy check over this repository's `Cargo.lock`, and no tracked file in the repository contains the check's logic: `scripts/check_one_copy.awk` does not exist, no tracked file is byte-equal to it, and no `Makefile` or `deny.toml` line names it. PLANNED (IR-581). | Test (TC-441) |
| FR-043-AC-2 | Each of three fixture locks fails the check and the output names the crate: (a) two entries of one first-party crate at one version from two git revisions; (b) two entries of one first-party crate at two versions; (c) one git entry and one registry entry of one first-party crate name. A tool that compares git revisions alone fails this criterion. PLANNED (IR-581). | Test (TC-441) |
| FR-043-AC-3 | Given a fixture lock with one entry for each first-party crate and a third-party crate that appears twice, and given this repository's own `Cargo.lock`, the check passes. PLANNED (IR-581). | Test (TC-441) |
| FR-043-AC-4 | Every exception the check allows (for cargo-deny, each `skip` entry) names no first-party crate (as defined above) and carries a reason, and an exception that matches no duplicate in the lock makes the check fail (for cargo-deny, `-D unmatched-skip`). A tool that has no exceptions meets this criterion trivially. PLANNED (IR-581). | Test (TC-441) |

## Dependencies

AD-007 (the owner table row, G-3, G-4 and O-4) states the decision and the measured options.
FR-028 states why this repository's own graph holds one copy of its model crate.

## Status

All four criteria are planned (IR-581): no code has landed. The three copies of the awk
script in IR, CG and RT are deleted by each repository's own change, after the owned check is
invoked there; IR's deletion is this requirement's code change. CG, RT and QSL are separate
lanes and are not specified here.
