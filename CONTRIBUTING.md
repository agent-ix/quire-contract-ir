# Contributing

Contributions are welcome from people using any development method, including
agent-assisted workflows. The standard is the same for every contribution:

- requirements and acceptance criteria are updated before implementation;
- the repository's specification, review, test, and assurance gates pass;
- source and third-party provenance remain truthful and reviewable;
- domain producers retain their native structured result semantics, while
  Quire supplies static definition references and Quoin consumes explicit
  results without invoking producers; and
- the CODEOWNER, `@kreneskyp`, reviews the pull request and exclusively owns
  release decisions.

Do not push directly to `main`. Generated artifacts must retain their declared
derivation metadata and licensing. Do not copy material from repositories or
documents whose license does not permit reuse.

An agent may prepare a candidate and its evidence, but may not approve its own
pull request, decide that evidence is sufficient, create a source-release tag,
or claim validation, accreditation, or certification for a consuming project.
The exact ownership and compatibility rules are normative in
[`PGM-01`](spec/program/PGM-01-governance.md); the linked
[reconciliation record](spec/program/STD-002-shared-assurance-governance.md) explains the
common-work sequence. Quire and Quoin remain non-executing development-time
boundaries, not runtime dependencies or a shared producer runner.

## Quoting removed campaign policy

Campaign documents must not carry a removed prescription as live policy. A
review artifact still has to be able to cite the text it removed, so exactly one
exception applies:

- In `reviews/**` only, a removed prescription may appear inside a quotation —
  a Markdown blockquote line (`>`) or a fenced code block. Quoted text is a
  citation, not policy.
- Anywhere else in `reviews/**`, and anywhere at all in `README.md`,
  `CONTRIBUTING.md`, `spec/`, `plan/`, or `docs/`, a removed prescription is
  rejected however it is written. Quoting does not exempt governed campaign
  content.

TC-028 enforces this rule.
