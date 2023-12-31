use std::fmt::{Display, Formatter};

use bloom_offchain::execution_engine::liquidity_book::side::SideM;
use spectrum_cardano_lib::AssetClass;

pub mod event_sink;
pub mod execution_engine;
pub mod operator_address;
pub mod orders;
pub mod pools;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct PairId(AssetClass, AssetClass);

impl PairId {
    /// Build canonical pair.
    pub fn canonical(x: AssetClass, y: AssetClass) -> Self {
        let xs = order_canonical(x, y);
        Self(xs[0], xs[1])
    }
}

/// Determine side of a trade relatively to canonical pair.
pub fn side_of(input: AssetClass, output: AssetClass) -> SideM {
    let xs = order_canonical(input, output);
    if xs[0] == input {
        SideM::Ask
    } else {
        SideM::Bid
    }
}

fn order_canonical(x: AssetClass, y: AssetClass) -> [AssetClass; 2] {
    let mut bf = [x, y];
    bf.sort();
    bf
}

impl Display for PairId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("{}:{}", self.0, self.1).as_str())
    }
}
