//! Aggregator is an off-chain actor
//! Aggregator composite multiple components to process the off-chain status:
//!
//! * Sync layer2 blocks from layer1, then send to generator
//! * Accept layer2 tx via RPC, then send to generator
//! * Watch the chain, send challenge if a invalid block is committed
//! * Become validator and submit new blocks to layer1

mod chain;
mod collector;
mod config;
mod crypto;
mod deposition;
mod jsonrpc_types;
mod rpc;
mod state_impl;
mod tx_pool;

use crate::state_impl::SyncCodeStore;
use anyhow::Result;
use chain::{Chain, HeaderInfo};
use ckb_types::prelude::*;
use collector::lumos::Lumos;
use collector::Collector;
use config::Config;
use gw_generator::Generator;
use state_impl::StateImpl;
use tx_pool::{NextBlockContext, TxPool};

fn build_config() -> Config {
    unimplemented!()
}

fn build_collector(_config: &Config) -> impl Collector {
    Lumos
}

fn run() -> Result<()> {
    let config = build_config();
    let tip = config.chain.l2_genesis.clone();
    let collector = build_collector(&config);
    let genesis = collector.get_header_by_number(0).unwrap().unwrap();
    let last_synced = HeaderInfo {
        number: 0,
        block_hash: genesis.calc_header_hash().unpack(),
    };
    let code_store = SyncCodeStore::new(Default::default());
    let state = StateImpl::default();
    let tx_pool = {
        let generator = Generator::new(code_store.clone());
        let nb_ctx = NextBlockContext {
            // TODO get next block context from consensus
            aggregator_id: config
                .chain
                .signer
                .as_ref()
                .expect("must have a signer")
                .account_id,
        };
        TxPool::create(state.new_overlay()?, generator, &tip, nb_ctx)?
    };
    let mut chain = {
        let generator = Generator::new(code_store);
        Chain::new(
            config.chain,
            state,
            tip.raw(),
            last_synced,
            collector,
            generator,
            tx_pool,
        )
    };
    println!("sync chain!");
    chain.sync()?;
    let deposition_requests = Vec::new();
    chain.produce_block(deposition_requests)?;
    Ok(())
}

fn main() {
    run().expect("run aggregator");
}
