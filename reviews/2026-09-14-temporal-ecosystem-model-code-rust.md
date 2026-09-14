---
id: SR-538
title: "Code and Rust review of the bounded temporal ecosystem model"
type: SpecReview
analysis: code-review
scope: "quire-contract-ir#74; PLAN-007; FR-027; TC-040; manifest admission, typed graph, deterministic model export/read and proposal boundary"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-027
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/TC-040
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/PLAN-007
    type: reviews
---

## Summary

The review evaluated exact candidate
`60cc38be060821ef64b150b5017d0cddf48978f3` against FR-027, AD-001,
PLAN-007, TC-040 and the complete diff from merged base
`69ec82bf4da1bdbee710544a4570c2042dc781a5`. The candidate implements a
closed nine-repository manifest, typed ownership/dependency/evidence graph,
deterministic bounded model, independent re-exporting reader and descriptive
proposal value. It exercises the real predicate and temporal owner path but
adds no parser, evaluator, network discovery, callback, trust flag or authority
transition.

## Verdict

**PASS** — every code, Rust, identity, resource, graph, architecture and test
finding found during review was repaired. No actionable scoped finding remains.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-6500 | high | **FIXED:** the first model identity used a convenience identity profile instead of FR-027's exact output profile plus NUL and identity-omitted body. The implementation now hashes exactly the specified preimage and tests recompute it independently. | `src/ecosystem_model/document.rs`; FR-027-AC-1/3 |
| FND-6501 | high | **FIXED:** the first campaign expectation pinned repository revisions but not the complete manifest selection, allowing a caller-authorized population change under the same campaign label. `ExpectedCampaign` now requires the exact raw manifest digest before exposing a checked view. | `src/ecosystem_model/manifest.rs`; FR-027-AC-1/2 |
| FND-6502 | medium | **FIXED:** canonical byte/depth/string/work exhaustion was initially mapped to invalid-document. Resource failures now retain the closed `ecosystem_resource_exhausted` code. | `src/ecosystem_model/manifest.rs`; `STD-001` |
| FND-6503 | medium | **FIXED:** export initially trusted the admission-time node ceilings even when its caller selected lower model ceilings. Export now checks every retained node, gap and edge count against its own effective limits; exact and one-under tests cover every dimension. | `src/ecosystem_model/document.rs`; TC-040 |
| FND-6504 | medium | **FIXED:** the initial allocation owner maximum was effectively unbounded. The public platform-independent limit now clamps to a finite 128 MiB owner maximum, and deterministic failpoints return no partial view. | `src/ecosystem_model/manifest.rs`; FR-027-AC-4 |
| FND-6505 | medium | **FIXED:** gap ordering rejected exact duplicate rows but admitted the same gap identity with different content. Admission now enforces distinct gap identities independently of row ordering. | `src/ecosystem_model/manifest.rs`; TC-040 |
| FND-6506 | low | **FIXED:** a selected repository could initially own no runtime component. Every repository now must own at least one declared component; every non-repository node has exactly one repository owner and every executable contract exactly one component owner. | `src/ecosystem_model/graph.rs`; FR-027-AC-2 |
| FND-6507 | low | **FIXED:** the first schema node maximum was miscomputed and did not declare unique arrays. Both immutable schemas now match the closed maxima and require unique node/edge/gap populations and topological identities. | `schemas/temporal-ecosystem-*.schema.json` |
| FND-6508 | low | **FIXED:** authority-boundary coverage inspected the API but did not compile a forbidden consumer. Compile-fail doctests now prove checked manifests cannot be forged and validated models expose no acceptance transition. | `src/ecosystem_model/manifest.rs`; `src/ecosystem_model/reader.rs` |
| FND-6509 | low | **FIXED:** the initial example manifest selected only its two new document contracts. It now retains every exact predicate and temporal owner contract exercised by the end-to-end path. | `tests/ecosystem_model.rs`; FR-027-AC-1/5 |

## Rust Review

- The public facade is `ecosystem_model::{manifest::read, export, read}` with
  constructor-private checked/validated values and immutable canonical bytes.
- Production source contains no `unsafe`, panic/unwrap/expect path, unchecked
  numeric cast, stub, async/blocking bridge, lock, process or network access.
- Wire lengths and platform-independent limits use `u64`; conversion to host
  `usize` clamps safely and never narrows a wire value unchecked.
- `BTreeMap`/`BTreeSet`, explicit sort keys and prerequisite-first Kahn ordering
  make graph projections independent of insertion and hash-map order.
- Strict readers reject noncanonical, unknown, duplicate and trailing input,
  exact-contract/campaign mismatch, every graph inconsistency and all configured
  resource excess before returning a checked or validated public view.
- TC-040 uses real QSL, QObs, QProtocol, tl-syntax and tl-mltl public owners.
  Native and TL results are produced and read independently before Contract IR
  performs a structural join; no mock or Boolean fallback establishes success.
- Repository evidence revisions and immutable contract-publication revisions
  remain separate axes. Repository identity must agree with ownership, while a
  valid earlier owner publication is not incorrectly retargeted to a later
  evidence snapshot.

## Gates

| Gate | Result |
| --- | --- |
| `cargo fmt --all -- --check` | pass |
| locked workspace/all-target/all-feature tests | pass; 82 Rust tests |
| locked workspace doctests | pass; 2 compile-fail tests |
| all-feature workspace/all-target Clippy with `-D warnings` | pass |
| release workspace build | pass |
| warning-denied public documentation | pass |
| `cargo deny check` | pass; advisories, bans, licenses and sources clean |
| `cargo audit` | pass; 135 dependencies scanned |
| schema digest sidecars | pass |
| unsafe/stub/panic-path source audit | pass |
| Quire 0.32.0 FR-027 coverage | pass; 6/6 criteria and 5 TC-040 symbols |

Repository-wide Quire coverage is 137/144. FR-027 is 6/6, both test matrices
are fully backed, and Quire reports zero unbacked matrix rows, status lies or
untracked symbols. The seven remaining repository-wide denominator gaps are
pre-existing NFR portability/toolchain rows outside PLAN-007.
