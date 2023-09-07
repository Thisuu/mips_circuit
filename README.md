# Mips Circuit

Mips Circuit is a crypto circuit lib for Zero-Knowledge(ZK) VM based on MIPS Architecture. 

## Prequistise

- Postgres DB

```
DROP TABLE IF EXISTS t_block_witness_cloud;
CREATE TABLE t_block_witness_cloud
(
    f_id             bigserial PRIMARY KEY,
    f_block          BIGINT NOT NULL,
    f_version        BIGINT NOT NULL,
    f_object_key     text   NOT NULL,
    f_object_witness text   NOT NULL
);

DROP TABLE IF EXISTS t_prover_job_queue_cloud;
CREATE TABLE t_prover_job_queue_cloud
(
    f_id           bigserial PRIMARY KEY,
    f_job_status   INTEGER                  NOT NULL,
    f_job_priority INTEGER                  NOT NULL,
    f_job_type     TEXT                     NOT NULL,
    f_created_at   timestamp with time zone NOT NULL DEFAULT now(),
    f_version      BIGINT                   NOT NULL,
    f_updated_by   TEXT                     NOT NULL,
    f_updated_at   timestamp with time zone NOT NULL,
    f_first_block  BIGINT                   NOT NULL,
    f_last_block   BIGINT                   NOT NULL,
    f_object_key   TEXT                     NOT NULL,
    f_object_job   TEXT                     NOT NULL
);

DROP TABLE IF EXISTS t_proofs;
CREATE TABLE t_proofs
(
    f_id           bigserial PRIMARY KEY,
    f_block_number BIGINT                   NOT NULL,
    f_proof        jsonb                    NOT NULL,
    f_created_at   TIMESTAMP with time zone NOT NULL DEFAULT now()
);

DROP TABLE IF EXISTS t_witness_block_number;
CREATE TABLE t_witness_block_number
(
    f_id           bigserial PRIMARY KEY,
    f_block     BIGINT                   NOT NULL
);

DROP TABLE IF EXISTS t_proof_block_number;
CREATE TABLE t_proof_block_number
(
    f_id           bigserial PRIMARY KEY,
    f_block     BIGINT                   NOT NULL
);
CREATE TABLE t_verified_proof_block_number
(
    f_id           bigserial PRIMARY KEY,
    f_block       BIGINT                    NOT NULL
);


INSERT INTO t_witness_block_number(f_block) VALUES(${first execution trace id});

INSERT INTO t_proof_block_number(f_block) VALUES(${first execution trace id});
```  

- Program Execution Trace

Generating the mips arch based program execution trace:

https://github.com/zkMIPS/cannon-mips/tree/mipsevm-minigeth-trace#readme

- Compile MIPS VM circuit using Zokrates 

1.curl -LSfs get.zokrat.es | sh

2.cd mips_circuit/core/lib/circuit

3.zokrates compile -i mips_vm_poseidon.zok

## Deploy Verifier Contract
The compiled verifier contract is at /mips_circuit/contract/verifier/g16/verifier.
Users can use the program at /mips_circuit/contract/bin/deploy/contract/src/main.rs or other tools to deploy the contract.
Remember to replace the chain_url to your dest_chain in the 'deploy' function.


## Witness Generator

1.Setting the environment variables:

DATABASE_URL=postgresql://postgres:db@ip:port/table
DATABASE_POOL_SIZE=10
API_PROVER_PORT=8088
API_PROVER_URL=http://127.0.0.1:8088
API_PROVER_SECRET_AUTH=sample
PROVER_PROVER_HEARTBEAT_INTERVAL=1000
PROVER_PROVER_CYCLE_WAIT=500
PROVER_PROVER_REQUEST_TIMEOUT=10
PROVER_PROVER_DIE_AFTER_PROOF=false
PROVER_CORE_GONE_TIMEOUT=60000
PROVER_CORE_IDLE_PROVERS=1
PROVER_WITNESS_GENERATOR_PREPARE_DATA_INTERVAL=500
PROVER_WITNESS_GENERATOR_WITNESS_GENERATORS=2
CIRCUIT_FILE_PATH=/core/lib/circuit/out # generated by zokrates compile -i mips_vm_poseidon.zok
CIRCUIT_ABI_FILE_PATH=/mips_circuit/core/lib/circuit/abi.json # generated by zokrates compile -i mips_vm_poseidon.zok
RUST_LOG=warn  
VERIFIER_CHAIN_URL=http://127.0.0.1:8545 # chain url where the verifier contract deployed    
VERIFIER_CONTRACT_ADDRESS=0xe691cff16ddae0f79bfd0d7850ddb0ef162a9cd3 # verifier contract address  
VERIFIER_ACCOUNT=E2EA1B7352BaEfEba0D11A5FA614dF6cE9E77693 # account for send transactions  
VERIFIER_ABI_PATH=/mips_circuit/contract/verifier/g16/verifier  

2.Compile the witness generator

cd mips_circuit/core/bin/server

DATABASE_URL=postgresql://postgres:db@ip:port/table cargo build --release 

3.Running the witness generator

nohup ./target/release/server > server.output 2>&1 &

## Prover

1.Setting the environment variables:

PROVER_PROVER_HEARTBEAT_INTERVAL=1000
PROVER_PROVER_CYCLE_WAIT=500
PROVER_PROVER_REQUEST_TIMEOUT=10
PROVER_PROVER_DIE_AFTER_PROOF=false
PROVER_CORE_GONE_TIMEOUT=60000
PROVER_CORE_IDLE_PROVERS=1
PROVER_WITNESS_GENERATOR_PREPARE_DATA_INTERVAL=500
PROVER_WITNESS_GENERATOR_WITNESS_GENERATORS=2
DATABASE_URL=postgresql://postgres:<db>@<ip>:<port>/<table>
DATABASE_POOL_SIZE=10
RUST_LOG=info
CHAIN_ETH_NETWORK=rinkeby
CIRCUIT_FILE_PATH=/mips_circuit/core/lib/circuit/out # generated by: zokrates compile -i mips_vm_poseidon.zok
CIRCUIT_PROVING_KEY_PATH=/mips_circuit/core/lib/circuit/proving.key # generated by: zokrates compile -i mips_vm_poseidon.zok

2. Compile the prover 

cd mips_circuit/core/bin/prover

DATABASE_URL=postgresql://postgres:db@ip:port/table cargo build --release

3.Running the prover

nohup ./target/release/prover > prover.output 2>&1 &