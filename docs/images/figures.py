# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 Agent-IX
"""Quire Contract IR's documentation figures, rendered by the docs-figures skill.

Regenerate from the repository root with the agent-ix/dev-tools skill:

    python3 -m pip install fonttools
    python3 <dev-tools>/skills/docs-figures/scripts/render.py docs/images/figures.py

Add --check to confirm the committed SVGs are current. The node families and the
lowering outcomes are the ones the model crate declares; the figures state them
here by hand, so change them together with `vocabulary.rs` and `lower.rs`.
"""

from docs_figures import FONTS, SMALL, TITLE, Figure  # noqa: F401

STYLE = "ix-docs"

# One role per kind of thing, fixed across the Quire repositories.
ROLES = [
    ("code", "authored source", "box"),
    ("logic", "compiler stage", "box"),
    ("model", "checked package / IR", "box"),
    ("evidence", "outcome", "box"),
    ("ink", "input", "box"),
]


def hero():
    """README hero: checked-package bytes in, one ContractPackage out, two consumers."""
    f = Figure(
        "hero",
        980,
        330,
        "Quire Contract IR reads the bytes of a quire.checked-package/v2 together with the "
        "evidence they name. The reader admits the package, refuses it, or stops at a limit. An "
        "admitted package is lowered, for the node families a backend supports, into one "
        "quire.contract-package/v1, which code generation and analysis consume.",
    )
    f.box(0, 20, 190, 78, "model", "checked package", "package_id · lock", "semantic_graph", "source_map · …")
    f.box(0, 150, 190, 62, "ink", "evidence", "domain packages", "dependencies")
    f.box(240, 80, 150, 62, "logic", "read", "strict · canonical", "id re-derived")
    f.line([(190, 59), (215, 59), (215, 111), (237, 111)], "model")
    f.line([(190, 178), (215, 178), (215, 111)], "ink", arrow=False)
    f.box(240, 190, 200, 50, "evidence", "refused · incomplete", "typed code, JSON pointer")
    f.line([(315, 142), (315, 187)], "evidence")
    f.box(430, 80, 160, 62, "model", "admitted", "CheckedPackageV2", "immutable")
    f.line([(390, 111), (427, 111)], "logic")
    f.box(630, 80, 144, 62, "logic", "lower", "closure per node", "own work budget")
    f.line([(590, 111), (627, 111)], "model")
    f.box(617, 190, 170, 62, "ink", "lowering profile", "families · bounds", "work budget")
    f.line([(702, 190), (702, 145)], "ink")
    f.box(810, 80, 170, 62, "model", "contract package", "contract-package/v1", "canonical bytes")
    f.line([(774, 111), (807, 111)], "logic")
    f.box(790, 250, 90, 50, "ink", "codegen", "backend")
    f.box(890, 250, 90, 50, "ink", "analyze", "backend")
    f.line([(895, 142), (895, 175), (835, 175), (835, 247)], "model")
    f.line([(895, 175), (935, 175), (935, 247)], "model", arrow=True)
    f.legend(0, 322, [ROLES[2], ROLES[1], ("evidence", "outcome", "box"), ROLES[4]])
    return f


def inside_package():
    """What the checked package holds, and what the model crate reads from it."""
    f = Figure(
        "inside-package",
        980,
        356,
        "A quire.checked-package/v2 holds eight members. The reader checks each one, and lowering "
        "works on the semantic graph, whose nodes come in thirteen families: scalar_type, composite_type, bounded_domain, value, expression, "
        "function, model, relation, state, temporal, protocol, claim and correspondence.",
    )
    members = [
        ("contract_version", "must select v2", "logic"),
        ("identity_preimage", "what the id hashes", "logic"),
        ("package_id", "re-derived and compared", "logic"),
        ("lock", "must not contradict itself", "logic"),
        ("semantic_graph", "typed nodes", "model"),
        ("source_map", "validated, then kept", "logic"),
        ("capability_report", "required ones available", "logic"),
        ("diagnostics", "part of the package", "logic"),
    ]
    f.text(0, 12, "CHECKED PACKAGE MEMBERS", "muted", "mono", SMALL)
    for i, (name, what, role) in enumerate(members):
        y = 24 + i * 38
        f.rect(0, y, 320, 32, role)
        f.text(12, y + 21, name, role, "mono", SMALL)
        f.text(308, y + 21, what, "muted", "mono", SMALL, anchor="end")
    f.text(400, 12, "SEMANTIC GRAPH NODE FAMILIES", "muted", "mono", SMALL)
    families = [
        "scalar_type", "composite_type", "bounded_domain", "value", "expression",
        "function", "model", "relation", "state", "temporal", "protocol", "claim",
        "correspondence",
    ]
    col_w = 180
    for i, name in enumerate(families):
        x = 400 + (i % 3) * (col_w + 10)
        y = 24 + (i // 3) * 38
        f.rect(x, y, col_w, 32, "model")
        f.text(x + 12, y + 21, name, "model", "mono", SMALL)
    f.legend(0, 346, [("logic", "checked by the reader", "box"), ("model", "lowered", "box")])
    return f


def lowering():
    """The five answers lowering gives for one requested node."""
    f = Figure(
        "lowering",
        980,
        340,
        "Lowering answers each requested node on its own with one of five records: lowered, "
        "unsupported when a reachable family is outside the profile, requires_bound when a "
        "reachable type is unbounded, invalid_input when the key is not in the package, or failed "
        "when the work or byte budget runs out. A request that cannot lower adds nothing to the "
        "package.",
    )
    f.box(0, 92, 170, 62, "ink", "requested node", "key in the graph")
    f.box(0, 190, 170, 62, "ink", "lowering profile", "families · bounds", "work budget")
    f.box(214, 110, 160, 56, "logic", "lower", "closure of a node")
    f.line([(170, 123), (191, 123), (191, 138), (211, 138)], "ink")
    f.line([(170, 218), (191, 218), (191, 148), (211, 148)], "ink")
    cases = [
        ("lowered", "node + ir_id", "model", "closure fits the profile"),
        ("unsupported", "first family outside", "evidence", "reachable, not in profile"),
        ("requires_bound", "first unbounded type", "evidence", "profile demands bounds"),
        ("invalid_input", "the requested key", "evidence", "not in the graph"),
        ("failed", "limit and count", "evidence", "work or byte budget"),
    ]
    top, step = 0, 62
    f.line([(374, 138), (400, 138)], "logic", arrow=False)
    f.line([(400, top + 25), (400, top + 4 * step + 25)], "logic", arrow=False)
    for i, (name, names, role, why) in enumerate(cases):
        y = top + i * step
        f.line([(400, y + 25), (430, y + 25)], "logic")
        f.box(433, y, 250, 50, role, name, names)
        f.text(704, y + 30, why, "muted", "mono", SMALL)
    f.legend(0, 332, [("model", "added to the package", "box"), ("evidence", "record only", "box"), ("ink", "input", "box"), ("logic", "stage", "box")])
    return f


REPOS = [
    ("ORCHESTRATION", [("quire-verification", True), ("quire-driver", True), ("quire-integration", True)]),
    ("BACKENDS", [("quire-contract-codegen", False), ("quire-contract-runtime", False),
                  ("quire-analyze", False), ("quire-protocol", True), ("quire-mltl", False)]),
    ("LANGUAGE", [("quire-specification", True), ("quire-spec-language", False), ("quire-contract-ir", False)]),
    ("FOUNDATIONS", [("quire-exact", False), ("quire-semantic-value", False), ("quire-walk", False),
                     ("quire-canonical", False), ("quire-verification-contracts", False),
                     ("quire-observation", False)]),
]


def ecosystem(here="quire-spec-language"):
    """The Quire repositories in bands; each band builds on the ones below."""
    f = Figure(
        "ecosystem",
        980,
        364,
        "The Quire repositories in four bands. Foundations: quire-exact, quire-semantic-value, "
        "quire-walk, quire-canonical, quire-verification-contracts, quire-observation. Language: "
        "quire-specification, quire-spec-language, quire-contract-ir. Backends: "
        "quire-contract-codegen, quire-contract-runtime, quire-analyze, quire-protocol, "
        "quire-mltl. Orchestration: quire-verification, quire-driver, quire-integration.",
    )
    y = 0
    gap = 10
    for band, repos in REPOS:
        f.text(0, y + 12, band, "muted", "mono", SMALL)
        widths = [FONTS["title"].width(name, TITLE) + 24 for name, _ in repos]
        spare = (980 - sum(widths) - gap * (len(repos) - 1)) / len(repos)
        x = 0
        for (name, private), w in zip(repos, widths):
            w = round(w + spare)
            role = "logic" if name == here else "ink"
            f.box(x, y + 20, w, 50, role, name, "private" if private else "public", dashed=private)
            x += w + gap
        y += 88
    f.legend(0, 360, [("logic", "this repository", "box"), ("ink", "private", "dash")])
    return f


def ecosystem_here():
    return ecosystem(here="quire-contract-ir")


ecosystem_here.__name__ = "ecosystem_here"

FIGURES = [hero, inside_package, lowering, ecosystem_here]

