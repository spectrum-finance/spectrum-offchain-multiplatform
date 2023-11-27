use std::collections::hash_map::Entry;
use std::collections::{BTreeMap, HashMap};

use spectrum_offchain::data::Has;

use crate::liquidity_book::types::{Price, SourceId};

pub trait PooledLiquidity<Pl> {
    fn best_price(&self) -> Option<Price>;
    fn try_pick<F>(&mut self, test: F) -> Option<Pl>
    where
        F: Fn(&Pl) -> bool;
    fn return_pool(&mut self, pool: Pl);
}

pub trait PoolStore<Pl> {
    fn update_pool(&mut self, pool: Pl);
}

#[derive(Debug, Clone)]
pub struct InMemoryPooledLiquidity<Pl> {
    pools: HashMap<SourceId, Pl>,
    quality_index: BTreeMap<PoolQuality, SourceId>,
}

impl<Pl: Has<SourceId>> PooledLiquidity<Pl> for InMemoryPooledLiquidity<Pl> {
    fn best_price(&self) -> Option<Price> {
        self.quality_index
            .first_key_value()
            .map(|(PoolQuality(p, _), _)| *p)
    }

    fn try_pick<F>(&mut self, test: F) -> Option<Pl>
    where
        F: Fn(&Pl) -> bool,
    {
        for id in self.quality_index.values() {
            match self.pools.entry(*id) {
                Entry::Occupied(pl) if test(pl.get()) => return Some(pl.remove()),
                _ => {}
            }
        }
        None
    }
    fn return_pool(&mut self, pool: Pl) {
        self.pools.insert(pool.get::<SourceId>(), pool);
    }
}

impl<Pl: Has<SourceId> + QualityMetric + Copy> PoolStore<Pl> for InMemoryPooledLiquidity<Pl> {
    fn update_pool(&mut self, pool: Pl) {
        let source = pool.get::<SourceId>();
        if let Some(old_pool) = self.pools.insert(source, pool) {
            self.quality_index.remove(&old_pool.quality());
            self.quality_index.insert(pool.quality(), source);
        }
    }
}

pub trait QualityMetric {
    fn quality(&self) -> PoolQuality;
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PoolQuality(/*price hint*/ Price, /*liquidity*/ u64);
