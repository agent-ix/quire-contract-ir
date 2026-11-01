// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX
//! Public serde decoding of terms containing flat wire node IDs.

use ix_trace_rs::trace;
use quire_contract_model::{CheckedOperationViewError, CheckedTerm};
use serde_json::{json, Value};

fn node() -> Value {
    json!({"package":"fixture","path":["expression",{"node":7}],"ordinal":2})
}

#[trace("TC-048", "IR-1208")]
#[test]
fn public_term_deserialization_retains_node_ids_from_text_and_buffered_values() {
    let reference = json!({"term":"reference","target":node()});
    let literal = json!({"term":"literal","type":node(),"value_kind":"boolean","value":true});
    let cases = [
        reference.clone(),
        literal.clone(),
        json!({"term":"dependency_reference","package":"dependency","node":node()}),
        json!({"term":"binding","name":"x","value":reference}),
        json!({"term":"aggregate","members":[literal,reference]}),
        json!({"term":"application","operator":"binary",
            "operation":{"identity":"quire.op.test","laws":[],"mode":null,"member":null,"leaves":[]},
            "result_type":node(),"arguments":[reference]}),
        json!({"term":"frame","modifies":[{"kind":"relationship","declaration":node()}],
            "creates":[node()],"deletes":[],"deletes_scoped":[]}),
    ];
    for wire in cases {
        let buffered: CheckedTerm = serde_json::from_value(wire.clone()).expect("buffered term decodes");
        let text: CheckedTerm = serde_json::from_str(&wire.to_string()).expect("text term decodes");
        assert_eq!(text, buffered);
        assert_eq!(serde_json::to_value(&buffered).unwrap(), wire);
        if let CheckedTerm::Application { operation, .. } = buffered {
            assert_eq!(operation.projection_field(), Err(CheckedOperationViewError::NotAdmitted));
        }
    }
}

#[trace("TC-048", "IR-1208")]
#[test]
fn public_term_deserialization_rejects_malformed_node_ids_and_unknown_members() {
    for id in [
        json!({"package":"fixture","path":[{"node":-1}],"ordinal":0}),
        json!({"package":"fixture","path":[{"node":9_007_199_254_740_992_u64}],"ordinal":0}),
        json!({"package":"fixture","path":[{"node":0,"extra":true}],"ordinal":0}),
        json!({"package":"fixture","path":[{"package":"nested","path":["x"],"ordinal":0}],"ordinal":0}),
        json!({"package":"fixture","path":["x"],"ordinal":"0"}),
        json!({"package":"fixture","path":["x"],"ordinal":0,"extra":true}),
    ] {
        let wire = json!({"term":"reference","target":id});
        assert!(serde_json::from_value::<CheckedTerm>(wire.clone()).is_err(), "{wire}");
        assert!(serde_json::from_str::<CheckedTerm>(&wire.to_string()).is_err(), "{wire}");
    }
    let unknown = json!({"term":"reference","target":node(),"extra":true});
    assert!(serde_json::from_value::<CheckedTerm>(unknown).is_err());
    for text in [
        r#"{"term":"reference","target":{"package":"p","path":["x"],"ordinal":0,"ordinal":1}}"#,
        r#"{"term":"reference","target":{"package":"p","path":[{"node":0,"node":1}],"ordinal":0}}"#,
    ] {
        assert!(serde_json::from_str::<CheckedTerm>(text).is_err());
    }
}
