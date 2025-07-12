use alloy::{primitives::Address, providers::ProviderBuilder, transports::http::reqwest::Url};

use crate::watcher::Watcher;

pub mod watcher;
pub mod types;
pub mod pgstore;

fn main() {
    let watcher = Watcher::new(Address::parse_checksummed("0x34fb72ae60ad74ee575e8626c3e04cde62de9a638ba68bad49844838e9a42558", None)
    .expect("Invalid contract address"), ProviderBuilder::new().connect_http(Url::parse("https://testnet-rpc.monad.xyz").unwrap())).with_block_span(500).with_interval(5000).with_start_block(25888177);
}
