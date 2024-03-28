// These constants represent the RISC-V ELF and the image ID generated by risc0-build.
// The ELF is used for proving and the ID is used for verification.
use anyhow::{Context, Error};
use nexus_core::{
    agg_types::{AggregatedTransaction, InitTransaction, SubmitProofTransaction},
    db::NodeDB,
    mempool::Mempool,
    state_machine::StateMachine,
    types::{AvailHeader, HeaderStore, NexusHeader, TransactionV2, TxParamsV2, H256},
};
use prover::{NEXUS_RUNTIME_ELF, NEXUS_RUNTIME_ID};
use relayer::Relayer;
use risc0_zkvm::{
    default_executor, default_prover, serde::from_slice, ExecutorEnv, Journal, Receipt,
};
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tokio::{self, sync::Mutex};
use warp::Filter;

use std::net::SocketAddr;
use std::str::FromStr;

use crate::rpc::routes;

pub mod rpc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = NodeDB::from_path(String::from("./node_db"));
    let old_state_root: H256 = match db.get_current_root()? {
        Some(i) => i,
        None => H256::zero(),
    };
    let mut state_machine = StateMachine::new(old_state_root, "./runtime_db");
    let relayer = Relayer::new();
    let shared_relayer = Arc::new(Mutex::new(relayer));

    // let init_tx = TransactionV2 {
    //     signature: TxSignature([0; 64]),
    //     params: TxParamsV2::InitAccount(InitAccount {
    //         app_id: AppAccountId::from(AppId(1)),
    //         statement: [1; 32],
    //     }),
    // };
    // let txs: Vec<TransactionV2> = vec![init_tx];

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async move {
        let receiver = {
            let mut relayer = shared_relayer.lock().await;

            relayer.receiver()
        };
        let mempool = Mempool::new();
        let mempool_clone = mempool.clone();
        let relayer_handle = tokio::spawn(async move {
            let cloned_relayer = shared_relayer.lock().await;
            println!("Trying to start");
            println!("started from our side bro");
            cloned_relayer.start().await;
        });

        let execution_engine = tokio::spawn(async move {
            while let Some(header) = receiver.lock().await.recv().await {
                let mut old_headers: HeaderStore = match db.get(b"previous_headers") {
                    Ok(Some(i)) => i,
                    Ok(None) => HeaderStore::new(32),
                    Err(_) => break,
                };
                let (txs, index) = mempool_clone.get_current_txs().await;
                //let (txs, index) = (vec![], 0);

                match execute_batch(
                    &txs,
                    &db,
                    &mut state_machine,
                    &AvailHeader::from(&header),
                    &mut old_headers,
                ) {
                    Ok(_) => mempool_clone.clear_upto_tx(index).await,
                    Err(e) => {
                        println!("Breaking because of error {:?}", e);
                        break;
                    }
                };
            }
        });

        //Server part//
        let routes = routes(mempool);
        let cors = warp::cors()
            .allow_any_origin()
            .allow_methods(vec!["POST"])
            .allow_headers(vec!["content-type"]);
        let routes = routes.with(cors);
        let server: tokio::task::JoinHandle<()> = tokio::spawn(async move {
            println!("trying to start rpc server");
            let address =
                SocketAddr::from_str(format!("{}:{}", String::from("127.0.0.1"), 7000).as_str())
                    .context("Unable to parse host address from config")
                    .unwrap();

            println!("RPC Server running on: {:?}", &address);
            warp::serve(routes).run(address).await;
        });

        let result = tokio::try_join!(server, execution_engine, relayer_handle);

        match result {
            Ok((_, _, _)) => {
                println!("Exiting node, should not have happened.");
            }
            Err(e) => {
                println!("Exiting node, should not have happened. {:?}", e);
            }
        }
    });

    Ok(())
}

fn execute_batch(
    txs: &Vec<TransactionV2>,
    db: &NodeDB,
    state_machine: &mut StateMachine,
    header: &AvailHeader,
    header_store: &mut HeaderStore,
) -> Result<Receipt, Error> {
    //let mut cloned_old_headers = old_headers.clone();
    let state_update = state_machine.execute_batch(&header, &mut header_store.clone(), &txs)?;

    let mut env_builder = ExecutorEnv::builder();

    let zkvm_txs = txs.iter().map(|tx| {
        if let TxParamsV2::SubmitProof(submit_proof_tx) = &tx.params {
            let proof = match &tx.proof {
                Some(i) => i,
                None => unreachable!("Proof cannot be empty if submit proof tx."),
            };
            let receipt = Receipt {
                inner: risc0_zkvm::InnerReceipt::Composite(proof.clone()),
                journal: Journal {
                    //TODO: remove unwrap below
                    bytes: bincode::serialize(&submit_proof_tx.public_inputs).unwrap(),
                },
            };

            env_builder.add_assumption(receipt);
        }
    });
    //Proof generation part.
    let env = env_builder
        .write(&txs)
        .unwrap()
        .write(&state_update)
        .unwrap()
        .write(&header)
        .unwrap()
        .write(&header_store)
        .unwrap()
        .build()
        .unwrap();
    let prover = default_prover();
    let receipt = prover.prove(env, NEXUS_RUNTIME_ELF)?;
    let result: NexusHeader = from_slice(&receipt.journal.bytes).unwrap();

    //db.put(b"previous_headers", &cloned_old_headers)?;

    db.set_current_root(&result.state_root)?;
    Ok(receipt)
}

#[derive(Clone, Debug)]
pub struct BatchesToAggregate(Arc<Mutex<Vec<(Vec<InitTransaction>, AggregatedTransaction)>>>);

impl BatchesToAggregate {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(vec![])))
    }

    pub async fn add_batch(&self, batch: (Vec<InitTransaction>, AggregatedTransaction)) {
        self.0.lock().await.push(batch);
    }

    pub async fn get_next_batch(&self) -> Option<(Vec<InitTransaction>, AggregatedTransaction)> {
        Some(self.0.lock().await.first()?.clone())
    }

    pub async fn remove_first_batch(&self) {
        let mut list = &mut self.0.lock().await;

        if !list.is_empty() {
            &list.remove(0);
        } else {
            // Handle case where index exceeds the length of tx_list
            list.clear();
        }
    }
}
