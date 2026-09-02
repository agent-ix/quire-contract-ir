---
id: SR-032
title: "Code review — drop the legacy evidence cruft"
type: SpecReview
analysis: code-review
scope: "branch chore/drop-legacy-evidence against origin/main 69cf238; evidence/, scripts/pgm01_compatibility_view.py, tests/fixtures/legacy-compat/, schemas/, assurance/, spec/"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-022
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/FR-009
    type: references
  - target: ix://agent-ix/quire-contract-ir/PGM-01
    type: references
---

# SR-032: Code review — drop the legacy evidence cruft

## Summary

Review of the change that deletes this repository's retained legacy evidence and
everything that existed only to carry it. The authority is the repository
owner's decision of 2026-09-02, transcribed into
[engineering-assurance#7](https://github.com/agent-ix/engineering-assurance/issues/7)
under "Preservation constraint released for the pre-stable phase". The epic's
completion criterion and its mandatory preservation control were amended before
this work, so no live constraint is being violated.

The deletion is irreversible, so the review concentrated on one question:
**does anything still need what was removed?**

## What makes this repository different

`quire-contract-ir` held the campaign's only working PGM-01 mapping. All ten
retained records mapped through
`engineering_assurance.verification_semantics.map_pgm01_bytes` with outcome
`lossy`, every source digest preserved, 43 files read and 0 bytes moved. The
other seven campaign repositories hold `quire.derivation-evidence/v1` envelopes
that the same pinned mapping refuses outright. This is the one place where the
compatibility view demonstrated something working rather than reporting a
refusal, and it is being removed under the released preservation constraint
rather than because it failed.

## Verdict

**CONDITIONAL** — no `high` finding remains open. Fourteen findings across **two
independent adversarial review rounds**, the second run against the exact final
head: eight from the first round, one `high` (FND-309) raised against this
change's own replacement work, and five from the re-review — which returned **no
high finding** and confirmed every remediation red under mutation. Twelve are
FIXED and two ACCEPTED with stated rationale. None deferred.

Every fix in the second round was checked in **both** directions — what it now
catches, and what the version it replaced caught that it must not stop catching.
That second question is what found FND-311's real shape: the obvious fix would
have swapped a table comparison for an observed-set comparison and silently lost
three assertions the old one did make.

## Exact-head gates

Run with a clean working tree at the branch head.

| Gate | Result |
| --- | --- |
| `make ci` | exit 0 |
| `make spec` | exit 0 |
| `quire coverage --scope . --json` | 0 unbacked rows, 0 status lies |
| `make ci` prerequisites | `fmt-check lint test corpus corpus-repro deny audit-unsafe assurance`, all green |

`release-check` is `ci spec`; both halves are green, so it is satisfiable.

## Coverage arithmetic

Captured before any deletion and again at the branch head.

| Figure | Before | After | Delta |
| --- | --- | --- | --- |
| `unbacked_rows` | 0 | 0 | 0 |
| `status_lies` | 0 | 0 | 0 |
| `totals.backed` | 108 | 102 | −6 |
| `totals.total` | 118 | 111 | −7 |
| `totals.criteria` | 85 | 81 | −4 |

The seven removed rows reconcile exactly against the deleted material:

| Group | Before | After | Rows removed |
| --- | --- | --- | --- |
| `spec/test-matrix.md` test-case | 24/24 | 22/22 | TC-024, TC-032 |
| `spec/contract-test-matrix.md` test-case | 9/9 | 8/8 | TC-022 |
| `FR-009` acceptance-criterion | 4/5 | 3/3 | FR-009-AC-4, FR-009-AC-5 |
| `FR-022` acceptance-criterion | 6/6 | 5/5 | FR-022-AC-4 |
| `NFR-004` nfr-acceptance-criterion | 4/4 | 3/3 | NFR-004-AC-4 |

The criteria delta (−4) equals the four retired acceptance criteria and the
remaining −3 equals the three retired test cases. **No row became unbacked as a
side effect**; every removed row was removed deliberately, with its criterion
retired in writing.

## Pre-deletion state census

Measured against the pre-deletion tree, per state, before anything was removed —
not by asking whether the union of all sources still reached twelve. A union
check would have stayed green while an individual state lost its only
demonstrator.

| State | Demonstrator before | After |
| --- | --- | --- |
| `pass` `fail` `unavailable` `not-computed` `partial` `stale` `tampered` | Quoin chain | unchanged |
| `vacuous` | adapter `refuses-a-vacuous-run` | unchanged |
| `inconclusive` | compatibility census `mapped_states` | TC-006, live governance corpus |
| `suspect` | compatibility census `suspect_demonstrated` | **lost** |
| `unsupported` | *unconditional literal — nothing* | adapter `refuses-a-foreign-protocol` |
| `malformed` | *unconditional literal — nothing* | adapter `refuses-a-malformed-stream` |

Two of the four states the census claimed were never demonstrated by it or by
anything else: `unsupported` and `malformed` were hardcoded strings in the
demonstrated set. Binding them to a matched adapter probe is a strengthening,
and the specification says so rather than presenting it as a like-for-like move.

Eleven of twelve states keep a named demonstrator. `suspect` is a genuine loss,
recorded as one with a test that keeps it recorded rather than quietly covered.

The lookup is per state (`demonstrated[home]`), not a union. Verified red:
pointing one state at a non-existent probe fails with
`states with no demonstrated case: ['malformed']` and nothing else.

### The `malformed` demonstrator, and why it is not a tautology

A gap found and filled with something unfalsifiable reads identically to a gap
filled properly, so the replacement was held to the test *name the specific
defect it catches, introduce that defect, watch it go red*.

This standard was applied to **every** remediation in this change, not only to
the ones flagged. A check written to close a finding is exactly as likely to be
broken as the code it replaces and arrives with less scrutiny, so each of the
five was probed: the per-state demonstrator lookup, the adapter-refusal
distinctness test, the `compatibility.py` digest pin, the removal sweep's
untracked scan, and the `malformed` demonstrator below. Two of those probes
found real defects in the remediation itself — the `malformed` demonstrator's
first form (FND-309) and the sweep's tracked-only first form — and both were
rebuilt rather than argued away.

The probe takes the **real** 99-row producer stream, truncates its **second**
row mid-object, leaves the other 98 exactly as the runner emitted them, and
calls the **actual** adapter. Three properties are deliberate:

- **Real bytes, real code path.** Not a synthesised row in a hand-built
  vocabulary. The adapter refuses with
  `AdapterError: contract-conformance: line 2 is not JSON: Unterminated …`.
- **Surviving rows either side.** If only the bad row were present, an adapter
  that silently *dropped* undecodable rows would leave an empty stream, exit
  non-zero for "no conformance rows", and be indistinguishable from a refusal.
  With 98 good rows, a dropped row leaves a valid run that transcribes and exits
  0 — so "dropped rather than refused" fails loudly.
- **Pre-adapter, compared by reason not exit code.** After the adapter a refused
  row and a failed row are both a non-zero integer, which is exactly where
  `malformed` collapses into `fail`.

Two named defects, each introduced and each observed red:

| Defect introduced | Result |
| --- | --- |
| The adapter drops the undecodable row and keeps the 98 that parse, with the chain told to expect exit 0 so nothing else notices | **RED**, 1 failure, the dedicated assertion alone: *"the adapter transcribed a stream containing undecodable bytes, which means it dropped the bad row … A dropped row is not a refused one"* |
| The adapter still refuses and still exits 1, but rejects the bytes as a foreign protocol rather than as undecodable | **RED**, 2 failures: the distinctness test (`expected 3, got 2`) and *"the refusal must name the decode failure"* |

Both restored to green afterwards. If neither mutation had made it fail it would
not have been a check.

### Can the removal check reach the files a reintroduction would live in?

Two sibling repositories were caught by a census that structurally could not see
the file it claimed to cover: one filtered on file extension, and `Makefile` has
none — so a reintroduced `compat-view` **target** would have lived in the one
file never scanned. A third asserted in its own review that a census covered
`CLAUDE.md` when it could not reach it.

This repository has **no automated "the deleted identifiers are gone" census at
all**, and this review does not claim one. `tc_028`'s document census checks for
obsolete *policy prescriptions* in markdown; it is a different check and is not
represented as this one. The removal sweep here is manual, and its reach was
**proved rather than assumed** in both directions it could have been wrong:

- **It sees extensionless files.** No extension filter is applied, so `Makefile`
  and `Cargo.toml` are in scope. Confirmed positively by grepping a string the
  Makefile does contain: `assurance-chain` → `CLAUDE.md`, `Makefile`,
  `scripts/assurance_chain.py`. A `compat-view` reintroduction would live in
  exactly that file, and a sibling repository was caught by a census that
  filtered on extension and therefore never scanned it.
- **It sees untracked files.** A first pass used `git ls-files` alone, which
  cannot see them — and untracked is precisely the state a reintroduced reader
  is in while someone is still writing it, before any `git add`. The sweep now
  enumerates tracked **plus untracked-not-ignored**
  (`git ls-files -z; git ls-files -z --others --exclude-standard`).

The sweep is an allow-nothing enumeration piped to `grep -Fl`: it applies no
extension filter and no name allow-list, so the three holes that have bitten
sibling repositories cannot exist in it by construction. Construction is not
proof, so each was probed. Every planted file was removed afterwards and the
tree left clean.

| Probe | Hole it tests | Result |
| --- | --- | --- |
| untracked `scripts/reintroduced_reader` (no extension at all) | an allow-list keyed on `Path::extension()`, which is `None` for extensionless files | **found** |
| untracked `scripts/reintroduced_reader.yaml` | `.yml` listed but `.yaml` omitted — and a workflow file may legitimately be `.yaml` | **found** |
| `# probe: … via a compat-view target` appended to the tracked `Makefile` | the single file a `compat-view` reintroduction would live in, and the one an extension filter silently drops | **found** |
| **negative control:** same untracked file, swept with the `--others` loop removed | whether the untracked scan is load-bearing or a silent no-op | **missed it**, so the loop is doing work |

The last row is the one that matters most. A sibling's untracked scan had no
probe at all, and deleting its entire `--others` loop left the test green — a
property silently lost. Here, removing that loop demonstrably stops the sweep
finding a planted reader, so the loop is not decorative.

| Identifier | Result over every tracked file |
| --- | --- |
| `compat-view` | clean |
| `COMPAT_RESULT` | clean |
| `PROOF-legacy-compatibility` | clean |
| `PRESERVE-legacy-bytes` | clean |
| `legacy_evidence_view` | clean |
| `pgm01_compatibility_view` | `assurance/pins.json` `retired_pins` only |
| `legacy-compat` | `assurance/change-assurance.json` amendment note, `schemas/README.md` |
| `historical-pgm01-files` | `spec/program/STD-002` past-tense passage |
| `evidence-correction-v1`, `pgm01-evidence-v1` | `schemas/README.md` deletion section |

Every surviving hit is an explicitly past-tense record of what was removed.
`reviews/**` and `plan/**` are excluded as dated historical documents.

`.github/` holds one workflow, `ci.yml`; there is no `.yaml` file in this
repository, and the sweep would have found one either way since it applies no
extension filter.

### The sealed file, re-read last

`assurance/change-assurance.json` is sealed on every chain run and travels into
the receipt, and this repository's census exempts it, so no gate here can catch
a stale claim in it. A sibling review found two `high` findings of exactly that
shape — statements corrected in the spec, the argument and the README, left
stale in the file that gets sealed. Because this repository held the campaign's
only working PGM-01 mapping, it was the likeliest to carry a substantive claim
about what that mapping demonstrated, so it was re-read in full, last, against
every claim corrected elsewhere.

| Claim in the sealed file | Finding |
| --- | --- |
| `FR-022-AC-4`, `PROOF-legacy-compatibility`, `PRESERVE-legacy-bytes` | already removed; no claim about the compatibility view survives anywhere in the file |
| `purpose` — "the change under issue #39" | **stale.** The file's content was amended by *this* change while still declaring itself solely a statement about the migration. Corrected to name the amendment, its date and its authority. |
| `FR-022-AC-2` — "an empty run is refused" | **incomplete.** The adapter now makes three distinct refusals. Corrected to name all three and to require each to state its own reason. |
| `revision: 1`, `parent_digest: null` | **amended content under an unamended lineage.** Ordinarily revision 2 naming its predecessor's digest — but that digest was computed by Quoin when the prior record was sealed, and this repository deliberately does not commit its evidence store, so it is not retained anywhere this file can read. Declaring revision 2 with a null parent would claim a lineage while omitting the field that makes one checkable. Left at revision 1 with a `revision_note` stating exactly that, so the choice is visible rather than silent. |
| `PRESERVE-manual-ci`, `PRESERVE-runtime-independence`, `PRESERVE-domain-producer` | verified still true |
| `UNKNOWN-ea-acceptance-not-in-a-release` | corrected earlier in this change: it said the two consumed artifacts are byte-identical at tag and main, which stopped being true when the pin moved to `compatibility.py`, whose whole point is that it *differs* |

### Census populations after the deletion

The change removes files that censuses walk, so their floors were re-derived
rather than inherited. A total-only floor is a weak instrument — a directory can
vanish entirely while the total moves less than ordinary churn — so the
populations are given per directory.

Populations were derived from **the code that performs the walk** — its own
directory list `["spec", "plan", "docs", "reviews"]` plus `README.md` and
`CONTRIBUTING.md`, with its `*.md` filter and its `if path.is_dir()` guard —
replayed against `git ls-tree origin/main` and `git ls-files`. They are not
quoted from any prose description of the census, and in particular not from
`schemas/README.md`, whose own account of a freeze was among the things this
change had to correct. The Rust walk uses `read_dir` and so would additionally
count untracked markdown; the working tree is clean at this head, so the two
coincide, and that is why the figures are stated as committed-tree counts.

| Census | Floor | Before | After | Headroom after |
| --- | --- | --- | --- | --- |
| `tc_028` campaign documents (`README`, `CONTRIBUTING`, and `*.md` under `spec/`, `plan/`, `docs/`, `reviews/`) | `> 20` | 99 | 101 | 81 |
| `test_no_verdict_is_read_from_a_console_stream` (`scripts/*.py`) | none | 6 | 5 | n/a |
| chain controls | `>= 4` | 4 | 4 | 0, unchanged by this work |

Per directory, after: `spec` 40, `reviews` 40, `plan` 19, `docs` 0. Before:
`spec` 40, `reviews` 38, `plan` 19, `docs` 0. The delta is exactly the `+2`
review artifacts this change adds.

**Two observations about `tc_028`'s floor, both pre-existing and neither
introduced here — reported rather than fixed, and explicitly not claimed as
something this change's checks cover.**

- A total-only floor cannot catch a directory vanishing, and here the arithmetic
  is stark: the largest directory is `spec` at 40, so `101 − 40 = 61`, still
  three times the floor of 20. **The entire specification directory could stop
  being walked and the floor would not notice.**
- `docs/` is in the declared directory list and **does not exist**; the walk
  guards it with `if path.is_dir()` and it contributes 0. So the list already
  contains one declared-but-unwalked directory today.

**No per-directory guard was added.** A guard built from the same array the walk
iterates cannot catch that array shrinking — deleting an entry removes it from
the walk and the guard together — so a correct one would have to compare against
a separate `const`, or discover directories from `read_dir` minus a skip list.
That is machinery this change was not asked to introduce, and inventing it here
would be a worse outcome than recording the gap with its measurements.

The `scripts/*.py` census lost one file and has no floor. Its population is the
**`.py` files only** — 6 before, 5 after. An earlier draft of this table said
8 → 7, which is the count of *all* files in `scripts/` including
`check_unsafe_comments.sh` and `unsafe_comment_baseline.txt`; that figure is
correct for the sweep discussed above but not for this census's `glob("*.py")`,
and stating it here contradicted this section's own claim that populations were
derived from the code that walks them. It never had one; the
only floor that ever guarded a `scripts/` census (`scripts.len() > 3`, inside the
deleted `TC-024`) guarded a different claim and went with it.

### Every file in `schemas/`, accounted for

The brief counted eight files here, the largest schema set in the campaign. All
eight are dispositioned, not just the five `.schema.json`:

| File | Disposition | Why |
| --- | --- | --- |
| `contract-conformance-manifest-v1.schema.json` | KEPT | live, see table above |
| `contract-conformance-manifest-v1.schema.json.sha256` | KEPT | its sidecar, written and compared by `scripts/generate_conformance_corpus.py` under `make corpus-repro`, inside `make ci` |
| `contract-package-reference-v1.schema.json` | KEPT | live, see table above |
| `contract-package-reference-v1.schema.json.sha256` | KEPT | same sidecar mechanism |
| `derivation-evidence-envelope-v1.schema.json` | KEPT | live validator, see table above |
| `README.md` | KEPT, rewritten | records the deletion and the liveness evidence |
| `pgm01-evidence-v1.schema.json` | DELETED | dead |
| `evidence-correction-v1.schema.json` | DELETED | dead |

### Schema deadness, proved exhaustively rather than by directory sample

Re-run over **every tracked file on `origin/main`** with no extension filter, so
`Makefile`, `Cargo.toml` and every extensionless file were included:

| Schema | Hits on `origin/main` | Where |
| --- | --- | --- |
| `pgm01-evidence-v1.schema.json` | 21 | 10 retained manifests, 7 legacy-compat fixtures, the historical lock, the `TC-024` freeze test, itself, `schemas/README.md` |
| `evidence-correction-v1.schema.json` | 20 | 9 retained manifests, 6 fixtures, `corpus/evidence-corrections/manifest.json`, the lock, the freeze test, itself, `schemas/README.md` |

Every hit except `schemas/README.md` is deleted by this change, and that file is
rewritten to record the deletion. **Zero live consumers, zero hits in `Makefile`
or `Cargo.toml`.**

All **seven** `include_str!`/`include_bytes!` sites in the repository were
enumerated and each points at a markdown specification or a JSON test fixture —
`PGM-01-governance.md`, `STD-002`, `README.md`, `CONTRIBUTING.md`,
`STD-001-diagnostic-registry.md`, and `fixtures/campaign-issue-dispositions-v1.json`.
**None points at any schema.**

### Criteria whose trace tag points at a test that never checks them

A sibling found an acceptance criterion bound by a trace tag to a test that
asserted nothing about it. Every criterion this change leaves behind on a
touched requirement was read against the body of its cited test:

| Criterion | Test | Does the test substantively check it? |
| --- | --- | --- |
| FR-009-AC-1 | TC-006 | Yes — validates the solver fixture and requires its result to stay `inconclusive` |
| FR-009-AC-2 | TC-004 | Yes — asserts `CODEOWNERS` is `* @kreneskyp`, the policy says "Only that human may record sufficiency", and `CONTRIBUTING` says "may not approve its own" |
| FR-009-AC-6 | TC-025 | Yes — asserts the policy names Quoin for retention/audit, ix-flow for the human decision, and both Quire and Quoin as non-executing |
| NFR-004-AC-1..3 | TC-014, TC-020, TC-021 | Yes, each unchanged by this work |
| PGM-01-R09-AC-1 | TC-004 | Yes, as above |
| PGM-01-R09-AC-2 | TC-029, TC-033 | Yes — released-pin classification and the per-state demonstrator check |

No hollow binding found.

**No directory this change *deletes from* is inside `tc_028`'s population.** The
deletions fall in `evidence/` (removed entirely), `tests/fixtures/legacy-compat/`
(entirely), `corpus/evidence-corrections/` (entirely), `schemas/` (5 → 3),
`scripts/` (8 → 7) and `tests/` (one `.rs` file) — none of which `tc_028` walks.
Its population **rose** by the two review artifacts this change adds, so its
floor is further from being threatened than before, and the per-directory
question does not arise for it.

The `scripts/*.py` census lost one file and has no floor. Its population is the
**`.py` files only** — 6 before, 5 after. An earlier draft of this table said
8 → 7, which is the count of *all* files in `scripts/` including
`check_unsafe_comments.sh` and `unsafe_comment_baseline.txt`; that figure is
correct for the sweep discussed above but not for this census's `glob("*.py")`,
and stating it here contradicted this section's own claim that populations were
derived from the code that walks them. It never had one; the
only floor that ever guarded a `scripts/` census (`scripts.len() > 3`, inside the
deleted `TC-024`) guarded a different claim and went with it. Recorded here with
the measured count rather than fixed, because adding a floor is machinery this
change was not asked to introduce.

### Empty-population and trace-binding checks

- **No empty-population clause was created.** The one that existed — `TC-024`'s
  assertion that no script references either frozen schema — was deleted with
  its subject rather than left asserting over nothing. `LOST_STATES` is asserted
  non-empty precisely so it cannot decay into one.
- **No comment created a spurious trace binding.** The static export's
  `unmatched_tags` fell from 12 to 10; the two removed are exactly the deleted
  `tests/evidence_corrections.rs` entries naming `FR-009` and `NFR-004`. **No
  new unmatched tag was introduced**, so the explanatory comment blocks added
  near tagged tests were not read as bindings.

## Mutation probe and control accounting

| Surface | Before | After | Why |
| --- | --- | --- | --- |
| `validate_governance.py --mutation-probes` | 7 | 7 | unaffected; domain governance lane |
| `pgm01_compatibility_view.py --mutation-probes` | 6 | 0 | all six mutated the deleted census's own mapper |
| adapter probes | 3 | 4 | added the `malformed` refusal |
| chain controls | 4 | 4 | unaffected |
| **total mutation probes** | **13** | **7** | −6, all guarding removed material |

`make compat-view` ran two things, the census and its six probes, so deleting
the target removed a mutation gate from every `make ci` run. That is stated
rather than left implicit. None of the six covered a check that still exists.

## Schemas: proved dead before removal

Each of the five schemas was grepped by filename across `src/`, `scripts/`,
`tests/`, `corpus/`, `assurance/` and `spec/`, and separately for `include_str!`
and `include_bytes!`. Filename was not treated as evidence of anything.

| Schema | Verdict | Evidence |
| --- | --- | --- |
| `contract-conformance-manifest-v1.schema.json` | **KEPT — live** | `src/conformance.rs:25` names its `$id`; `tests/conformance.rs:124,130,174,656` reads its bytes; named by `corpus/contract-v0.1/manifest.json` and regenerated by `scripts/generate_conformance_corpus.py` |
| `contract-package-reference-v1.schema.json` | **KEPT — live** | `src/conformance.rs:23`; `tests/conformance.rs:120,209,408,425`; `corpus/contract-v0.1/manifest.json` |
| `derivation-evidence-envelope-v1.schema.json` | **KEPT — live** | `scripts/validate_governance.py:21` loads it as `SCHEMA_PATH` and validates all 13 `corpus/governance/` fixtures against it on every `make governance`, which `make test` and `make ci` both run. Its name contains "evidence" and it has nothing to do with the retained tree; deleting it on the strength of its filename would have left the governance corpus unvalidated. |
| `pgm01-evidence-v1.schema.json` | DELETED — dead | every hit was inside the retained records, the legacy-compat fixtures, or the TC-024 freeze test, all deleted in the same change |
| `evidence-correction-v1.schema.json` | DELETED — dead | same three, plus `corpus/evidence-corrections/manifest.json`, which declared it as its `"schema"` with no runner behind it; the one apparent consumer, `tests/evidence_corrections.rs`, hand-asserted fields and never loaded the schema |

## `assurance/pins.json`

Two consumed artifacts — `verification_semantics.py` and
`schemas/pgm01-compatibility-view-v1.schema.json` — were pinned by digest solely
because the deleted reader consumed them. Leaving them would have made "the
digests of the artifacts it actually reads" false and left
`artifact_digest_mismatches` checking a file no gate opens.

**Chosen disposition: replaced with a live pin**, matching what `quire-analyze`
did, rather than removed with the vacuity recorded as `tl-parse` did. The
replacement is `engineering_assurance/compatibility.py`, which
`scripts/check_shared_pins.py:193` genuinely imports (`accepted`,
`classify_all`, `load_matrix`) and delegates every version verdict to.

The pinned digest `62829251…` is the file **as the v0.2.0 released tag carries
it**, not as `main` carries it — the two differ, because the human-acceptance
predicate landed on `main` after the tag. Pinning the tag's bytes is the point;
a pin that matched `main` would be a pin on a branch head, which FR-012-AC-1
forbids.

Probed red: appending one byte to the installed `compatibility.py` turns the
pin gate to exit 1 with `mismatch compatibility.py: b3e8cc8f… pinned
62829251…`. Restored to green afterwards.

The retired pins are recorded in a `retired_pins` field rather than silently
dropped.

## Findings

Raised by an independent adversarial review commissioned before merge.

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-301 | medium | `.gitattributes` asserted "Retained evidence bytes are immutable" over a deleted path | .gitattributes | correct-requirement-no-evidence |
| FND-302 | medium | The `inconclusive` demonstrator checked schema validity, not the fixture's status | tests/test_shared_assurance.py | implementation-bug-despite-evidence |
| FND-303 | medium | Three adapter refusals all exit 1 and nothing asserted they are distinguishable | tests/test_shared_assurance.py | missing-requirement |
| FND-304 | low | `AA-001` still assumed retained tool and environment identities | spec/assurance/AA-001-contract-ir-v01.md | correct-requirement-no-evidence |
| FND-305 | low | `schemas/README.md` omitted the `corpus/` hit from its own grep enumeration | schemas/README.md | correct-requirement-no-evidence |
| FND-306 | low | FR-022 prose understated the rehoming; the two states had been unbacked literals | spec/functional/FR-022-shared-assurance-intake.md | wrong-requirement |
| FND-307 | low | The live `derivation-evidence-envelope-v1.schema.json` lost its only byte lock | schemas/README.md | correct-requirement-no-evidence |
| FND-308 | low | `README.md` said pins.json records "digests" plural when only one is pinned | README.md | correct-requirement-no-evidence |
| FND-309 | high | The `malformed` demonstrator written to fill the gap was falsifiable only by accident: corrupting the sole row in its stream meant a dropping adapter and a refusing adapter produced the same non-zero exit | scripts/assurance_chain.py | implementation-bug-despite-evidence |
| FND-310 | medium | `FR-022-AC-2` was corrected in the sealed record but left understated in the requirement of record and its matrix row — the sibling's `high` class in mirror image | spec/functional/FR-022-shared-assurance-intake.md | correct-requirement-no-evidence |
| FND-311 | medium | `test_a_lost_state_stays_declared_lost…` compared two hand-maintained tables and never asked whether a home had started demonstrating a lost state; the scenario its own comment promised to catch left it green | tests/test_shared_assurance.py | implementation-bug-despite-evidence |
| FND-312 | low | The `scripts/*.py` census population was stated as 8 → 7, which counts every file in `scripts/`; the census globs `*.py` and is 6 → 5 | reviews/SR-032-drop-legacy-evidence-code-review.md | correct-requirement-no-evidence |
| FND-313 | low | `unsupported` and `malformed` are told apart only by quoin's prose stderr, and the console-stream census walks `scripts/*.py` so it cannot see the pattern now living in `tests/` | tests/test_shared_assurance.py | missing-requirement |
| FND-314 | low | `states_demonstrated` was not filtered by `matched`, unlike `adapter_probes`, so a scenario producing the wrong outcome still contributed its state name | scripts/assurance_chain.py | implementation-bug-despite-evidence |
| FND-309 | high | The `malformed` demonstrator written to fill the gap was falsifiable only by accident: corrupting the sole row in its stream meant a dropping adapter and a refusing adapter produced the same non-zero exit | scripts/assurance_chain.py | implementation-bug-despite-evidence |

## Disposition of every finding

| ID | Disposition | Detail |
| --- | --- | --- |
| FND-301 | **FIXED** | File deleted. Its entire content was a comment asserting a preservation claim this change released, plus a whitespace rule for `evidence/pgm-01-02568b1/**`. |
| FND-302 | **FIXED** | `result.status` is an open enum in the schema, so validity would not have caught a fixture edited to a conclusive status — and the comment above the check said exactly that while the code did the opposite. The fixture's bytes are now read and `result.status == "inconclusive"` asserted. |
| FND-303 | **FIXED** | `test_the_adapter_refusals_stay_distinguishable_from_one_another` compares the refusal detail each probe records. Probed red by pointing the malformed stream at the foreign-protocol bytes: fails with `adapter refusals are indistinguishable … expected 3, got 2`. |
| FND-309 | **FIXED** | The `malformed` demonstrator's first form corrupted the only row in its stream, so an adapter that dropped undecodable rows rather than refusing them would have exited non-zero for an empty stream and been credited with a refusal it never made. Rebuilt on the real 99-row stream with one row truncated and the rest intact, and both defects it now names were introduced and observed red. See "The `malformed` demonstrator, and why it is not a tautology". |
| FND-304 | **FIXED** | The assumption now names the Quoin evidence store as where tool and environment identities are obtainable, and states this repository retains none for the pre-stable phase. |
| FND-305 | **FIXED** | `corpus/evidence-corrections/manifest.json` is listed as the fourth hit. The conclusion was unchanged — it is deleted in the same change — but the sentence claimed a grep it then under-reported. |
| FND-306 | **FIXED** | Both the specification and the test comment now state that `unsupported` and `malformed` were unconditional literals bound to nothing, and that binding them to matched probes is a strengthening rather than a like-for-like move. |
| FND-307 | **ACCEPTED** | The seven `validate_governance.py` mutation probes freeze the schema behaviourally on every `make governance`, which catches an edit that matters rather than merely an edit. Recorded in `schemas/README.md` so the missing sidecar is deliberate and visible. Adding one would introduce machinery this change was not asked for. |
| FND-308 | **FIXED** | Singular, and the deliberately un-pinned compatibility matrix is named as the exception. |
| FND-310 | **FIXED** | `FR-022-AC-2`, its behavior bullet, and the `TC-030` matrix row now name all three refusals and require each to state its own reason. The sealed file and the requirement of record agree again; a mechanical diff of all five criteria confirms AC-1/3/5/6 already matched byte-for-byte. |
| FND-311 | **FIXED** | Both checks now read one shared `demonstrated_states()` map, so the lost-state check and the required-state check cannot disagree. Probed with the reviewer's exact mutation — inject `suspect` into a home leaving both tables untouched — which now fails with *"a state declared lost is being demonstrated after all: ['suspect']"*. The three assertions the old version did catch were kept, not replaced, and each re-probed red: `suspect` in both tables, `LOST_STATES` emptied, and a blank reason. |
| FND-312 | **FIXED** | Corrected to 6 → 5, with a note distinguishing the census's `glob("*.py")` population from the all-files count used elsewhere in this review. |
| FND-313 | **ACCEPTED** | Quoin's `evidence record` emits no structured error on refusal, so the only discriminator available is its stderr text. Recorded rather than papered over, and the assertion was tightened from `"is not json" in detail or "json" in detail` — whose second disjunct subsumed the first, making it merely "mentions json" — to `"is not json" in detail`. The residual coupling to quoin's message wording is real and is stated here rather than claimed away; a structured refusal code is upstream work. |
| FND-314 | **FIXED** | Filtered by `matched`, matching the `adapter_probes` treatment. Probed red by excluding `stale` from the filtered set: *"states with no demonstrated case: ['stale']"*. The previous behaviour was fail-closed only indirectly, via an unmatched scenario driving the whole chain red, which made the field weaker than its readers assumed. |

## Assurance Context

**Claim boundary.** That nothing in this repository still needs the deleted
material, and that no claim resting on it survives in a weakened form. It does
**not** claim the deleted records were unnecessary, that they failed, or that
their absence is costless: `suspect` lost its only demonstrator and six mutation
probes went with the census, both recorded above.

**Authoritative policy.** `engineering-assurance#7`, section "Preservation
constraint released for the pre-stable phase", recording the repository owner's
decision of 2026-09-02. An agent transcribed it; the agent did not make it. The
epic's completion criterion and mandatory control were amended before this work.

**Trust inputs.** Engineering Assurance v0.2.0 at its released tag, digest-pinned
on `compatibility.py`; the accepted compatibility matrix packaged with it, which
remains the sole authority on component versions.

**Failure posture.** Fail closed. A drifted consumed artifact digest is a
mismatch, not a read-past. A state with no demonstrator names itself and turns
the gate red. A lost state stays declared lost and cannot be satisfied by
another state's outcome.

**Execution boundary.** Unchanged. Producers run natively in
`make assurance-inputs`. Quire exports static facts; Quoin transcribes, retains
and audits bytes it is handed. Neither executes a producer. The deleted reader
was never an executor.

**Retained-output identity.** This repository now retains no evidence of its own
for the pre-stable phase. A run recorded by `make assurance-record` is retained
by whatever runs it, which is a deployment decision. The constraint re-applies
unchanged at the move toward stable releases.

## Constraints observed

- No hosted CI dispatched; every workflow remains `workflow_dispatch` only and
  `.github/` is untouched by this change.
- Nothing published, tagged, or released.
- No repository setting changed.
- The Make execution-control guard was not re-added.
- No repository other than this one was touched.
- No record was rewritten, backdated, or re-sealed. Deletion only.
