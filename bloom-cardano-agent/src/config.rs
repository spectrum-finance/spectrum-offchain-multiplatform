use std::time::Duration;

use cml_core::Slot;

use cardano_chain_sync::client::Point;
use cardano_explorer::data::ExplorerConfig;
use spectrum_cardano_lib::NetworkId;
use spectrum_offchain_cardano::creds::{OperatorCred, OperatorRewardAddress};

#[derive(serde::Deserialize)]
#[serde(bound = "'de: 'a")]
#[serde(rename_all = "camelCase")]
pub struct AppConfig<'a> {
    pub chain_sync: ChainSyncConfig<'a>,
    pub node: NodeConfig<'a>,
    pub tx_submission_buffer_size: usize,
    pub batcher_private_key: &'a str, //todo: store encrypted
    pub explorer: ExplorerConfig<'a>,
    pub reward_address: OperatorRewardAddress,
    pub executor_cred: OperatorCred,
    pub cardano_finalization_delay: Duration,
    pub backlog_capacity: u32,
    pub network_id: NetworkId,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeConfig<'a> {
    pub path: &'a str,
    pub magic: u64,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChainSyncConfig<'a> {
    pub starting_point: Point,
    pub disable_rollbacks_until: Slot,
    pub db_path: &'a str,
}
