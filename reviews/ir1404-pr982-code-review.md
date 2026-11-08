---
id: SR-202611070951309532
title: PR982 sample proof metadata code-review
type: SpecReview
analysis: code-review
scope: agent-ix/quire-contract-ir; IR-1404; tests/it/support/qspec_read_proof.rs
review_set: subset
---

## Summary

Ticket: IR-1404. Independent review of frozen PR982 at 56cadc32d1c4fdad611356dc3112a1006c69e646 against main d671d43b. Only the test proof harness metadata changes. Both plain and caller-budget rows report mode, existing comparison scope, explicit qualification and false AC512 credit. The caller-budget mode/scope fields are no longer hidden behind test-support. SAMPLE remains Core; FULL remains the existing uncapped all-node oracle. No production API, reader accounting, comparison oracle, ignore, fixture, Cargo dependency, schema or CI workflow changes. No applicable AssuranceProfile was found under spec/. Source remained untouched.

## Verdict

PASS for this metadata-only slice: no introduced defects found. No corpus qualification is granted.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Retained exact native test log shows compilation completed in 54.83s and the actual invalid-JSON control passed (1 passed, 0 failed; 0.06s). Parsed two actual JSON rows: plain and caller-budget, both sample/retained-content/NOT_QUALIFIED with ac_512_credit false and eight input bytes. The orchestration parent exited 143 after an attempted waiter cancellation raced lock acquisition; this is recorded separately and is not called a successful pipeline exit. The bounded child completed successfully in its retained log. Final selected1.98 formatting and diff check passed. No Cargo command or repeated aggregate was run by this reviewer. Hosted CI is independent; the known ar_archive_writer0.5.3 license-policy rejection is not changed or waived here. No whole-read AC511 or static AC513 qualification is inferred.

Selected Quire0.38.1 computed matrix was inspected: 1028 tagged,92 untagged,26 method-without-symbol and8 tagged-by-ignored-test records. These repository-wide existing gaps remain open; this subset review does not assert global assurance PASS. AC512 has20 existing binders; the new metadata control traces TC472 and makes no AC512 completion claim. No criterion, matrix, ignore or trace weakening is introduced. Optional whole-repository semantic review was not requested and was skipped.

Plan completion: not assessed

Full reviewed scope:

```yaml
scope:
- id: qspec_read_proof.rs::finish_proof
  path: tests/it/support/qspec_read_proof.rs
  role: examined
  excerpt: '"qualification":if mode == "sample" { "NOT_QUALIFIED" } else { "FULL_CORPUS_REQUIRED"
    },'
- id: qspec_read_proof.rs::read_budget_and_account
  path: tests/it/support/qspec_read_proof.rs
  role: examined
  excerpt: '"mode":mode,"public_view_equivalence":views.label(),'
- id: qualification_metadata_entries_preserve_the_existing_invalid_json_outcome
  path: tests/it/support/qspec_read_proof.rs
  role: examined
  excerpt: Exercise both log entry producers with the existing invalid-JSON fixture.
- id: FR-038-AC-512
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: context_only
  excerpt: 'Over the TC-048 corpus, every budget whose limits are at least the read''s
    measured requirement returns the same outcome as the unlimited budget: an equal
    admitted package (equal graph, terms, models, selected documents, source map,
    diagnostics and lowering of every node) or an equal refusal or structural `incomplete`
    (code, cause, loci, `document_pointer`). Repeating a read with equal bytes, evidence
    and limits charges equal work and read storage and stops at the same first unpaid
    charge and locus.'
```
