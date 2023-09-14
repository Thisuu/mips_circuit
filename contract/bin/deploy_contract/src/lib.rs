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
use web3::signing::SecretKeyRef;
use secp256k1::SecretKey;



pub async fn deploy(chainUrl:&str,compiledDirectoryPath:&str) -> String {
    let eth_key = "3d71348530009da0be20899b42617410cd4789714dea9671ed60666ef3e00358";
    let key_bytes = hex::decode(eth_key).unwrap();
    let key = SecretKey::from_slice(key_bytes.as_slice()).unwrap();
    let http = web3::transports::Http::new(chainUrl).unwrap();
    let web3 = web3::Web3::new(http);
    // let my_account = hex!("29da89f4F432E922383FA71a8B2CD11E0e9197aA");
    let bytecode_path = compiledDirectoryPath.to_owned() + "/Verifier.bin";
    let abi_path = compiledDirectoryPath.to_owned() + "/Verifier.abi";
    // println!("{:?}",bytecode_path);
    // Get the contract bytecode for instance from Solidity compiler
    let bytecode = fs::read_to_string(bytecode_path).unwrap();

    // let bytecode = include_str!("/Users/yuqi/project/zkdataflow/contract_source/build/KeyedVerifier.bin");
    // Deploying a contract

    // let contract = Contract::deploy(web3.eth(), include_bytes!("/Users/yuqi/project/zkdataflow/contract_source/build/KeyedVerifier.abi"))?

    let contract = Contract::deploy(web3.eth(), fs::read(abi_path).unwrap().as_slice()).unwrap()
        .confirmations(5)
        .options(Options::with(|opt| {
            opt.gas = Some(3_000_000.into());
        }))
        .sign_with_key_and_execute(
            bytecode,
            (),
            &key,
            Some(5),
        )
        .await.unwrap();

    // let contract = Contract::deploy(web3.eth(), fs::read(abi_path).unwrap().as_slice()).unwrap()
    //     .confirmations(5)
    //     .options(Options::with(|opt| {
    //         opt.gas = Some(3_000_000.into());
    //     }))
    //     .execute(
    //         bytecode,
    //         (),
    //         web3::types::H160(my_account),
    //     )
    //     .await.unwrap();
    // let add = contract.address().as_bytes();
    let add = helpers::to_string(&contract.address());

    return add;
}
