use cml_multi_era::babbage::BabbageTransactionOutput;

use bloom_derivation::{Fragment, Stable, Tradable};
use bloom_offchain::execution_engine::liquidity_book::fragment::{ExBudgetUsed, OrderState, StateTrans};
use bloom_offchain::execution_engine::liquidity_book::side::SideM;
use spectrum_cardano_lib::{NetworkTime, OutputRef};
use spectrum_offchain::ledger::TryFromLedger;

use crate::orders::spot::SpotOrder;

pub mod auction;
pub mod spot;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Fragment, Stable, Tradable)]
pub enum AnyOrder {
    Spot(SpotOrder),
}

impl OrderState for AnyOrder {
    fn with_updated_time(self, time: u64) -> StateTrans<Self> {
        match self {
            AnyOrder::Spot(spot) => spot.with_updated_time(time).map(AnyOrder::Spot),
        }
    }
    fn with_applied_swap(self, removed_input: u64, added_output: u64) -> (StateTrans<Self>, ExBudgetUsed) {
        match self {
            AnyOrder::Spot(spot) => {
                let (tx, budget) = spot.with_applied_swap(removed_input, added_output);
                (tx.map(AnyOrder::Spot), budget)
            }
        }
    }
}

impl TryFromLedger<BabbageTransactionOutput, OutputRef> for AnyOrder {
    fn try_from_ledger(repr: &BabbageTransactionOutput, ctx: OutputRef) -> Option<Self> {
        SpotOrder::try_from_ledger(repr, ctx).map(AnyOrder::Spot)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Stateful<O, S> {
    pub order: O,
    pub state: S,
}

impl<O, S> Stateful<O, S> {
    pub fn new(order: O, state: S) -> Self {
        Self { order, state }
    }
}

/// State of order compatible with TLB.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct TLBCompatibleState {
    /// Side of the order relative to pair it maps to.
    pub side: SideM,
    pub time_now: NetworkTime,
}
