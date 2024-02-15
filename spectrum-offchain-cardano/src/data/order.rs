use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};

use cml_chain::builders::tx_builder::SignedTxBuilder;
use cml_chain::plutus::PlutusData;
use cml_core::serialization::FromBytes;
use cml_multi_era::babbage::BabbageTransactionOutput;

use spectrum_cardano_lib::plutus_data::RequiresRedeemer;
use spectrum_cardano_lib::transaction::BabbageTransactionOutputExtension;
use spectrum_cardano_lib::OutputRef;
use spectrum_offchain::backlog::data::{OrderWeight, Weighted};
use spectrum_offchain::data::order::{SpecializedOrder, UniqueOrder};
use spectrum_offchain::data::unique_entity::Predicted;
use spectrum_offchain::executor::{RunOrder, RunOrderError};
use spectrum_offchain::ledger::TryFromLedger;

use crate::data::deposit::ClassicalOnChainDeposit;
use crate::data::execution_context::ExecutionContext;
use crate::data::limit_swap::ClassicalOnChainLimitSwap;
use crate::data::pool::AnyCFMMPool;
use crate::data::redeem::ClassicalOnChainRedeem;
use crate::data::{OnChain, OnChainOrderId, PoolId};

pub struct Input;

pub struct Output;

pub struct Base;

pub struct Quote;

pub struct PoolNft;

#[derive(Debug, Clone)]
pub struct ClassicalOrder<Id, Ord> {
    pub id: Id,
    pub pool_id: PoolId,
    pub order: Ord,
}

pub enum ClassicalOrderAction {
    Apply,
    Refund,
}

#[derive(Debug, Clone)]
pub enum ClassicalOnChainOrder {
    Swap(OnChain<ClassicalOnChainLimitSwap>),
    Deposit(OnChain<ClassicalOnChainDeposit>),
    Redeem(OnChain<ClassicalOnChainRedeem>),
}

impl PartialEq for ClassicalOnChainOrder {
    fn eq(&self, other: &Self) -> bool {
        <Self as UniqueOrder>::get_self_ref(self).eq(&<Self as UniqueOrder>::get_self_ref(other))
    }
}

impl Eq for ClassicalOnChainOrder {}

impl Hash for ClassicalOnChainOrder {
    fn hash<H: Hasher>(&self, state: &mut H) {
        <Self as UniqueOrder>::get_self_ref(self).hash(state)
    }
}

impl Display for ClassicalOnChainOrder {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("ClassicalLimitSwap")
    }
}

impl Weighted for ClassicalOnChainOrder {
    fn weight(&self) -> OrderWeight {
        match self {
            ClassicalOnChainOrder::Swap(limit_swap) => OrderWeight::from(limit_swap.value.order.fee.0),
            ClassicalOnChainOrder::Deposit(deposit) => OrderWeight::from(deposit.value.order.ex_fee),
            ClassicalOnChainOrder::Redeem(redeem) => OrderWeight::from(redeem.value.order.ex_fee),
        }
    }
}

impl SpecializedOrder for ClassicalOnChainOrder {
    type TOrderId = OnChainOrderId;
    type TPoolId = PoolId;
    fn get_self_ref(&self) -> Self::TOrderId {
        match self {
            ClassicalOnChainOrder::Swap(limit_swap) => limit_swap.value.id,
            ClassicalOnChainOrder::Deposit(deposit) => deposit.value.id,
            ClassicalOnChainOrder::Redeem(redeem) => redeem.value.id,
        }
    }
    fn get_pool_ref(&self) -> Self::TPoolId {
        match self {
            ClassicalOnChainOrder::Swap(limit_swap) => limit_swap.value.pool_id,
            ClassicalOnChainOrder::Deposit(deposit) => deposit.value.pool_id,
            ClassicalOnChainOrder::Redeem(redeem) => redeem.value.pool_id,
        }
    }
}

impl RequiresRedeemer<ClassicalOrderAction> for ClassicalOnChainOrder {
    fn redeemer(action: ClassicalOrderAction) -> PlutusData {
        match action {
            ClassicalOrderAction::Apply => {
                PlutusData::from_bytes(hex::decode("d8799f00010100ff").unwrap()).unwrap()
            }
            ClassicalOrderAction::Refund => {
                PlutusData::from_bytes(hex::decode("d8799f01000001ff").unwrap()).unwrap()
            }
        }
    }
}

impl TryFromLedger<BabbageTransactionOutput, OutputRef> for ClassicalOnChainOrder {
    fn try_from_ledger(repr: &BabbageTransactionOutput, ctx: OutputRef) -> Option<Self> {
        ClassicalOnChainLimitSwap::try_from_ledger(repr, ctx)
            .map(|swap| {
                ClassicalOnChainOrder::Swap(OnChain {
                    value: swap,
                    source: repr.clone().upcast(),
                })
            })
            .or_else(|| {
                ClassicalOnChainDeposit::try_from_ledger(repr, ctx).map(|deposit| {
                    ClassicalOnChainOrder::Deposit(OnChain {
                        value: deposit,
                        source: repr.clone().upcast(),
                    })
                })
            })
            .or_else(|| {
                ClassicalOnChainRedeem::try_from_ledger(repr, ctx).map(|redeem| {
                    ClassicalOnChainOrder::Redeem(OnChain {
                        value: redeem,
                        source: repr.clone().upcast(),
                    })
                })
            })
    }
}

impl RunOrder<ClassicalOnChainOrder, ExecutionContext, SignedTxBuilder> for OnChain<AnyCFMMPool> {
    fn try_run(
        self,
        order: ClassicalOnChainOrder,
        ctx: ExecutionContext,
    ) -> Result<(SignedTxBuilder, Predicted<Self>), RunOrderError<ClassicalOnChainOrder>> {
        match order {
            ClassicalOnChainOrder::Swap(limit_swap) => <Self as RunOrder<
                OnChain<ClassicalOnChainLimitSwap>,
                ExecutionContext,
                SignedTxBuilder,
            >>::try_run(self, limit_swap, ctx)
            .map_err(|err| err.map(|inner| ClassicalOnChainOrder::Swap(inner))),
            ClassicalOnChainOrder::Deposit(deposit) => <Self as RunOrder<
                OnChain<ClassicalOnChainDeposit>,
                ExecutionContext,
                SignedTxBuilder,
            >>::try_run(self, deposit, ctx)
            .map_err(|err| err.map(|inner| ClassicalOnChainOrder::Deposit(inner))),
            ClassicalOnChainOrder::Redeem(redeem) => <Self as RunOrder<
                OnChain<ClassicalOnChainRedeem>,
                ExecutionContext,
                SignedTxBuilder,
            >>::try_run(self, redeem, ctx)
            .map_err(|err| err.map(|inner| ClassicalOnChainOrder::Redeem(inner))),
        }
    }
}
