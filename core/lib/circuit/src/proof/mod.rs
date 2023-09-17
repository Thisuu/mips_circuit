use rand_0_8::rngs::StdRng;
use rand_0_8::SeedableRng;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fs::File;
use std::io::{BufReader, Cursor};
use std::path::Path;
use zokrates_ark::Ark;
use zokrates_ast::ir;
use zokrates_ast::ir::{ProgEnum, Witness};
use zokrates_bellman::Bellman;
use zokrates_common::helpers::{BackendParameter, Parameters, SchemeParameter, CurveParameter};
use zokrates_field::{Bn128Field,Field};
use zokrates_proof_systems::{Backend, G16, GM17, Marlin, Scheme, TaggedProof};
use zokrates_proof_systems::rng::get_rng_from_entropy;
use zokrates_proof_systems::Proof;
use web3::{
    helpers,
    contract::{Contract, Options},
    types::{U64,Address,H256}
};
// use primitive_types::U256;

use std::fs;
use std::str::FromStr;
use web3::ethabi::Token;
use zokrates_proof_systems::{
   SolidityCompatibleScheme
};
use web3::contract::tokens::Tokenizable;
use web3::types::TransactionRequest;
use web3::types::U256;
use to_token::ToToken;
pub mod to_token;
use secp256k1::SecretKey;



pub fn generate_proof(args: &HashMap<String, String>) -> Result<String, String> {
    let path = Path::new(args.get(&"input".to_string()).unwrap());
    let file = File::open(path).map_err(|why| format!("Could not open {}: {}", path.display(), why))?;

    let mut reader = BufReader::new(file);
    let prog = ProgEnum::deserialize(&mut reader)?;

    let curve_parameter = CurveParameter::try_from(prog.curve())?;

    let backend_parameter = BackendParameter::try_from(args.get(&"backend".to_string()).unwrap().as_str())?;
    let scheme_parameter =
        SchemeParameter::try_from(args.get(&"proving-scheme".to_string()).unwrap().as_str())?;

    let parameters = Parameters(backend_parameter, curve_parameter, scheme_parameter);

    match parameters {
        #[cfg(feature = "bellman")]
        Parameters(BackendParameter::Bellman, _, SchemeParameter::G16) => match prog {
            ProgEnum::Bn128Program(p) => generate::<_, _, G16, Bellman>(p, args),
            ProgEnum::Bls12_381Program(p) => {
                generate::<_, _, G16, Bellman>(p, args)
            }
            _ => unreachable!(),
        },
        #[cfg(feature = "ark")]
        Parameters(BackendParameter::Ark, _, SchemeParameter::G16) => match prog {
            ProgEnum::Bn128Program(p) => generate::<_, _, G16, Ark>(p, args),
            ProgEnum::Bls12_381Program(p) => generate::<_, _, G16, Ark>(p, args),
            ProgEnum::Bls12_377Program(p) => generate::<_, _, G16, Ark>(p, args),
            ProgEnum::Bw6_761Program(p) => generate::<_, _, G16, Ark>(p, args),
            _ => unreachable!(),
        },
        #[cfg(feature = "ark")]
        Parameters(BackendParameter::Ark, _, SchemeParameter::GM17) => match prog {
            ProgEnum::Bn128Program(p) => generate::<_, _, GM17, Ark>(p, args),
            ProgEnum::Bls12_381Program(p) => generate::<_, _, GM17, Ark>(p, args),
            ProgEnum::Bls12_377Program(p) => generate::<_, _, GM17, Ark>(p, args),
            ProgEnum::Bw6_761Program(p) => generate::<_, _, GM17, Ark>(p, args),
            _ => unreachable!(),
        },
        #[cfg(feature = "ark")]
        Parameters(BackendParameter::Ark, _, SchemeParameter::MARLIN) => match prog {
            ProgEnum::Bn128Program(p) => generate::<_, _, Marlin, Ark>(p, args),
            ProgEnum::Bls12_381Program(p) => {
                generate::<_, _, Marlin, Ark>(p, args)
            }
            ProgEnum::Bls12_377Program(p) => {
                generate::<_, _, Marlin, Ark>(p, args)
            }
            ProgEnum::Bw6_761Program(p) => generate::<_, _, Marlin, Ark>(p, args),
            _ => unreachable!(),
        },
        _ => unreachable!(),
    }
}

fn generate<
    'a,
    T: Field,
    I: Iterator<Item = ir::Statement<'a, T>>,
    S: Scheme<T>,
    B: Backend<T, S>,
>(
    program: ir::ProgIterator<'a, T, I>,
    args: &HashMap<String, String>,
) -> Result<String, String> {
    vlog::info!("Generating proof...");

    let t = "true".to_string();
    // deserialize witness
    let witness_str = args.get(&"witness".to_string()).unwrap().clone();
    vlog::info!("witness_str:{}\n", witness_str);
    let witness_bytes = hex::decode(witness_str).unwrap();
    let mut buff = Cursor::new(witness_bytes);
    let witness = Witness::read(buff).map_err(|why| format!("Could not load witness: {:?}", why))?;

    let pk_path = Path::new(args.get(&"proving-key-path".to_string()).unwrap());
    let pk_file = File::open(pk_path)
        .map_err(|why| format!("Could not open {}: {}", pk_path.display(), why))?;

    let pk_reader = BufReader::new(pk_file);

    let mut rng = args
        .get("entropy")
        .map(|s| s.as_str())
        .map(get_rng_from_entropy)
        .unwrap_or_else(StdRng::from_entropy);

    let proof = B::generate_proof(program, witness, pk_reader, &mut rng);

    let proof_str =
        serde_json::to_string_pretty(&TaggedProof::<T, S>::new(proof.proof, proof.inputs)).unwrap();

    let verbose = matches!(args.get(&"verbose".to_string()).unwrap_or(&"true".to_string()), t);
    if verbose {
        vlog::info!("Proof:\n{}", proof_str);
    }

    Ok(proof_str)
}

pub async fn call_verify<S: SolidityCompatibleScheme<Bn128Field> + ToToken<Bn128Field>>(
    proof: Proof<Bn128Field, S>,chainUrl:&str,contractAddress:&str,abiPath:&str,account:&str) -> bool {

    let solidity_proof = S::Proof::from(proof.proof);

    // let modified_solidity_proof = S::modify(solidity_proof);

    let proof_token = S::to_token(solidity_proof.clone());

    // let proof_token = S::to_token(modified_solidity_proof.clone());

    let input_token = Token::FixedArray(
        proof
            .inputs
            .iter()
            .map(|s| {
                let mut bytes = hex::decode(s.trim_start_matches("0x")).unwrap();
                debug_assert_eq!(bytes.len(), 32);
                Token::Uint(U256::from(&bytes[..]))
            })
            .collect::<Vec<_>>(),
    );

    // let inputs = ethabi::encode(&[proof_token, input_token.clone()]);

    let key_bytes = hex::decode(account).unwrap();

    let key = SecretKey::from_slice(key_bytes.as_slice()).unwrap();


    let http = web3::transports::Http::new(chainUrl).unwrap();
    let web3 = web3::Web3::new(http);

    let mut op = Options::default();
    op.gas = Some(3_000_000.into());
    let abi_path = abiPath.to_owned() + "/verifier.abi";
    let contract = Contract::from_json(web3.eth(),contractAddress.parse().unwrap(),fs::read(abi_path).unwrap().as_slice()).unwrap();

    // let inputs = contract.abi().function("verifyTx").unwrap().encode_input(&[proof_token, input_token.clone()]).unwrap();
    let result = contract.signed_call_with_confirmations("verifyTx",vec![proof_token, input_token.clone()].as_slice(), op, 1, &key).await.unwrap();
    if result.status.unwrap() == U64::from(1){
        println!("verify proof success, tx hash is: {:?}",result.transaction_hash);
        vlog::warn!("verify proof success, tx hash is: {:?}",result.transaction_hash);
        return true;
    } else {
        println!("verify proof false");
        vlog::warn!("verify proof false");
        return  false;
    }

}
