
use deploy_contract::deploy;


#[tokio::main]
async fn main() {

    let address = deploy("http://127.0.0.1:8545","/Users/yuqi/project/mips_circuit/contract/verifier/g16/verifier").await;
    println!("{:?}",address);
}
