use alloy::{primitives::Address, providers::Provider, rpc::types::{Filter, Log, Topic}, transports::http::reqwest::Url, sol_types::SolEvent};

use crate::types::{AlloyProvider, TransferContract, TransferContractInstance};

pub struct Watcher {
    pub contract: TransferContractInstance,
    pub start_block: u64,
    pub interval: u64,
    pub block_span: u64,
}

impl Watcher {
    pub fn new(address: Address, rpc: AlloyProvider) -> Self {
        Watcher {
            contract: TransferContract::new(address, rpc),
            start_block: 0,  // Default value
            interval: 0,     // Default value
            block_span: 0,   // Default value
        }
    }

    pub fn with_start_block(mut self, start_block: u64) -> Self {
        self.start_block = start_block;
        self
    }

    pub fn with_interval(mut self, interval: u64) -> Self {
        self.interval = interval;
        self
    }

    pub fn with_block_span(mut self, block_span: u64) -> Self {
        self.block_span = block_span;
        self
    }


    pub async fn run(&self){

    }

    pub async fn process_logs(&self, logs: Vec<Log>) -> (){
        for log in logs{
            println!("log {:?}",log.topic0());
        }
    }

    pub async fn get_logs(&self) -> Vec<Log>{
        let start_block = self.start_block;
        let end_block = self.start_block + self.block_span;

        let address = self.contract.address();
        let filter = Filter::new()
            .address(*address)
            .from_block(start_block)
            .to_block(end_block)
            .event_signature(Topic::from(vec![TransferContract::TransferTo::SIGNATURE_HASH]));
        let logs = self
            .contract
            .provider()
            .get_logs(&filter)
            .await
            .map_err(|e| {
                eprintln!("{}", e);
            }).unwrap();
        logs
    }
}
