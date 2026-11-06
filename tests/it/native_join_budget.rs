// SPDX-License-Identifier: AGPL-3.0-or-later
//! Shared exact-limit assertion for the six admitted native join families.

use quire_contract_model::{
    CheckedNodeId, ExactTypeProjectionError, ExactTypeProjectionErrorCode, NativeJoinBudget,
    NativeJoinCharge, NativeJoinChargeKind, SelectedExactIdentities,
};

pub(crate) fn assert_exact_join(
    locus: &CheckedNodeId,
    work_locus: &CheckedNodeId,
    mut run: impl FnMut(
        &mut NativeJoinBudget,
    ) -> Result<SelectedExactIdentities, ExactTypeProjectionError>,
) -> (SelectedExactIdentities, NativeJoinCharge) {
    let mut ample = NativeJoinBudget::new(u64::MAX, u64::MAX);
    let expected = run(&mut ample).expect("admitted native join succeeds");
    let required = ample.consumed();
    assert!(required.work > 0, "join must charge work");
    assert!(required.storage > 0, "join must charge storage");
    assert_eq!(ample.last_success_delta(), Some(required));
    assert_eq!(ample.first_unpaid(), None);

    for kind in [NativeJoinChargeKind::Work, NativeJoinChargeKind::Storage] {
        let (work, storage) = match kind {
            NativeJoinChargeKind::Work => (required.work, u64::MAX),
            NativeJoinChargeKind::Storage => (u64::MAX, required.storage),
        };
        let mut exact = NativeJoinBudget::new(work, storage);
        assert_eq!(run(&mut exact), Ok(expected.clone()));
        assert_eq!(exact.consumed(), required);
        assert_eq!(exact.last_success_delta(), Some(required));
        assert_eq!(exact.first_unpaid(), None);

        let (work, storage) = match kind {
            NativeJoinChargeKind::Work => (required.work - 1, u64::MAX),
            NativeJoinChargeKind::Storage => (u64::MAX, required.storage - 1),
        };
        let mut short = NativeJoinBudget::new(work, storage);
        let denied = run(&mut short).expect_err("one below exact allowance refuses");
        assert_eq!(
            denied.code,
            match kind {
                NativeJoinChargeKind::Work => ExactTypeProjectionErrorCode::ExhaustedWork,
                NativeJoinChargeKind::Storage => ExactTypeProjectionErrorCode::ExhaustedStorage,
            }
        );
        assert_eq!(denied.requested.as_deref(), Some(locus));
        assert_eq!(
            denied.at.as_deref(),
            Some(match kind {
                NativeJoinChargeKind::Work => work_locus,
                NativeJoinChargeKind::Storage => locus,
            })
        );
        let attempted = short.first_unpaid().expect("first unpaid charge recorded");
        assert_eq!(attempted.kind, kind);
        assert!(attempted.amount > 0);
        assert_eq!(short.last_success_delta(), None);
        match kind {
            NativeJoinChargeKind::Work => {
                assert!(short.consumed().work <= work);
                assert!(short.consumed().storage <= required.storage);
            }
            NativeJoinChargeKind::Storage => {
                assert!(short.consumed().storage <= storage);
                assert!(short.consumed().work <= required.work);
            }
        }
    }
    (expected, required)
}
