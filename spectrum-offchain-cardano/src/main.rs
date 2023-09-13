use std::path::Path;

use futures::StreamExt;
use pallas_network::miniprotocols::Point;

use cardano_chain_sync::chain_sync_stream;
use cardano_chain_sync::client::{ChainSyncClient, ChainSyncConf};
use cardano_chain_sync::data::LedgerTxEvent;
use cardano_chain_sync::event_source::event_source_ledger;
use cardano_submit_api::client::{LocalTxSubmissionClient, LocalTxSubmissionClientConf};

mod constants;
mod data;
mod event_sink;

#[tokio::main]
async fn main() {
    let chain_sync_conf = ChainSyncConf {
        path: Path::new("/var/lib/docker/volumes/cardano_node-ipc/_data/node.socket"),
        magic: 1,
        starting_point: Point::Specific(
            37792291,
            hex::decode("516771c5f7bdb225a704afb67b0a31d86af8ae7cf747b65f7f5930dcd7381f48").unwrap(),
        ),
    };
    let chain_sync = ChainSyncClient::init(chain_sync_conf)
        .await
        .expect("ChainSync initialization wasn't successful");
    let tx_submit_conf = LocalTxSubmissionClientConf {
        path: Path::new("/var/lib/docker/volumes/cardano_node-ipc/_data/node.socket"),
        magic: 1,
    };
    let mut tx_submit = LocalTxSubmissionClient::init(tx_submit_conf)
        .await
        .expect("ChainSync initialization wasn't successful");
    let mut ledger_stream = Box::pin(event_source_ledger(chain_sync_stream(chain_sync)));
    loop {
        if let Some(next) = ledger_stream.next().await {
            match next {
                LedgerTxEvent::TxApplied(tx) => {
                    println!("Apply()");
                    tx_submit.submit_tx(tx).await.expect("Not ok");
                },
                LedgerTxEvent::TxUnapplied(tx) => println!("UnApply()"),
            }
        }
    }
}
