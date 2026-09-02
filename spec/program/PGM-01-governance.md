---
id: PGM-01
title: "Cross-repository governance and evidence policy"
type: policy
---
# PGM-01: Cross-repository governance and evidence policy

PGM-01 is the single normative governance source for `quire-contract-ir`,
`quire-contract-runtime`, `quire-contract-codegen`, `quire-analyze`,
`tl-syntax`, `tl-parse`, `tl-rewrite`, and `tl-mltl`. A workstream specification
shall cite `ix://agent-ix/quire-contract-ir/PGM-01`; it shall not fork or weaken
this policy. A local constraint may be stricter when it identifies PGM-01 as its
baseline and describes the delta.

## Policy

All requirements in this section are mandatory for v0.1 candidates unless the
named human release authority records a bounded exception under PGM-01-R09.

### PGM-01-R01 — schema compatibility

- Every serialized boundary shall carry a non-empty schema identity and an
  explicit major wire version. Each identity belongs to the domain artifact it
  describes; the program declares no common evidence envelope or retention
  schema, and the withdrawn `quire.derivation-evidence/v1` identity is not
  reused (see PGM-01-R08).
- A consumer shall reject an unknown major version and shall not guess,
  silently migrate, or reinterpret it. A migration shall be explicit,
  versioned, tested, and recorded as a derivation.
- A v1 schema revision may clarify descriptions or relax validation without
  changing accepted values' meaning. New required fields, renamed fields,
  changed meaning, or newly ambiguous interpretation require `/v2`.
- Checked-in schemas and corpus files are release artifacts. Consumers shall
  pin both schema identity and SHA-256 digest; a crate version alone is not a
  wire-schema identity.

### PGM-01-R02 — crate compatibility and pins

- Crates follow Cargo semantic versioning. Before 1.0, a minor release may be
  breaking; a patch release shall preserve the documented public contract.
- Cross-repository development uses exact source revisions. A release candidate
  uses exact source tags plus checksums and retains `Cargo.lock` for tools and
  evidence collection. Version ranges are convenience declarations, not
  qualification pins.
- External engines, adapters, toolchains, schemas, inputs, outputs, feature
  sets, target triples, and material configuration shall be pinned by version
  and digest in retained evidence. An omitted identity is an invalid record.

### PGM-01-R03 — source-release order

Each source tag is `v0.1.0` for the first program release. A repository may tag
only after the exact dependency tags and checksums are available:

```text
quire-contract-ir
├── quire-contract-codegen (also requires quire-contract-runtime)
└── quire-analyze

quire-contract-runtime
└── quire-contract-codegen (also requires quire-contract-ir)

tl-syntax
├── tl-parse
├── tl-mltl
└── tl-rewrite (also requires retained evaluator evidence)
```

`quire-contract-ir`, `quire-contract-runtime`, and `tl-syntax` are independent
initial source-tag roots and may be tagged in any order. `quire-contract-codegen`
follows the IR and runtime; `quire-analyze` follows the IR; `tl-parse` and
`tl-mltl` follow `tl-syntax`; and `tl-rewrite` follows `tl-syntax` plus retained
evaluator evidence for its rewrite corpus. Where an actual manifest adds a
dependency, normal topological order applies. A source-release manifest shall
name every exact dependency tag, commit, and checksum. Rebuilds of an existing
tag are forbidden.

### PGM-01-R04 — licensing and third-party provenance

- Every program repository and reusable template is `MIT OR Apache-2.0`.
  Registry publication remains disabled through the v0.1 source review.
- Generated source defaults to `SPDX-License-Identifier: MIT OR Apache-2.0`.
  A consumer-selected SPDX expression may replace that default only when the
  generator records the selection and every incorporated template permits it.
  Generated evidence retains its schema license and provenance notice.
- Every copied or adapted third-party element shall record origin URI, immutable
  revision, path or span, copyright notice, license expression, retrieval
  digest, transformation, and reviewer. Dependency metadata is retained in the
  lockfile and license report. Unknown, incompatible, or absent licenses block
  incorporation.

### PGM-01-R05 — clean-room grammar rule

Ambiguously licensed or unlicensed grammar text, parser tables, tests, examples,
and code shall not be copied, translated, mechanically transformed, or used as
a line-by-line implementation guide. A clean-room implementation shall retain:

1. the public, lawfully accessible behavioral sources used as requirements;
2. an origin-and-license inventory reviewed before implementation;
3. an independently authored grammar and fixtures;
4. contributor attestations that prohibited material was not used; and
5. differential results containing inputs and outcomes, never copied internals.

A failed provenance review blocks the affected source and fixtures. Merely
calling an implementation “clean room” is not evidence.

### PGM-01-R06 — contribution provenance and human authority

Human, agent-assisted, generated, and mixed contributions are accepted under
the same requirements, review, testing, license, and provenance gates. The
contribution method shall be reported truthfully; it neither grants nor removes
technical merit. Agents may prepare changes and evidence but may not approve
their own work or make a release decision.

`@kreneskyp` is the v0.1 human source-release authority. `.github/CODEOWNERS`
assigns every path to that owner, and protected `main` requires a non-stale
CODEOWNER approval, strict required checks, and resolved conversations. Admin
enforcement is the default policy; only that human may record a bounded admin
bypass exception under PGM-01-R09. Only that human may record sufficiency,
authorize a source tag, or reject a candidate. Automation shall leave the
decision open.

### PGM-01-R07 — component and artifact classification

| Repository or artifact | Primary class | Boundary consequence |
|---|---|---|
| `quire-contract-ir`; schemas and corpus | direct development tool | Defines inputs and identities; not shipped as generated customer runtime. |
| `quire-contract-runtime` | linked runtime | Linked code is inside the consuming software boundary and needs target-project verification. |
| `quire-contract-codegen` | direct development tool | Generator stays outside runtime; emitted oracle/proptest/Kani Rust is linked runtime. |
| Codegen maps and derivation records | analysis/evidence tool | Domain-producer results identify the generating tool and may be retained by Quoin; they never self-approve. |
| `quire-analyze`; consistency/implication reports | analysis/evidence tool | Domain-producer results identify solver/configuration and preserve non-conclusive states. |
| Z3 and cvc5 integrations | external engine adapter | The engine binary, adapter, encoding, configuration, and result are separate identities. |
| `tl-syntax` | linked runtime | May be linked into consuming evaluators or monitors; use is project-scoped. |
| `tl-parse` | direct development tool | Parsed output is a derivation; deployment of the parser changes the project boundary. |
| `tl-rewrite`; rewrite proof records | analysis/evidence tool | Domain results retain source/output/profile identity and do not establish semantic preservation alone. |
| `tl-mltl`; reference evaluation reports | analysis/evidence tool | Domain results support comparison but do not accredit a production monitor. |
| R2U2/C2PO integrations and monitor inputs | external engine adapter | External engine/version/configuration and adapter identity are mandatory. |

When actual deployment differs from the primary class, the authoritative
domain result shall declare the deployed role and the consuming project shall
reassess the boundary. Classification does not confer qualification.

The shared responsibility assignment is exact:

| Responsibility | Authoritative owner | Boundary |
|---|---|---|
| Static verification definitions, obligations, relations, and locators | Quire | Parses and exports only; does not invoke producers. |
| Verification execution | Contract or temporal domain producer | Invoked by the operator or project-native build/test system. |
| Structured domain result and diagnostic | Originating contract or temporal producer | Preserves its native result states and domain semantics. |
| Evidence intake, retained bytes, integrity, audit, and report views | Quoin | Consumes explicit structured inputs; does not invoke producers. |
| Human approval, rejection, revision, and workflow state | ix-flow | Retains attributed decisions; tools do not infer them. |
| Cross-repository campaign policy and release order | PGM-01 | Governs boundaries without becoming a runner or evidence store. |

Published runtime and domain crates shall not acquire runtime dependencies on
Quire or Quoin. Development-time export, validation, intake, audit, and report
commands may use exact pinned releases without linking them into customer
software. Quire and Quoin are explicitly non-executing.

### PGM-01-R08 — domain derivation provenance and structured results

**Withdrawn 2026-09-02 by the repository owner.** This rule previously required
every generated, transformed, analyzed, proved, tested, monitored, or packaged
domain artifact to carry a producer-owned derivation record in a published
Draft 7 envelope schema, validated here by a repository-local Draft 7 corpus and
its mutation probes. That envelope was the same deprecated pre-stable format as
the retained records deleted under PGM-01-R09, and it is withdrawn on the same
ground and under the same decision, recorded in
[engineering-assurance#7](https://github.com/agent-ix/engineering-assurance/issues/7)
under "Preservation constraint released for the pre-stable phase". The schema,
its validator, its fixture corpus, the discrete requirement FR-008 that carried
it, and the test cases TC-005 through TC-012 that drove it are deleted. Nothing
replaces them: no successor envelope, no second result family, and no local
validator is introduced in their place.

What survives is not a weaker version of the rule but the part that never
depended on the envelope. A domain producer still owns its own structured
result and its own diagnostics. SHA-256 remains the v1 digest algorithm, and
digests are lowercase hexadecimal over the exact bytes stored or transferred.
Quoin may retain a producer's result or its exact output bytes and cite them
from an audit or report; it shall not copy their fields into a second generic
result family. Quire may link static definition identity but shall not execute
the producer.

This repository's own live structured result is the conformance runner's
`quire.contract.conformance-jsonl/v1` stream over
`corpus/contract-v0.1/manifest.json`, which reaches Quoin through the declared
adapter under [FR-022](../functional/FR-022-shared-assurance-intake.md). Its
wire form and its corpus manifest are described by
`schemas/contract-package-reference-v1.schema.json` and
`schemas/contract-conformance-manifest-v1.schema.json`, which are domain
contracts owned here rather than an evidence envelope, and are unaffected by
this withdrawal.

### PGM-01-R09 — historical records, assurance handoff, and release decision

**Amended 2026-09-02 by the repository owner.** This rule previously held the
`evidence/pgm-01-<short-source-revision>/` records, their external checksum
files, corrections, and the `quire.pgm01-evidence/v1` schema as immutable
historical inputs and a closed set, readable only through Engineering
Assurance's explicit read-only compatibility mapping. The owner released that
preservation constraint for the pre-stable phase, on the ground that this is
early-development output; the decision and its exact scope are recorded in
[engineering-assurance#7](https://github.com/agent-ix/engineering-assurance/issues/7)
under "Preservation constraint released for the pre-stable phase". This
repository's retained records, its compatibility census, and the schemas frozen
only because those records named them are accordingly deleted. Nothing was
rewritten, backdated, or re-sealed on the way out, and no claim that they still
verify anything survives them.

The rest of the rule is unchanged and still binding. A repository shall not
carry a local verifier, envelope, manifest, or retention authority over
evidence, because a second authority over evidence bytes is exactly the parallel
evidence family this campaign removes. The preservation constraint itself
re-applies unchanged at the move toward stable releases, and evidence retained
from that point is immutable.

For shared-assurance adoption, an operator or project-native system invokes the
domain producer and supplies its structured result to Quoin. Quoin owns
retention, integrity checks, audit, and report views; Quire supplies only static
definition references; ix-flow owns any human decision. The shared components
are reached at the exact released pins the accepted compatibility matrix
records; a branch head, a bare revision, or a floating tag is not a pin. Failed,
skipped, unavailable, not-computed, inconclusive, partial, stale, suspect,
vacuous, tampered, unsupported, and unreadable states remain explicit and
distinct from one another.

The human release decision shall identify the exact candidate, authoritative
record references, open gaps and accepted exceptions, decision (`approve`,
`reject`, or `defer`), rationale, owner `@kreneskyp`, timestamp, and authorized
tag. It is retained as an ix-flow decision event once that pinned path is
adopted. Absence of a decision means no release decision. CI success is
evidence, not approval.

### PGM-01-R10 — qualification and accreditation boundary

The crates, schemas, corpora, and adapters provide reusable
qualification support only. Their release does **not** validate or accredit a
tool for a consuming project and does not certify a system under NASA, DO-178C,
ISO 26262, IEC 61508, or any other regime. Any validation or accreditation is
specific to the consuming project, intended use, tool/version/configuration,
environment, hazards, evidence, independent review, and named decision
authority. Claims shall state that bounded context and shall not inherit a
program release decision by implication.

### PGM-01-R11 — campaign reconciliation and legacy prototype disposition

The program epic's former statement that Quire and Quoin integration is outside
the campaign is superseded. Shared static-definition export, result intake,
retention, audit, reporting, and human-decision integration are common-work
dependencies governed by [Engineering Assurance #7](https://github.com/agent-ix/engineering-assurance/issues/7). Domain repositories
still own their producers and structured results, and their published crates
remain runtime-independent from Quire and Quoin.

[`quire-contract-ir#7`](https://github.com/agent-ix/quire-contract-ir/issues/7)
is re-scoped to a post-release inventory of conditional Quire/Quoin
catalog and adapter opportunities. It does not defer ownership decisions or
authorize repository migrations. [`quire-contract-ir#20`](https://github.com/agent-ix/quire-contract-ir/issues/20)
is superseded by the common-work
gates: its repeatability and fail-closed threat cases remain valuable, while
its proposed shared executor, central execution profile, evidence authority
index, and retention component are rejected.

The standalone `quire-evidence` prototype is not an adopted component. Preserve
as technology-independent adversarial fixtures: checksum/reseal tampering,
fabricated or contradictory success, tool shadowing and identity drift,
historical profile/parameter drift, deletion/retraction ambiguity, exact
resource ceilings, descendant timeout containment, distinct unavailable and
failed outcomes, and mutations that must turn a domain oracle red. Preserve as
domain cases: contract validation/canonicalization, property/proof/vacuity
results, SMT non-conclusive results, and temporal parse/rewrite/evaluation
results. Do not preserve its generic command executor, execution profile,
overall verdict aggregation, authority index, repository adoption command, or
retention layout.

## Acceptance Criteria

| ID | Criterion | Verification |
|---|---|---|
| PGM-01-R01-AC-1 | Unknown schema majors and silent migration are forbidden. | Policy inspection TC-001 |
| PGM-01-R02-AC-1 | Exact release and qualification pins are mandatory. | Policy inspection; TC-001 |
| PGM-01-R03-AC-1 | All eight repositories have a topological source-tag rule. | TC-002 |
| PGM-01-R04-AC-1 | Generated and third-party material has explicit license provenance. | Policy inspection; TC-003 |
| PGM-01-R05-AC-1 | Clean-room sources and prohibited reuse are explicit. | Policy inspection; TC-003 |
| PGM-01-R06-AC-1 | Human authority is named and enforced by CODEOWNERS/protection. | TC-004; protected-branch API evidence |
| PGM-01-R07-AC-1 | Each crate and emitted artifact has a boundary class. | TC-002 |
| PGM-01-R09-AC-1 | An automated record cannot replace the human decision. | Policy inspection; TC-004 |
| PGM-01-R09-AC-2 | The shared components are reached at accepted released pins and every non-success state the surviving path covers stays distinct. | TC-029, TC-033 |
| PGM-01-R10-AC-1 | Release does not confer project validation/accreditation. | Policy inspection; TC-003 |
| PGM-01-R11-AC-1 | Every shared responsibility has one owner; issue #7/#20 and the legacy prototype have explicit linked dispositions. | TC-023, TC-025 through TC-028 |
