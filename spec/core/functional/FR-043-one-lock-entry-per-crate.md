---
id: FR-043
title: "Hold one lock entry per crate, checked by one owned tool"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/FR-028
    type: references
  - target: ix://agent-ix/quire-contract-ir/AD-007
    type: references
---
# FR-043: Hold one lock entry per crate, checked by one owned tool

## Description

The repository's `Cargo.lock` SHALL hold one entry for each crate name, at one version
and, for a git source, at one revision, and `make deny` SHALL fail when it holds two. The
check SHALL be run by one owned tool that is not a copy in this repository, so the
repository carries no script that another repository also carries. This is AD-007's
invariant G-4 for IR, decided by the owner's ruling on AD-007 O-4 (first-hand, 2026-10-04,
recorded on IR-581).

## Inputs

The repository's `Cargo.lock` and `deny.toml`.

## Outputs

A pass, or a failure that names the crate and the number of lock entries it holds.

## Behavior

Two entries of one first-party crate at different revisions of one version build the crate
twice with types that do not unify (FR-028's reason for the branch rule), and the check
exists to refuse that lock. The check applies to every first-party crate without a list of
their names, so a first-party crate added later is checked from the day it is added.

The tool is the one AD-007 O-4 names. The AD recommends cargo-deny configuration: a global
`multiple-versions = "deny"` in `deny.toml`, with a `skip` entry for each third-party crate
that legitimately appears twice, and no `skip` entry for a first-party crate. If the owner
chooses another tool (AD-007 O-4a and O-4b), the criteria below still hold and only AC-1's
mechanism changes. The check is run by `make deny`, and by CI where CI runs the deny gate.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-043-AC-1 | `make deny` runs the owned one-copy check over this repository's `Cargo.lock`, and the repository holds no file whose job is the check: `scripts/check_one_copy.awk` does not exist and nothing in the `Makefile` or `deny.toml` names it. PLANNED (IR-581). | Test (TC-441) |
| FR-043-AC-2 | Given a fixture lock with two entries of one first-party crate at one version from two revisions of one git repository, the check fails and its output names that crate. PLANNED (IR-581). | Test (TC-441) |
| FR-043-AC-3 | Given a fixture lock with one entry for each first-party crate, and given this repository's own `Cargo.lock`, the check passes. PLANNED (IR-581). | Test (TC-441) |
| FR-043-AC-4 | Every exception the check allows (for cargo-deny, each `skip` entry) names a third-party crate and carries a reason; no exception names a crate whose lock source is under the agent-ix organisation. PLANNED (IR-581). | Test (TC-441) |

## Dependencies

AD-007 (the owner table row, G-4 and O-4) states the decision and the measured options.
FR-028 states why a second copy of a first-party crate is a defect.

## Status

All four criteria are planned (IR-581): no code has landed. The three copies of the awk
script in IR, CG and RT are deleted by each repository's own change; IR's is this
requirement's code change. CG, RT and QSL are separate lanes and are not specified here.
