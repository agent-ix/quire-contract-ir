---
id: TASK-003
title: "Bind PGM-01 tests and local gates"
type: Task
status: done
track: A
priority: P0
relationships:
  - target: ix://agent-ix/quire-contract-ir/TASK-002
    type: depends_on
---
# TASK-003: Bind PGM-01 tests and local gates

Use the published schema as the only conformance engine, declare the Python
lane, run schema mutation probes, and bind every matrix row to a tracing-tagged
test symbol. Keep remote Actions disabled and undispatched.
