---
id: PLAN-006
title: "Accepted source-profile decision and mapping qualification allocation"
type: Plan
status: done
relationships:
  - target: ix://agent-ix/quire-contract-ir/ADR-0053
    type: references
---

# Issue #53: accepted source-profile disposition

This is the specification-only disposition companion to
[ADR-0053](../spec/decisions/ADR-0053-formal-clause-source-profiles.md).
The owner decision is complete: native Quire is the sole editable source and
the external notations are output mappings. The historical discriminating
corpus remains useful input to those mapping tickets, but it is not a pending
external-parser gate for issue #53.

## Completed decision allocation

- [x] Select native Quire (`ix:native`) as the sole editable source authority.
- [x] Separate language/profile meaning from Contract-IR and backend capability.
- [x] Preserve the bounded admission, source, anchor, reference and resource
  constraints without giving the first-party frontend an exemption.
- [x] Reclassify OCL 2.4, SysML v2/KerML and FRETish as Rust-emitted FS06 mapping
  targets with explicit preserved/conditional/unrepresented/refused results.
- [x] Retain ConfigVersion's minimum-only bound and parent relationship as an
  unresolved model/reference gate; do not approximate either.
- [x] Allocate coverage-state work to #58 and mapping implementation to #55–#57.

Native implementation and mapping qualification remain in their owning tickets;
they are not incomplete subtasks of this completed decision plan.

## Input provenance

Read in full: `/home/peter/dev_bak/filament-research/sysml-v2-integration-options.md`
(SHA-256 `a64565419dd69b5f40cb5ecf84e0491cd1d98f580f2fc64c6750f6b0e01965d0`)
and `fret-vs-ears.md`
(`556db39f726a45d53ab09b3e9e089235b8f0f41bec1cf75ae679fa5170e1d8af`).
The final TypeSpec addendum supersedes their earlier structural-authority
recommendation. The owner's #52 comment 5532708641 says this research is not a
language decision and makes compiled domain packages from filament-core-data
#36 the type-environment source. The actual FR-006 source, revision, digest,
and limits of the three synthetic probes are recorded in ADR-0053.

Upstream documentation and pinned license/README/CLI materials were inspected
on 2026-09-06 using read-only requests. Some browser fetches of pinned GitHub
files failed; successful GitHub content API reads supplied their actual bytes.
The pinned SysML license correction is a concrete finding: stable 2025-09 is
LGPL-3.0-or-later per its README, not the unqualified EPL-2.0 assumption.

## Preserved mapping discriminators

The table below is the original candidate corpus. Read OCL, FRETish and SysML
rows as output-mapping preservation/refusal cases, never as authorization for
another source frontend or foreign-runtime test dependency.

| Pair/control | Expected distinction | Owning protected construct |
|---|---|---|
| P1 version 0 / version 1 | False / true over the declared synthetic [0,10000] domain | FR-013 numeric bounds; FR-014 comparison |
| P2 pre=4,post=5 / pre=4,post=4 | False / true; no full-immutability claim | FR-012 post anchor; FR-014 state observation |
| P3 response at tick 3 / first response tick 4 | True / false in a complete [0,3] native window | Native temporal contract plus #57 FRETish mapping, not a new FR-014 operator |
| P3 short trace / complete trace | Incomplete / evaluated; FRET `LAST` behavior remains a mapping-loss discriminator | Native temporal completeness and #57 mapping contract |
| Same P1 text with unbounded int / explicit finite type | Refuse / typeable | FR-013 bounds |
| Wrong operation or absent anchor / exact operation and anchor | Refuse / bindable | FR-012 anchor; FR-014 observations |
| Post value guarded by pre-state fact / matching observed subject | No cross-observation proof; any partial acceptance requires an explicit later correspondence rule | FR-015 exact-subject guard facts |
| Optional parent UUID navigation / scalar UUID equality | Refuse navigation / eligible scalar comparison; never recursive record or object substitution | FR-013 record cycles and closed values; FR-014 field access |
| Nullable and absent collapsed / separately qualified representation | Refuse ambiguous collapse / eligible only after representation qualification | FR-013 option |
| Possible zero divisor behind implication / declaration excludes zero | Refuse guarded partial expression / eligible rational arithmetic with range proof | FR-014 total Boolean; FR-015 nonzero/range |
| OCL Integer division / independently bounded rational division | Refuse unsupported result/rounding mapping / eligible with all obligations | FR-014 numeric type equality |
| Saturating integer / reject-overflow bounded integer | Refuse changed arithmetic meaning / eligible checked arithmetic | FR-013 overflow policy; FR-015 checked range |
| Set or Bag / explicitly bounded Sequence | Refuse collection-kind erasure / eligible supported iterators | FR-013 collection; FR-016 ordered items |
| Empty Sequence forAll / exists | True / false with defined predicate and no invented element | FR-014 finite quantifier |
| Sequence at(1) / size() | Refuse unqualified one-based indexing / eligible length | FR-014 index/length type |
| Ambient library import / exact adapter-owned compiled closure | Refuse / eligible binding; tool acceptance alone is insufficient | FR-012 dependency identity; FR-016 semantic identity |
| Missing primitive declaration or duplicate authority / exact one source | Refuse / eligible binding | FR-012 exact references; FR-013 declaration namespace |
| Valid Boolean / null, invalid or Option<Boolean> root | Eligible / refuse | FR-014 root type; FR-015 definedness |
| At-limit / one-past-limit source bytes, nesting, nodes and collection sizes | Admit normal validation / bounded resource refusal, no partial result | FR-014 and FR-019 limits |
| One package split into many clauses | Aggregate limit unchanged, no per-clause budget multiplication | FR-019 complete-input bound |
| Nested quantifier product 4096 / 4097 | Eligible / frontend cost refusal, subject to independently checked visit budget | Proposed stricter profile bound; FR-014 finite quantifier |
| Same semantics with reordered declaration source / semantic field or profile change | Canonical identity stable for order-only change / semantic or derivation identity changes as appropriate | FR-016 canonical bytes; profile provenance |
| One authored native implication mapped to OCL total or/not / zero IR Implication nodes | Source implication-vacuity unavailable/deferred, never a vacuity pass inferred from zero probes | FR-014 operator identity; source/IR/codegen correspondence |

These are preserved candidate controls, not executable fixtures and not a
statement that all expected-positive examples pass the existing IR or any
output mapping. Each owning implementation ticket must select the applicable
native source case, expected mapping/loss result and independent oracle before
claiming support.

## Accepted allocation and remaining external work

1. Quire Specification FS01 freezes the native charter/profile decision and
   preserves historical definition identities; FS02–FS05 own their named model,
   semantic and interchange contracts.
2. ADR-0054 remains accepted and #54 remains closed. A later reference/graph
   implementation uses a separately versioned identity-preserving contract;
   no UUID cast or recursive-record substitution is permitted.
3. #55–#57 specify and implement Rust output mappings. They retain the useful
   candidate pins as research provenance but do not execute foreign runtimes for
   first-party qualification without a new owner decision.
4. Independent mapping corpora compare native source meaning with the emitted
   artifact and its loss report. Parser acceptance is never the oracle.
5. #58 reports every criterion's formalization disposition and exact clause
   address using the native source authority and current PGM-01 result boundary.

No current release/test-matrix row is promoted by this packet. Direction
approval and technical fixture success remain separate from human evidence
sufficiency and release approval under PGM-01.

## Cross-lane review disposition

The coordinator identified that total-or lowering erases the IR Implication
nodes used by the current codegen vacuity census. ADR-0053 now explicitly
withholds source-level implication-vacuity claims under that lowering. A later
source-to-IR/probe correspondence, or independently reviewed preservation of
IR Implication for pure independently defined operands, needs an independent
source population and retained source-clause/profile provenance. No semantics
were changed to obtain probes; source-profile admission still forbids importing
antecedent guard facts into a partial operand. This review finding is addressed
as a named qualification gap, not presented as implemented vacuity support.
