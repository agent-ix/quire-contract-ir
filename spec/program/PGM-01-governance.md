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
  explicit major wire version. The common evidence envelope identity is
  `quire.derivation-evidence/v1`.
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
CODEOWNER approval, strict required checks, resolved conversations, and admin
enforcement. Only that human may record sufficiency, accept a bounded exception,
authorize a source tag, or reject a candidate. Automation shall leave the
decision open.

### PGM-01-R07 — component and artifact classification

| Repository or artifact | Primary class | Boundary consequence |
|---|---|---|
| `quire-contract-ir`; schemas and corpus | direct development tool | Defines inputs and identities; not shipped as generated customer runtime. |
| `quire-contract-runtime` | linked runtime | Linked code is inside the consuming software boundary and needs target-project verification. |
| `quire-contract-codegen` | direct development tool | Generator stays outside runtime; emitted oracle/proptest/Kani Rust is linked runtime. |
| Codegen maps and derivation records | analysis/evidence tool | Inform assurance; never self-approve a result. |
| `quire-analyze`; consistency/implication reports | analysis/evidence tool | Conclusions identify solver/configuration and preserve non-conclusive states. |
| Z3 and cvc5 integrations | external engine adapter | The engine binary, adapter, encoding, configuration, and result are separate identities. |
| `tl-syntax` | linked runtime | May be linked into consuming evaluators or monitors; use is project-scoped. |
| `tl-parse` | direct development tool | Parsed output is a derivation; deployment of the parser changes the project boundary. |
| `tl-rewrite`; rewrite proof records | analysis/evidence tool | Rewrites retain source/output/profile identity and do not establish semantic preservation alone. |
| `tl-mltl`; reference evaluation reports | analysis/evidence tool | Reference results support comparison but do not accredit a production monitor. |
| R2U2/C2PO integrations and monitor inputs | external engine adapter | External engine/version/configuration and adapter identity are mandatory. |

When actual deployment differs from the primary class, the evidence envelope
shall declare the deployed role and the consuming project shall reassess the
boundary. Classification does not confer qualification.

### PGM-01-R08 — common derivation and evidence envelope

Every generated, transformed, analyzed, proved, tested, monitored, or packaged
artifact shall have a record conforming to
`schemas/derivation-evidence-envelope-v1.schema.json`. The record shall identify:

- envelope schema and record identity;
- producer name, version, source revision, executable digest, and invocation;
- every material input's role, URI, schema identity/version/digest, and content
  digest;
- the backend as either an explicitly justified `none` or a named engine/tool
  with version, executable digest, and configuration digest;
- every output's role, URI, media type, schema identity/version/digest, and
  content digest;
- parameter, dependency, environment, repository, contribution-method, and
  candidate identities; and
- a typed result that preserves `inconclusive`, `unsupported`, `rejected`,
  `timed-out`, `pending`, and `error` rather than converting them to success.

SHA-256 is the v1 digest algorithm. Digests are lowercase hexadecimal over the
exact bytes stored or transferred. A transformation with no external backend
uses `kind: none` with a reason; omission is never equivalent to none. An
extension key must be a reverse-DNS name and must not change core-field meaning.

The published Draft 7 schema is the normative validation boundary. The
conformance report uses `UNSUPPORTED_SCHEMA`, `MISSING_PRODUCER`,
`MISSING_INPUTS`, `MISSING_SCHEMA_IDENTITY`, `MISSING_BACKEND`,
`MISSING_OUTPUTS`, and `INVALID_DIGEST` for those targeted conditions; every
other Draft 7 failure is `SCHEMA_VIOLATION`. Error order is deterministic by
instance path then schema message.

### PGM-01-R09 — evidence retention and release record

Candidate evidence shall be immutable, revision-scoped, content-addressed, and retained with a
manifest containing source revision, collection time, commands, tool/dependency
identities, environment, individual outcomes, limitations, and checksums. A
rerun is a new record at `evidence/pgm-01-<short-source-revision>/`. Failed and
skipped measurements remain visible. A checksum file outside that directory
covers its manifest and every retained output without self-reference. The
manifest conforms to the checked-in `quire.pgm01-evidence/v1` schema and pins
that schema by path and SHA-256 digest. Release verification compares the
complete non-evidence `HEAD` tree with the recorded input set so squash-merged
records remain verifiable and later added files cannot escape coverage.

The human release record shall identify the candidate and evidence manifest,
open gaps and accepted exceptions, decision (`approve`, `reject`, or `defer`),
rationale, owner `@kreneskyp`, timestamp, and authorized tag. Absence of that
record means no release decision. CI success is evidence, not approval.

### PGM-01-R10 — qualification and accreditation boundary

The crates, schemas, corpora, adapters, and retained evidence provide reusable
qualification support only. Their release does **not** validate or accredit a
tool for a consuming project and does not certify a system under NASA, DO-178C,
ISO 26262, IEC 61508, or any other regime. Any validation or accreditation is
specific to the consuming project, intended use, tool/version/configuration,
environment, hazards, evidence, independent review, and named decision
authority. Claims shall state that bounded context and shall not inherit a
program release decision by implication.

## Acceptance Criteria

| ID | Criterion | Verification |
|---|---|---|
| PGM-01-R01-AC-1 | Unknown schema majors and silent migration are forbidden. | TC-008; policy inspection TC-001 |
| PGM-01-R02-AC-1 | Exact release and qualification pins are mandatory. | Policy inspection; TC-001 |
| PGM-01-R03-AC-1 | All eight repositories have a topological source-tag rule. | TC-002 |
| PGM-01-R04-AC-1 | Generated and third-party material has explicit license provenance. | Policy inspection; TC-003 |
| PGM-01-R05-AC-1 | Clean-room sources and prohibited reuse are explicit. | Policy inspection; TC-003 |
| PGM-01-R06-AC-1 | Human authority is named and enforced by CODEOWNERS/protection. | TC-004; protected-branch API evidence |
| PGM-01-R07-AC-1 | Each crate and emitted artifact has a boundary class. | TC-002 |
| PGM-01-R08-AC-1 | Tool, input, schema, backend, and output identities cannot be omitted. | TC-005 through TC-012 |
| PGM-01-R09-AC-1 | Evidence cannot silently replace the human decision. | Policy inspection; TC-004 |
| PGM-01-R10-AC-1 | Release does not confer project validation/accreditation. | Policy inspection; TC-003 |
