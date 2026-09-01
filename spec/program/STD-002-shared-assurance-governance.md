---
id: STD-002
title: "Shared assurance governance reconciliation"
type: Standard
code: shared-assurance-governance-v1
relationships:
  - target: ix://agent-ix/quire-contract-ir/PGM-01
    type: references
  - target: ix://agent-ix/quire-contract-ir/issues/38
    type: references
---
# Shared assurance governance reconciliation

This document records the reviewed disposition for
[`quire-contract-ir#38`](https://github.com/agent-ix/quire-contract-ir/issues/38).
The normative assignment remains
[PGM-01-R07](../spec/program/PGM-01-governance.md#pgm-01-r07--component-and-artifact-classification);
this operational summary must agree with it and grants no additional authority.

## Responsibility summary

| Responsibility | Authoritative owner | Operational boundary |
|---|---|---|
| Static verification definitions, obligations, relations, and locators | Quire | Export only; never invoke producers. |
| Verification execution | Contract or temporal domain producer | Operator or project-native build/test invocation. |
| Structured domain result and diagnostic | Originating contract or temporal producer | Preserve native states and domain semantics. |
| Evidence intake, retained bytes, integrity, audit, and report views | Quoin | Consume explicit structured inputs; never invoke producers. |
| Human approval, rejection, revision, and workflow state | ix-flow | Retain attributed decisions; never infer sufficiency. |
| Cross-repository campaign policy and release order | PGM-01 | Govern boundaries; never become a runner or evidence store. |

Quire and Quoin are non-executing. Published runtime and domain crates have no
runtime dependency on either tool. Exact released CLI pins may be used at
development time only after their common-work release gate passes.

## Historical compatibility

Existing PGM-01 records, external checksums, corrections, and schemas remain
immutable inputs. `tests/fixtures/historical-pgm01-files.sha256` locks every
historical record, checksum, correction, and PGM-01 domain/evidence/correction
schema byte present when this reconciliation was accepted. Additions require a
new lock entry but do not rewrite an accepted byte. The local verifier continues
to authenticate the original representation.

The compatibility implementation is the explicit read-only, lossy mapping
owned by [Engineering Assurance #5](https://github.com/agent-ix/engineering-assurance/issues/5)
and merged at
[`ee4db6f`](https://github.com/agent-ix/engineering-assurance/commit/ee4db6fc9c22544d8c8bfc8f0b97fe097835c029).
It returns source-field references and limitations. It does not mutate source
bytes, synthesize missing producer/configuration/decision fields, or turn a
legacy intake or merge-readiness label into a successful check or human
decision.

## Campaign issue dispositions

The required repository state after this gate is:

| Issue | Disposition | Boundary |
|---|---|---|
| [`#1`](https://github.com/agent-ix/quire-contract-ir/issues/1) | reconciled | The future-integration deferral is replaced with the exact shared ownership and dependency order. |
| [`#7`](https://github.com/agent-ix/quire-contract-ir/issues/7) | re-scoped | Open only as a post-release inventory of conditional catalog and adapter opportunities; authorizes no migration. |
| [`#20`](https://github.com/agent-ix/quire-contract-ir/issues/20) | superseded and closed | Reusable threats and domain cases are preserved; the proposed component architecture is rejected in favor of [Engineering Assurance #7](https://github.com/agent-ix/engineering-assurance/issues/7). |

The exact inspected states, timestamps, body digests, and #20 closure-comment
identity are retained in
`tests/fixtures/campaign-issue-dispositions-v1.json`, and the inspected bodies
themselves are retained verbatim under `tests/fixtures/campaign-bodies/`.

The scope of that receipt, and of TC-026 which checks it, is exact:

- Proven offline: each retained body still hashes to the digest the receipt
  records, and every `requiredMarkers` entry occurs — and every `absentMarkers`
  entry does not occur — in those retained bytes. This is fixture integrity.
- Not proven offline: that the live GitHub issues, comments, states, or
  timestamps still match. That correspondence was established once, by live
  inspection at the recorded `observedAt`, and is not re-observed by any test.

The duplicated constants in TC-026 are therefore an integrity binding, not a
live GitHub oracle. A later issue edit requires a new reviewed receipt and
newly retained bytes before any claim relies on the changed state.

## Legacy prototype inventory

The standalone `quire-evidence` prototype is not an adopted component. The
following technology-independent cases remain useful inputs for later native
domain or adapter fixtures.

Accepted adversarial cases:

- `checksum-reseal-tamper`
- `fabricated-or-contradictory-success`
- `tool-shadowing-and-identity-drift`
- `historical-profile-or-parameter-drift`
- `deletion-retraction-or-authority-ambiguity`
- `exact-resource-ceilings-and-overflow-nonpublication`
- `descendant-timeout-containment`
- `failed-versus-unavailable`
- `mutation-must-turn-domain-oracle-red`

Accepted domain cases:

- `contract-validation-and-canonicalization`
- `property-proof-and-vacuity-results`
- `smt-nonconclusive-results`
- `temporal-parse-rewrite-and-evaluation-results`

Rejected architecture:

- `generic-command-executor`
- `central-execution-profile`
- `aggregate-overall-verdict`
- `evidence-authority-index`
- `repository-adoption-command`
- `parallel-result-family`
- `retention-layout-or-store`

## Common-work order

```text
Engineering Assurance #5 verification semantics (complete)
  -> quire-contract-ir #38 governance reconciliation
    -> Quire CLI #74 static export
      -> Quoin CLI #322 explicit result intake
        -> Engineering Assurance #9 compatibility fixtures
          -> Quoin #323 adapter inventory
            -> Engineering Assurance #8 exact releases and pins
              -> Engineering Assurance #10 migration contract
                -> eight separately reviewed repository migrations
```

The eight migration issues are not part of this gate. No migration begins until
all common gates above it are reviewed, released where applicable, and pinned.
