---
id: SR-582
title: "EARS review of CheckedPackage V2 consumption"
type: SpecReview
analysis: ears-conformance
scope: "QCI #106; FR-038 normative statements"
review_set: subset
evaluated_revision: "agent-e/106-checked-package-v2 based on e463103"
review_date: "2026-09-16"
---
# EARS review of CheckedPackage V2 consumption

## Summary

PASS after fixes. `quire validate` reports FR-038 grammar-clean. Each SHALL
statement names its subject (consumer, dispatcher, V2 reader, migrator, V2
lowerer) and, where behavior is conditional, an explicit `When` trigger.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The migration and lowering paragraphs initially began with a noun phrase and no subject (`ears:missing-subject`, `ears:unclassifiable`); rewritten as event-driven statements. | FR-038 Behavior |
| FND-002 | low | The nominal-identity paragraph opened with a prepositional scope instead of a trigger; rewritten as `When a node is … the V2 reader shall`. | FR-038 Behavior |
