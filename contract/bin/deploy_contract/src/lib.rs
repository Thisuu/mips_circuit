use std::collections::HashMap;
use std::fs;
use hex_literal::hex;
use web3::{
    helpers,
    contract::{Contract, Options},
    types::{U256,U64,Address,H256}
};
use core::str;
use web3::contract::tokens::Tokenizable;


fn get_account() -> Address {
    let account = hex!("E2EA1B7352BaEfEba0D11A5FA614dF6cE9E77693").into();
    return account;
}

pub async fn deploy(chainUrl:&str,compiledDirectoryPath:&str) -> String {
    let http = web3::transports::Http::new(chainUrl).unwrap();
    let web3 = web3::Web3::new(http);
    let my_account = get_account();
    let bytecode_path = compiledDirectoryPath.to_owned() + "/Verifier.bin";
    let abi_path = compiledDirectoryPath.to_owned() + "/Verifier.abi";

    // Get the contract bytecode for instance from Solidity compiler
    let bytecode = fs::read_to_string(bytecode_path).unwrap();

    // let bytecode = include_str!("/Users/yuqi/project/zkdataflow/contract_source/build/KeyedVerifier.bin");
    // Deploying a contract

    // let contract = Contract::deploy(web3.eth(), include_bytes!("/Users/yuqi/project/zkdataflow/contract_source/build/KeyedVerifier.abi"))?

    let contract = Contract::deploy(web3.eth(), fs::read(abi_path).unwrap().as_slice()).unwrap()
        .confirmations(0)
        .options(Options::with(|opt| {
            opt.gas = Some(3_000_000.into());
        }))
        .execute(
            bytecode,
            (),
            my_account,
        )
        .await.unwrap();
    // let add = contract.address().as_bytes();
    let add = helpers::to_string(&contract.address());

    return add;
}
