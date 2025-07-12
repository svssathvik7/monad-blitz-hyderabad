use alloy::{
    hex::FromHex,
    network::{Ethereum, EthereumWallet, TransactionBuilder},
    primitives::{Address, FixedBytes, U256},
    providers::{
        fillers::{BlobGasFiller, ChainIdFiller, GasFiller, JoinFill, NonceFiller, WalletFiller},
        Identity, Provider, ProviderBuilder, RootProvider,
    },
    rpc::types::TransactionRequest,
    signers::local::PrivateKeySigner,
    sol,
};
use ipnetwork::IpNetwork;
use reqwest::Url;
use serde::{Deserialize, Serialize};

use crate::{
    constants::faucet,
    store::{PgStore, Store, Token, TokenTransfer, TokenType},
    ZERO_ADDRESS,
};

use tracing::error;

sol!(
    #[sol(rpc)]
    ERC20,
    "./erc20_abi.json"
);

type AlloyProvider = alloy::providers::fillers::FillProvider<
    JoinFill<
        JoinFill<
            Identity,
            JoinFill<GasFiller, JoinFill<BlobGasFiller, JoinFill<NonceFiller, ChainIdFiller>>>,
        >,
        WalletFiller<EthereumWallet>,
    >,
    RootProvider,
    Ethereum,
>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DripResponse {
    pub tx_hash: String,
    pub amount: String,
    pub magnification: u8,
}

pub struct Faucet {
    address: Address,
    provider: AlloyProvider,
    store: PgStore,
}

impl Faucet {
    pub fn new(private_key: &str, rpc_url: &str, store: PgStore) -> Self {
        let url = Url::parse(&rpc_url).expect("Failed to parse rpc url");
        let private_key_bytes = FixedBytes::from_hex(private_key).expect("Invalid private key");
        let signer = PrivateKeySigner::from_bytes(&private_key_bytes).expect("Invalid private key");
        let wallet = EthereumWallet::new(signer.clone());
        let provider = ProviderBuilder::new().wallet(wallet).on_http(url);

        Self {
            address: signer.address(),
            provider,
            store,
        }
    }

    pub async fn send_erc_20(
        &self,
        token_address: &str,
        to: &str,
        amount: u128,
        ip: IpNetwork,
        magnification: u8,
    ) -> Result<DripResponse, String> {
        if amount == 0 {
            return Err("0 amount".to_string());
        }

        let token_address = Address::parse_checksummed(&token_address, None).map_err(|e| {
            error!("Failed to parse token address {} {}", token_address, e);
            "Invalid token address"
        })?;
        let contract = ERC20::new(token_address, self.provider.clone());
        let to_address = Address::parse_checksummed(&to, None).map_err(|e| {
            error!("Failed to parse to_address {}: {}", to, e);
            "Invalid to address"
        })?;

        let balance: ERC20::balanceOfReturn =
            contract.balanceOf(self.address).call().await.map_err(|e| {
                error!("Error fetching balance {}", e);
                "Failed to fetch balance".to_string()
            })?;

        if balance._0 == U256::ZERO || balance._0 < U256::from(amount) {
            return Err("Insufficient balance".to_string());
        }

        let send_amount = U256::from(amount);
        let tx = contract.transfer(to_address, send_amount);

        let unconfirmed_tx = tx.send().await.map_err(|e| {
            error!("Failed to send erc20 drip transaction to chain {}", e);
            "Failed to send transaction"
        })?;

        let chain_id = self.provider.get_chain_id().await.unwrap_or_default();

        if let Err(e) = self
            .store
            .create_token_transfer(TokenTransfer {
                token_type: TokenType::ERC20,
                token_address: token_address.to_string(),
                to_address: to.to_string(),
                tx_hash: unconfirmed_tx.tx_hash().to_string(),
                amount: amount.to_string(),
                from_address: self.address.to_string(),
                chain_id: chain_id as i32,
                ip,
            })
            .await
        {
            println!(
                "Failed to store token transfer: {:?} token_address: {:?} to: {:?} amount: {:?} tx_hash: {:?}",
                e, token_address, to, amount, unconfirmed_tx.tx_hash().to_string()
            );
        }

        Ok(DripResponse {
            amount: amount.to_string(),
            tx_hash: unconfirmed_tx.tx_hash().to_string(),
            magnification,
        })
    }

    #[allow(dead_code)]
    pub async fn build_send_erc20_tx(
        &self,
        token_address: &str,
        to: &str,
        amount: u128,
    ) -> Result<Vec<u8>, String> {
        if amount == 0 {
            return Err("0 amount".to_string());
        }

        let token_address = Address::parse_checksummed(&token_address, None).map_err(|e| {
            error!("Failed to parse token address {} {}", token_address, e);
            "Invalid token address"
        })?;
        let contract = ERC20::new(token_address, self.provider.clone());
        let to_address = Address::parse_checksummed(&to, None).map_err(|e| {
            error!("Failed to parse to_address {} {}", to, e);
            "Invalid to_address"
        })?;

        let balance: ERC20::balanceOfReturn =
            contract.balanceOf(self.address).call().await.map_err(|e| {
                error!("Failed to fetch balance of {} {}", self.address, e);
                "Failed to fetch balance"
            })?;

        if balance._0 == U256::ZERO || balance._0 < U256::from(amount) {
            return Err("Insufficient balance".to_string());
        }

        let send_amount = U256::from(amount);
        let transfer = contract.transfer(to_address, send_amount);
        let calldata = transfer.calldata();

        Ok(calldata.to_vec())
    }

    pub async fn send_native_token(
        &self,
        to: &str,
        amount: u128,
        ip: IpNetwork,
        magnification: u8,
    ) -> Result<DripResponse, String> {
        let _amount = U256::from(amount);
        let to_address = Address::parse_checksummed(to, None).map_err(|e| {
            error!("Failed to parse to_address {} {}", to, e);
            "Invalid to_address"
        })?;
        let tx = TransactionRequest::default()
            .with_from(self.address)
            .with_to(to_address)
            .with_value(_amount);
        let unconfirmed_tx = self.provider.send_transaction(tx).await.map_err(|e| {
            error!("Failed to send transaction to chain {}", e);
            "Failed to send transaction"
        })?;

        let chain_id = self.provider.get_chain_id().await.unwrap_or_default();

        if let Err(e) = self
            .store
            .create_token_transfer(TokenTransfer {
                token_type: TokenType::NATIVE,
                token_address: ZERO_ADDRESS.to_string(),
                tx_hash: unconfirmed_tx.tx_hash().to_string(),
                from_address: self.address.to_string(),
                to_address: to.to_string(),
                amount: amount.to_string(),
                chain_id: chain_id as i32,
                ip,
            })
            .await
        {
            println!(
                "Failed to store token transfer: {:?} to: {:?} amount: {:?} tx_hash: {:?}",
                e,
                to,
                amount,
                unconfirmed_tx.tx_hash()
            );
        }

        Ok(DripResponse {
            amount: amount.to_string(),
            tx_hash: unconfirmed_tx.tx_hash().to_string(),
            magnification,
        })
    }

    pub async fn deploy_erc_20(
        &self,
        name: String,
        symbol: String,
        total_supply: u128,
        decimals: u8,
        logo_url: String,
        deployer_address: String,
    ) -> Result<String, String> {
        sol! {
        #[allow(missing_docs)]
        #[sol(rpc, bytecode="608060405234801561000f575f5ffd5b5060405161194438038061194483398181016040528101906100319190610507565b8383816003908161004291906107aa565b50806004908161005291906107aa565b5050506100896100666100ac60201b60201c565b82600a61007391906109d5565b8461007e9190610a1f565b6100b360201b60201c565b8060055f6101000a81548160ff021916908360ff16021790555050505050610b48565b5f33905090565b5f73ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff1603610123575f6040517fec442f0500000000000000000000000000000000000000000000000000000000815260040161011a9190610a9f565b60405180910390fd5b6101345f838361013860201b60201c565b5050565b5f73ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff1603610188578060025f82825461017c9190610ab8565b92505081905550610256565b5f5f5f8573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f2054905081811015610211578381836040517fe450d38c00000000000000000000000000000000000000000000000000000000815260040161020893929190610afa565b60405180910390fd5b8181035f5f8673ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f2081905550505b5f73ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff160361029d578060025f82825403925050819055506102e7565b805f5f8473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f82825401925050819055505b8173ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff167fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef836040516103449190610b2f565b60405180910390a3505050565b5f604051905090565b5f5ffd5b5f5ffd5b5f5ffd5b5f5ffd5b5f601f19601f8301169050919050565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52604160045260245ffd5b6103b08261036a565b810181811067ffffffffffffffff821117156103cf576103ce61037a565b5b80604052505050565b5f6103e1610351565b90506103ed82826103a7565b919050565b5f67ffffffffffffffff82111561040c5761040b61037a565b5b6104158261036a565b9050602081019050919050565b8281835e5f83830152505050565b5f61044261043d846103f2565b6103d8565b90508281526020810184848401111561045e5761045d610366565b5b610469848285610422565b509392505050565b5f82601f83011261048557610484610362565b5b8151610495848260208601610430565b91505092915050565b5f819050919050565b6104b08161049e565b81146104ba575f5ffd5b50565b5f815190506104cb816104a7565b92915050565b5f60ff82169050919050565b6104e6816104d1565b81146104f0575f5ffd5b50565b5f81519050610501816104dd565b92915050565b5f5f5f5f6080858703121561051f5761051e61035a565b5b5f85015167ffffffffffffffff81111561053c5761053b61035e565b5b61054887828801610471565b945050602085015167ffffffffffffffff8111156105695761056861035e565b5b61057587828801610471565b9350506040610586878288016104bd565b9250506060610597878288016104f3565b91505092959194509250565b5f81519050919050565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52602260045260245ffd5b5f60028204905060018216806105f157607f821691505b602082108103610604576106036105ad565b5b50919050565b5f819050815f5260205f209050919050565b5f6020601f8301049050919050565b5f82821b905092915050565b5f600883026106667fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff8261062b565b610670868361062b565b95508019841693508086168417925050509392505050565b5f819050919050565b5f6106ab6106a66106a18461049e565b610688565b61049e565b9050919050565b5f819050919050565b6106c483610691565b6106d86106d0826106b2565b848454610637565b825550505050565b5f5f905090565b6106ef6106e0565b6106fa8184846106bb565b505050565b5b8181101561071d576107125f826106e7565b600181019050610700565b5050565b601f821115610762576107338161060a565b61073c8461061c565b8101602085101561074b578190505b61075f6107578561061c565b8301826106ff565b50505b505050565b5f82821c905092915050565b5f6107825f1984600802610767565b1980831691505092915050565b5f61079a8383610773565b9150826002028217905092915050565b6107b3826105a3565b67ffffffffffffffff8111156107cc576107cb61037a565b5b6107d682546105da565b6107e1828285610721565b5f60209050601f831160018114610812575f8415610800578287015190505b61080a858261078f565b865550610871565b601f1984166108208661060a565b5f5b8281101561084757848901518255600182019150602085019450602081019050610822565b868310156108645784890151610860601f891682610773565b8355505b6001600288020188555050505b505050505050565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52601160045260245ffd5b5f8160011c9050919050565b5f5f8291508390505b60018511156108fb578086048111156108d7576108d6610879565b5b60018516156108e65780820291505b80810290506108f4856108a6565b94506108bb565b94509492505050565b5f8261091357600190506109ce565b81610920575f90506109ce565b816001811461093657600281146109405761096f565b60019150506109ce565b60ff84111561095257610951610879565b5b8360020a91508482111561096957610968610879565b5b506109ce565b5060208310610133831016604e8410600b84101617156109a45782820a90508381111561099f5761099e610879565b5b6109ce565b6109b184848460016108b2565b925090508184048111156109c8576109c7610879565b5b81810290505b9392505050565b5f6109df8261049e565b91506109ea836104d1565b9250610a177fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff8484610904565b905092915050565b5f610a298261049e565b9150610a348361049e565b9250828202610a428161049e565b91508282048414831517610a5957610a58610879565b5b5092915050565b5f73ffffffffffffffffffffffffffffffffffffffff82169050919050565b5f610a8982610a60565b9050919050565b610a9981610a7f565b82525050565b5f602082019050610ab25f830184610a90565b92915050565b5f610ac28261049e565b9150610acd8361049e565b9250828201905080821115610ae557610ae4610879565b5b92915050565b610af48161049e565b82525050565b5f606082019050610b0d5f830186610a90565b610b1a6020830185610aeb565b610b276040830184610aeb565b949350505050565b5f602082019050610b425f830184610aeb565b92915050565b610def80610b555f395ff3fe608060405234801561000f575f5ffd5b5060043610610091575f3560e01c8063313ce56711610064578063313ce5671461013157806370a082311461014f57806395d89b411461017f578063a9059cbb1461019d578063dd62ed3e146101cd57610091565b806306fdde0314610095578063095ea7b3146100b357806318160ddd146100e357806323b872dd14610101575b5f5ffd5b61009d6101fd565b6040516100aa9190610a68565b60405180910390f35b6100cd60048036038101906100c89190610b19565b61028d565b6040516100da9190610b71565b60405180910390f35b6100eb6102af565b6040516100f89190610b99565b60405180910390f35b61011b60048036038101906101169190610bb2565b6102b8565b6040516101289190610b71565b60405180910390f35b6101396102e6565b6040516101469190610c1d565b60405180910390f35b61016960048036038101906101649190610c36565b6102fb565b6040516101769190610b99565b60405180910390f35b610187610340565b6040516101949190610a68565b60405180910390f35b6101b760048036038101906101b29190610b19565b6103d0565b6040516101c49190610b71565b60405180910390f35b6101e760048036038101906101e29190610c61565b6103f2565b6040516101f49190610b99565b60405180910390f35b60606003805461020c90610ccc565b80601f016020809104026020016040519081016040528092919081815260200182805461023890610ccc565b80156102835780601f1061025a57610100808354040283529160200191610283565b820191905f5260205f20905b81548152906001019060200180831161026657829003601f168201915b5050505050905090565b5f5f610297610474565b90506102a481858561047b565b600191505092915050565b5f600254905090565b5f5f6102c2610474565b90506102cf85828561048d565b6102da858585610520565b60019150509392505050565b5f60055f9054906101000a900460ff16905090565b5f5f5f8373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f20549050919050565b60606004805461034f90610ccc565b80601f016020809104026020016040519081016040528092919081815260200182805461037b90610ccc565b80156103c65780601f1061039d576101008083540402835291602001916103c6565b820191905f5260205f20905b8154815290600101906020018083116103a957829003601f168201915b5050505050905090565b5f5f6103da610474565b90506103e7818585610520565b600191505092915050565b5f60015f8473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f8373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f2054905092915050565b5f33905090565b6104888383836001610610565b505050565b5f61049884846103f2565b90507fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff81101561051a578181101561050b578281836040517ffb8f41b200000000000000000000000000000000000000000000000000000000815260040161050293929190610d0b565b60405180910390fd5b61051984848484035f610610565b5b50505050565b5f73ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff1603610590575f6040517f96c6fd1e0000000000000000000000000000000000000000000000000000000081526004016105879190610d40565b60405180910390fd5b5f73ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff1603610600575f6040517fec442f050000000000000000000000000000000000000000000000000000000081526004016105f79190610d40565b60405180910390fd5b61060b8383836107df565b505050565b5f73ffffffffffffffffffffffffffffffffffffffff168473ffffffffffffffffffffffffffffffffffffffff1603610680575f6040517fe602df050000000000000000000000000000000000000000000000000000000081526004016106779190610d40565b60405180910390fd5b5f73ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff16036106f0575f6040517f94280d620000000000000000000000000000000000000000000000000000000081526004016106e79190610d40565b60405180910390fd5b8160015f8673ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f8573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f208190555080156107d9578273ffffffffffffffffffffffffffffffffffffffff168473ffffffffffffffffffffffffffffffffffffffff167f8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925846040516107d09190610b99565b60405180910390a35b50505050565b5f73ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff160361082f578060025f8282546108239190610d86565b925050819055506108fd565b5f5f5f8573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f20549050818110156108b8578381836040517fe450d38c0000000000000000000000000000000000000000000000000000000081526004016108af93929190610d0b565b60405180910390fd5b8181035f5f8673ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f2081905550505b5f73ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff1603610944578060025f828254039250508190555061098e565b805f5f8473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f82825401925050819055505b8173ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff167fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef836040516109eb9190610b99565b60405180910390a3505050565b5f81519050919050565b5f82825260208201905092915050565b8281835e5f83830152505050565b5f601f19601f8301169050919050565b5f610a3a826109f8565b610a448185610a02565b9350610a54818560208601610a12565b610a5d81610a20565b840191505092915050565b5f6020820190508181035f830152610a808184610a30565b905092915050565b5f5ffd5b5f73ffffffffffffffffffffffffffffffffffffffff82169050919050565b5f610ab582610a8c565b9050919050565b610ac581610aab565b8114610acf575f5ffd5b50565b5f81359050610ae081610abc565b92915050565b5f819050919050565b610af881610ae6565b8114610b02575f5ffd5b50565b5f81359050610b1381610aef565b92915050565b5f5f60408385031215610b2f57610b2e610a88565b5b5f610b3c85828601610ad2565b9250506020610b4d85828601610b05565b9150509250929050565b5f8115159050919050565b610b6b81610b57565b82525050565b5f602082019050610b845f830184610b62565b92915050565b610b9381610ae6565b82525050565b5f602082019050610bac5f830184610b8a565b92915050565b5f5f5f60608486031215610bc957610bc8610a88565b5b5f610bd686828701610ad2565b9350506020610be786828701610ad2565b9250506040610bf886828701610b05565b9150509250925092565b5f60ff82169050919050565b610c1781610c02565b82525050565b5f602082019050610c305f830184610c0e565b92915050565b5f60208284031215610c4b57610c4a610a88565b5b5f610c5884828501610ad2565b91505092915050565b5f5f60408385031215610c7757610c76610a88565b5b5f610c8485828601610ad2565b9250506020610c9585828601610ad2565b9150509250929050565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52602260045260245ffd5b5f6002820490506001821680610ce357607f821691505b602082108103610cf657610cf5610c9f565b5b50919050565b610d0581610aab565b82525050565b5f606082019050610d1e5f830186610cfc565b610d2b6020830185610b8a565b610d386040830184610b8a565b949350505050565b5f602082019050610d535f830184610cfc565b92915050565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52601160045260245ffd5b5f610d9082610ae6565b9150610d9b83610ae6565b9250828201905080821115610db357610db2610d59565b5b9291505056fea264697066735822122030fdd9efa6e0c8049052c65527407bafea0d65e5e84fe1b15458e74230160f4564736f6c634300081c0033")]
        contract ERC20Token is ERC20 {
            uint8 private _decimals;

                constructor(string memory name, string memory symbol, uint256 total_supply, uint8 decimals) ERC20(name, symbol) {
                    _mint(_msgSender(), total_supply * (10 ** decimals));
                    _decimals = decimals;
                }

                function decimals() public view override returns (uint8) {
                    return _decimals;
                }
            }
        }

        let contract = ERC20Token::deploy(
            &self.provider,
            name.clone(),
            symbol.clone(),
            U256::from(total_supply),
            decimals,
        )
        .await
        .map_err(|e| {
            error!("Failed to deploy contract {} {}", name, e);
            "Failed to deploy contract"
        })?;

        let contract_address = contract.address();
        let chain_id = self.provider.get_chain_id().await.unwrap_or_default();

        let withdraw_limit = total_supply as f64 / faucet::WITHDRAW_LIMIT_DENOMINATOR;
        let limit = withdraw_limit * (10.0_f64.powi(decimals as i32)).floor();

        if let Err(_e) = self
            .store
            .create_token_entry(Token {
                address: contract_address.to_string(),
                token_type: TokenType::ERC20,
                name,
                chain_id: chain_id as i32,
                symbol,
                logo_url,
                created_by: deployer_address,
                decimals: decimals as i32,
                withdraw_limit: limit.to_string(),
            })
            .await
        {}

        Ok(contract_address.to_string())
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::config::Config;

//     use super::*;

//     #[tokio::test]
//     async fn test_deploy_erc20() {
//         let config = Config::from_env();
//         let instance = Erc20::new(&config.private_key, &config.rpc_url);
//         let contract = instance
//             .deploy_erc_20("New Mani".to_string(), "MANI".to_string(), 147000000, 3)
//             .await
//             .expect("error");

//         println!("contract: {:?}", contract)
//     }
// }

#[cfg(test)]
mod tests {
    use crate::{config::Config, db};

    use super::*;

    #[tokio::test]
    async fn test_native_token_transfer() {
        let config = Config::from_env();
        let db_pool = db::init_db(&config.db_url)
            .await
            .expect("Failed to connect to DB");
        let store = PgStore::new(db_pool);

        let erc20 = Faucet::new(&config.private_key, &config.rpc_url, store);

        let tx = erc20
            .send_native_token(
                "0xE85EFc62D582C94a2be96AbB4bbE6d40fa773377",
                10000000000000000,
                ipnetwork::IpNetwork::V4("60.243.163.75".parse().unwrap()),
                1,
            )
            .await
            .expect("error sending tx");
        println!("response: {:?}", tx);
    }
    #[tokio::test]
    async fn test_erc20_token_transfer() {
        let config = Config::from_env();
        let db_pool = db::init_db(&config.db_url)
            .await
            .expect("Failed to connect to DB");
        let store = PgStore::new(db_pool);

        let erc20 = Faucet::new(&config.private_key, &config.rpc_url, store);

        let tx = erc20
            .send_erc_20(
                "0x222a8742a79078CFBB4A385922d8EE4cB367758C",
                "0xDda173bd23b07007394611D789EF789a9Aae5CF5",
                10000000000,
                ipnetwork::IpNetwork::V4("60.243.163.75".parse().unwrap()),
                1,
            )
            .await
            .expect("error sending tx");
        println!("response: {:?}", tx);
    }
}
