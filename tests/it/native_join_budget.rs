// SPDX-License-Identifier: AGPL-3.0-or-later
//! Shared exact-limit assertion for the six admitted native join families.

use ix_trace_rs::trace;
use quire_contract_model::{
    CheckedNodeId, ExactTypeProjectionError, ExactTypeProjectionErrorCode, NativeJoinBudget,
    NativeJoinCharge, NativeJoinChargeKind, NativeJoinDeniedCharge, NativeJoinLocus,
    NativeJoinLocusSource, SelectedExactIdentities,
};

fn charge_selection(budget: &mut NativeJoinBudget) -> Result<(), NativeJoinDeniedCharge> {
    budget.accounted_charge(
        NativeJoinChargeKind::Work,
        3,
        NativeJoinLocus {
            source: NativeJoinLocusSource::Package,
            form: "driver selection",
        },
    )
}

fn charge_adapter(budget: &mut NativeJoinBudget) -> Result<(), NativeJoinDeniedCharge> {
    budget.accounted_charge(
        NativeJoinChargeKind::Work,
        5,
        NativeJoinLocus {
            source: NativeJoinLocusSource::TupleRow(2),
            form: "adapter row",
        },
    )
}

/// Trace: FR-358-AC-9, FR-357-AC-15.
#[trace("TC-467", "FR-358-AC-9", "FR-357-AC-15")]
#[test]
fn public_call_sites_share_finite_accounting_and_first_denial() {
    let mut exact = NativeJoinBudget::new(8, 0);
    exact.resume();
    assert_eq!(charge_selection(&mut exact), Ok(()));
    assert_eq!(charge_adapter(&mut exact), Ok(()));
    assert_eq!(
        exact.consumed(),
        NativeJoinCharge {
            work: 8,
            storage: 0
        }
    );
    assert_eq!((exact.work_limit(), exact.storage_limit()), (8, 0));
    exact.resume();
    assert_eq!(exact.consumed().work, 8);
    assert_eq!(
        charge_selection(&mut exact),
        Err(NativeJoinDeniedCharge {
            kind: NativeJoinChargeKind::Work,
            amount: 3,
        })
    );
    assert_eq!(exact.consumed().work, 8);
    assert_eq!(
        exact.first_unpaid_locus(),
        Some(NativeJoinLocus {
            source: NativeJoinLocusSource::Package,
            form: "driver selection",
        })
    );

    let mut one_below = NativeJoinBudget::new(7, 0);
    one_below.resume();
    assert_eq!(charge_selection(&mut one_below), Ok(()));
    assert_eq!(
        charge_adapter(&mut one_below),
        Err(NativeJoinDeniedCharge {
            kind: NativeJoinChargeKind::Work,
            amount: 5,
        })
    );
    assert_eq!(
        one_below.consumed(),
        NativeJoinCharge {
            work: 3,
            storage: 0
        }
    );
    assert_eq!(
        one_below.first_unpaid_locus(),
        Some(NativeJoinLocus {
            source: NativeJoinLocusSource::TupleRow(2),
            form: "adapter row",
        })
    );
    assert_eq!(one_below.last_success_delta(), None);
    assert_eq!((one_below.work_limit(), one_below.storage_limit()), (7, 0));
    assert_eq!(
        one_below.accounted_charge(
            NativeJoinChargeKind::Storage,
            1,
            NativeJoinLocus {
                source: NativeJoinLocusSource::Package,
                form: "adapter allocation",
            }
        ),
        Err(NativeJoinDeniedCharge {
            kind: NativeJoinChargeKind::Work,
            amount: 5
        })
    );
    assert_eq!(one_below.consumed().storage, 0);
    assert_eq!(
        one_below.accounted_charge(
            NativeJoinChargeKind::Work,
            1,
            NativeJoinLocus {
                source: NativeJoinLocusSource::Package,
                form: "later selection",
            },
        ),
        Err(NativeJoinDeniedCharge {
            kind: NativeJoinChargeKind::Work,
            amount: 5,
        })
    );
    assert_eq!(one_below.consumed().work, 3);
    assert_eq!(
        one_below.first_unpaid_locus(),
        Some(NativeJoinLocus {
            source: NativeJoinLocusSource::TupleRow(2),
            form: "adapter row",
        })
    );

    let mut zero_storage = NativeJoinBudget::new(8, 0);
    zero_storage.resume();
    let allocation_locus = NativeJoinLocus {
        source: NativeJoinLocusSource::TupleRow(2),
        form: "adapter allocation",
    };
    let mut allocation_ran = false;
    let allocation = zero_storage
        .accounted_charge(NativeJoinChargeKind::Storage, 1, allocation_locus)
        .map(|()| {
            allocation_ran = true;
            vec![42_u8]
        });
    assert_eq!(
        allocation,
        Err(NativeJoinDeniedCharge {
            kind: NativeJoinChargeKind::Storage,
            amount: 1,
        })
    );
    assert!(!allocation_ran, "allocation must follow an admitted charge");
    assert_eq!(zero_storage.consumed(), NativeJoinCharge::default());
    assert_eq!(zero_storage.first_unpaid_locus(), Some(allocation_locus));
    assert_eq!(
        (zero_storage.work_limit(), zero_storage.storage_limit()),
        (8, 0)
    );
}

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
        if kind == NativeJoinChargeKind::Work {
            assert!(
                denied.requested.is_some(),
                "ample storage admits first locus"
            );
        }
        if denied.requested.is_some() {
            assert_eq!(denied.requested.as_deref(), Some(locus));
            assert_eq!(
                denied.at.as_deref(),
                Some(match kind {
                    NativeJoinChargeKind::Work => work_locus,
                    NativeJoinChargeKind::Storage => locus,
                })
            );
            assert!(denied.form.is_some());
        } else {
            assert_eq!(denied.at, None);
            assert_eq!(denied.form, None);
        }
        let attempted = short.first_unpaid().expect("first unpaid charge recorded");
        assert_eq!(attempted.kind, kind);
        assert!(attempted.amount > 0);
        assert_eq!(denied.denied, Some(attempted));
        assert_eq!(short.last_success_delta(), None);
        match kind {
            NativeJoinChargeKind::Work => {
                assert!(short.consumed().work <= work);
            }
            NativeJoinChargeKind::Storage => {
                assert!(short.consumed().storage <= storage);
                assert!(short.consumed().work <= required.work);
            }
        }
    }

    let mut zero_storage = NativeJoinBudget::new(u64::MAX, 0);
    let denied = run(&mut zero_storage).expect_err("zero storage refuses");
    assert_eq!(denied.code, ExactTypeProjectionErrorCode::ExhaustedStorage);
    assert_eq!(denied.requested, None);
    assert_eq!(denied.at, None);
    assert_eq!(denied.form, None);
    assert_eq!(zero_storage.consumed().storage, 0);
    assert_eq!(denied.denied, zero_storage.first_unpaid());
    (expected, required)
}

/// Each external locus source is admitted at the exact cumulative allowance and, one
/// unit below, is reported unchanged as the first unpaid locus without consuming.
#[trace("TC-471", "FR-358-AC-10")]
#[test]
fn external_loci_report_their_exact_source_at_the_allowance_edge() {
    let sources = [
        NativeJoinLocusSource::AdapterRow(4),
        NativeJoinLocusSource::AuthorityRow,
        NativeJoinLocusSource::PreparationSelection(7),
        NativeJoinLocusSource::PreparationOutput(9),
        NativeJoinLocusSource::MonitorAuthority,
    ];
    for source in sources {
        let locus = NativeJoinLocus {
            source,
            form: "external form",
        };
        let mut exact = NativeJoinBudget::new(2 + 5, 0);
        exact.resume();
        assert_eq!(
            exact.accounted_charge(NativeJoinChargeKind::Work, 2, locus),
            Ok(())
        );
        assert_eq!(
            exact.accounted_charge(NativeJoinChargeKind::Work, 5, locus),
            Ok(())
        );
        assert_eq!(
            exact.consumed(),
            NativeJoinCharge {
                work: 7,
                storage: 0
            }
        );
        assert_eq!(exact.first_unpaid_locus(), None);

        let mut short = NativeJoinBudget::new(2 + 5 - 1, 0);
        short.resume();
        assert_eq!(
            short.accounted_charge(NativeJoinChargeKind::Work, 2, locus),
            Ok(())
        );
        let denied = NativeJoinDeniedCharge {
            kind: NativeJoinChargeKind::Work,
            amount: 5,
        };
        assert_eq!(
            short.accounted_charge(NativeJoinChargeKind::Work, 5, locus),
            Err(denied)
        );
        assert_eq!(
            short.consumed(),
            NativeJoinCharge {
                work: 2,
                storage: 0
            }
        );
        assert_eq!(short.first_unpaid(), Some(denied));
        assert_eq!(short.first_unpaid_locus(), Some(locus));
    }
}
