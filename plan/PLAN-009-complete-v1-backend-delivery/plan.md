---
id: PLAN-009
title: "Deliver the complete-V1 exact Contract IR and backend chain"
type: Plan
status: active
relationships:
  - target: ix://agent-ix/quire-contract-ir/issues/99
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-035
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-036
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-037
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: references
  - target: ix://agent-ix/quire-contract-ir/issues/106
    type: references
---
# PLAN-009: Deliver the complete-V1 exact Contract IR and backend chain

## Source Authority

QSpec revision `8d0fbad65d0fef84b8685ad6eec1b8884980cd76` is the accepted
semantic source for AD-004, AD-010, FR-195 through FR-197, I12, I13, I16, I19,
and TC-217 through TC-219. This plan does not amend Quire grammar, model, or
protocol authority. Closed Contract IR #69/#82–#86 and #95 are retained as
evidence for their exact bounded profiles and common output seam; they are not
reopened or generalized by implication.

## Dependency DAG

```text
#99 / FR-035..037 / AD-003 (specified and reviewed)
  -> #100 Contract IR complete node families and exact lowering
    -> runtime #16 exact typed oracle operators
      -> codegen #48 deterministic complete-V1 oracle generation
        -> codegen #49 complete bounded Kani generation
          -> codegen #50 canonical replay and parity
            -> #55 OCL, #56 SysML/KerML, #57 FRETish
              -> #58 formalization coverage
                -> #40 cross-repository closeout
                  -> #101 cross-backend qualification
```

No consumer may start an implementation interface before its direct producer
has merged its reviewed contract. Each ticket runs its own scoped spec cycle if
it introduces a new representation, correspondence, or compatibility promise.

## Capability Allocation and Disposition

| Capability | Contract / interface | Producer | Consumer / ticket | Current disposition |
| --- | --- | --- | --- | --- |
| V1-BACK-001 | QSpec FR-031, I12/I13/I19 | Contract IR #100 | runtime #16, codegen #48 | planned exact lowering; no new artifact in #99 |
| V1-BACK-002 | QSpec FR-031, I12/I13/I19 | Contract IR #100 | runtime #16, codegen #48 | planned exact lowering; no new artifact in #99 |
| V1-BACK-003 | QSpec FR-030, I12/I13/I19 | Contract IR #100 | runtime #16, codegen #48 | planned exact lowering; no new artifact in #99 |
| V1-BACK-004 | QSpec FR-195, I12/I13/I19 | Contract IR #100 | runtime #16, codegen #48 | planned exact lowering; refusal is per item |
| V1-BACK-005 | QSpec FR-196, I13/I19 | codegen #49 | codegen #50 | requires model bound or exact encoding; no approximate harness |
| V1-BACK-006 | QSpec FR-197, I13/I19 | codegen #50 | runtime #16 | canonical replay only; mismatch is parity failure |
| V1-BACK-007 | QSpec FR-120, I12/I16 | #55 OCL | #58/#101 | output only; preservation/conditional/unrepresented/refused record |
| V1-BACK-008 | QSpec FR-120, I12/I16 | #56 SysML/KerML | #58/#101 | output only; preservation/conditional/unrepresented/refused record |
| V1-BACK-009 | QSpec FR-120, I12/I16 | #57 FRETish | #58/#101 | output only; preservation/conditional/unrepresented/refused record |
| V1-BACK-010 | QSpec FR-125, I12/I16 | #55/#56/#57 | #58/#101 | output only; target parser acceptance is not equivalence |
| V1-BACK-011 | QSpec FR-121, I12/I16 | #55/#56/#57 | #58/#101 | explicit source-bound loss accounting |
| V1-BACK-012 | QSpec FR-019, I12/I16 | #55/#56/#57 | #58/#101 | deterministic derived package; never source authority |
| V1-BACK-013 | QSpec FR-052, I12/I16 | #55/#56/#57 | #58/#101 | exact pins and output-map qualification |
| V1-BACK-014 | QSpec FR-019, I12/I13/I19 | Contract IR #100 | runtime #16, codegen #48–#50 | identity-preserving cross-backend contract |

`planned` records incomplete delivery, never implementation support. At runtime
and provider boundaries, every non-representable item must instead receive the
terminal disposition named by FR-035 or FR-036 and no artifact.

## Work Packages

| Order | Ticket | Owning repository | Required evidence before successor |
| --- | --- | --- | --- |
| E00 | Contract IR #99 | quire-contract-ir | AD-003, FR-035–037, TC-044–046, this plan, accepted self review |
| E01 | Contract IR #100 | quire-contract-ir | ✅ merged #104 (`1baa5af`): strict I04 reader, complete node-family lowering, independent per-item records and TC-044 evidence |
| E01b | Contract IR issue #106 (PR #107) | quire-contract-ir | FR-038 and TC-047–050: version dispatch, strict CheckedPackage V2 reader with nominal identity re-derivation, typed V1-to-V2 MigrationOutcome and per-item V2 lowering against QSpec `5aa00f35`; frozen V1 reader unchanged |
| E02 | runtime #16 | quire-contract-runtime | exact oracle outcomes over the merged ContractPackage |
| E03 | codegen #48 | quire-contract-codegen | deterministic oracle artifacts and accounting |
| E04 | codegen #49 | quire-contract-codegen | exact bounded Kani harnesses and pins |
| E05 | codegen #50 | quire-contract-codegen | canonical replay/parity evidence |
| E06 | #55/#56/#57/#58/#40 -> #101 | Contract IR plus consumers | output records, coverage, exact pins, cross-backend qualification |

## Exit Criteria

- Every V1-BACK row has a single producer, direct consumer, test/corpus control,
  and terminal per-item outcome vocabulary.
- No ticket treats a source, model, bound, artifact, target parse, or sibling
  result mutation as an approximation or an implicit success.
- Generated OCL, SysML/KerML, and FRETish remain outputs only.
- The complete corpus and cross-backend qualification retain unexecuted and
  unavailable work as visible gaps rather than deferrals or success claims.
