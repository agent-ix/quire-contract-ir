// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! QSpec TC-233's published nominal preimages stay at their exact identities.

use super::{fixtures_dir, proposal_dir_of, read_json};
use ix_trace_rs::trace;
use quire_contract_model::NominalIdentityPreimage;

/// Trace: FR-063-AC-3
#[trace("TC-233", "FR-063-AC-3")]
#[test]
fn tc_233_dimension_equivalence_vectors_have_the_published_keys_and_scales() {
    let path = proposal_dir_of(&fixtures_dir()).join("node-identity-vectors.json");
    let vectors = read_json(&path).expect("QSpec node identity vectors");
    let rows = vectors["vectors"].as_array().expect("vectors array");
    let cases = [
        (
            "dimension-equivalence-rpc-flow-time",
            "1",
            "1",
            "bbf770ea312108434ee12d0ca4448c6d3d53a0004484b93b8ec577ff1ede00d4",
        ),
        (
            "dimension-equivalence-rpc-clock-time",
            "1000",
            "1",
            "d707eb948ab055a64cf90b8009997ebc54c9bfe5361479afb6485c9c2df56c24",
        ),
        (
            "dimension-equivalence-flow-clock-time",
            "1000",
            "1",
            "584c6bbcb5056cf9dd5d10504339e49c5507348601530e5ef6d67bc5e351714b",
        ),
    ];
    for (name, numerator, denominator, expected_key) in cases {
        let row = rows
            .iter()
            .find(|row| row["name"] == name)
            .expect("published case");
        assert_eq!(row["sha256"], expected_key);
        let preimage: NominalIdentityPreimage =
            serde_json::from_value(row["preimage"].clone()).expect("typed published preimage");
        let NominalIdentityPreimage::DimensionEquivalence(edge) = &preimage else {
            panic!("published equivalence variant: {name}")
        };
        assert_eq!(edge.root_scale.numerator.as_ref(), numerator);
        assert_eq!(edge.root_scale.denominator.as_ref(), denominator);
        assert_eq!(
            preimage.digest(u64::MAX).expect("canonical key"),
            expected_key
        );
    }
    for (name, expected_key) in [
        (
            "model-dimension-acme-rpc-time",
            "57798558b4187c759907231c80c74ae4d3857b5b4c3bfd4fed20f146dab409e7",
        ),
        (
            "model-dimension-acme-flow-time",
            "a005567d213eebe2b5afcea6571eed60595e02a2138cd8e21b9c0310efbf8a7c",
        ),
        (
            "model-dimension-acme-clock-time",
            "d6728197a52375cb5e131e7d439f053e1b622950b4875a09b31d67ab46012c46",
        ),
    ] {
        let row = rows
            .iter()
            .find(|row| row["name"] == name)
            .expect("published dimension");
        assert_eq!(row["sha256"], expected_key);
        let preimage: NominalIdentityPreimage =
            serde_json::from_value(row["preimage"].clone()).expect("typed dimension");
        assert_eq!(
            preimage.digest(u64::MAX).expect("canonical key"),
            expected_key
        );
    }
}
