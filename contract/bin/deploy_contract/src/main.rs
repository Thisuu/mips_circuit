
use deploy_contract::deploy;


#[tokio::main]
async fn main() {

    let address = deploy("https://goerli.infura.io/v3/e4903a8305824888b3d9ea0e7760b31e","/Users/yuqi/project/mips_circuit/contract/verifier/g16/verifier").await;
    // let address = deploy("http://127.0.0.1:8545","/Users/yuqi/project/mips_circuit/contract/verifier/g16/verifier").await;

    println!("{:?}",address);
}
