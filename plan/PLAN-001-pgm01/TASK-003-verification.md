---
id: TASK-003
title: "Bind PGM-01 tests and local gates"
type: Task
relationships:
  - target: ix://agent-ix/quire-contract-ir/TASK-002
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/TM-001
    type: verifies
---
# TASK-003: Bind PGM-01 tests and local gates

Use the published schema as the only conformance engine, declare the Python
lane, run schema mutation probes, and bind every matrix row to a tracing-tagged
test symbol. Keep remote Actions disabled and undispatched.

