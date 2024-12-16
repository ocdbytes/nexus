use anyhow::{anyhow, Context, Error};
pub use avail_subxt::Header;
use jmt::storage::TreeUpdateBatch;
use nexus_core::{
    db::{BatchTransaction, NodeDB},
    mempool::Mempool,
    state::VmState,
    state_machine::StateMachine,
    traits::NexusTransaction,
    types::{
        AvailHeader, HeaderStore, NexusBlock, NexusBlockWithPointers, NexusHeader,
        Proof as NexusProof, Transaction, TransactionResult, TransactionStatus,
        TransactionWithStatus, TransactionZKVM, TxParams, H256,
    },
    zkvm::{
        traits::{ZKVMEnv, ZKVMProof, ZKVMProver},
        ProverMode,
    },
};
use serde_json;
use std::{collections::HashMap, mem, thread};
use tokio::fs;

use crate::rpc::routes;
#[cfg(any(feature = "risc0"))]
use nexus_core::zkvm::risczero::{RiscZeroProof as Proof, RiscZeroProver as Prover, ZKVM};

#[cfg(any(feature = "sp1"))]
use nexus_core::zkvm::sp1::{Sp1Proof as Proof, Sp1Prover as Prover, SP1ZKVM as ZKVM};

#[cfg(any(feature = "risc0"))]
use prover::{NEXUS_RUNTIME_ELF, NEXUS_RUNTIME_ID};
pub use relayer::{Relayer, SimpleRelayer};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use std::{env::args, fmt::Debug as DebugTrait};
use tokio::sync::{mpsc::UnboundedReceiver, watch, Mutex};
use tokio::time::{sleep, Duration};
use warp::Filter;

pub mod rpc;
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AvailToNexusPointer {
    number: u32,
    nexus_hash: H256,
}

pub fn setup_components(db_path: &str) -> (Arc<Mutex<NodeDB>>, Arc<Mutex<VmState>>) {
    // Construct the node_db path directly as a string
    let node_db_path = format!("{}/node_db", db_path);
    let node_db = NodeDB::from_path(&node_db_path);

    // Use the runtime_db path directly as a string
    let runtime_db_path = format!("{}/runtime_db", db_path);
    let state = Arc::new(Mutex::new(VmState::new(&runtime_db_path)));

    (Arc::new(Mutex::new(node_db)), state)
}

pub async fn relayer_handle(
    relayer_mutex: Arc<Mutex<impl Relayer + Send + 'static>>,
    node_db_mutex: Arc<Mutex<NodeDB>>,
    mut shutdown_rx: watch::Receiver<bool>,
) -> () {
    let relayer = relayer_mutex.lock().await;
    let start_height: u32 = {
        let db_lock = node_db_mutex.lock().await;

        let avail_hash: Option<H256> = match db_lock.get::<HeaderStore>(b"previous_headers") {
            //Can do unwrap below as an empty store would not be stored.
            Ok(Some(i)) => Some(i.first().unwrap().avail_header_hash),
            Ok(None) => None,
            Err(_) => panic!("Could not access db"),
        };

        if let Some(hash) = avail_hash {
            let height = match db_lock.get::<AvailToNexusPointer>(hash.as_slice()) {
              Ok(Some(i)) => i.number,
              Ok(None) => panic!("Node DB error. Cannot find mapping to avail -> nexus block for already processed block"),
              Err(e) => {
                  println!("{:?}", e);

                  panic!("Node DB error. Cannot find mapping to avail -> nexus block")
              },
          } + 1;

            height
        } else {
            10000
        }
    };

    tokio::select! {
        _ = relayer.start(start_height) => {
            println!("Relayer start function exited");
        }
        _ = shutdown_rx.changed() => {
            if *shutdown_rx.borrow() {
                println!("Shutdown signal received. Stopping relayer handle...");
                relayer.stop();
            }
        }
    }

    println!("Exited relayer handle");
}

async fn execute_batch<
    Z: ZKVMProver<P>,
    P: ZKVMProof + Serialize + Clone + DebugTrait + TryFrom<NexusProof>,
    E: ZKVMEnv,
>(
    txs: &Vec<Transaction>,
    state_machine: &mut StateMachine<E, P>,
    header: &AvailHeader,
    header_store: &mut HeaderStore,
    prover_mode: ProverMode,
) -> Result<(P, NexusHeader, HashMap<H256, bool>, Option<TreeUpdateBatch>), Error>
where
    <P as TryFrom<NexusProof>>::Error: std::fmt::Debug,
{
    let (tree_update_batch, state_update, tx_result): (
        Option<jmt::storage::TreeUpdateBatch>,
        nexus_core::types::StateUpdate,
        HashMap<H256, bool>,
    ) = state_machine
        .execute_batch(&header, header_store, &txs)
        .await?;

    let (proof, result) = {
        #[cfg(any(feature = "sp1"))]
        let NEXUS_RUNTIME_ELF: &[u8] =
            include_bytes!("../../prover/sp1-guest/elf/riscv32im-succinct-zkvm-elf");

        let mut zkvm_prover = Z::new(NEXUS_RUNTIME_ELF.to_vec(), prover_mode);

        let zkvm_txs: Result<Vec<TransactionZKVM>, anyhow::Error> = txs
            .iter()
            .map(|tx| {
                if let TxParams::SubmitProof(submit_proof_tx) = &tx.params {
                    //TODO: Remove transactions that error out from mempool
                    let proof = submit_proof_tx.proof.clone();
                    let receipt: P = P::try_from(proof).unwrap();
                    zkvm_prover.add_proof_for_recursion(receipt).unwrap();
                }

                Ok(TransactionZKVM {
                    signature: tx.signature.clone(),
                    params: tx.params.clone(),
                })
            })
            .collect();

        let zkvm_txs = zkvm_txs?;

        zkvm_prover.add_input(&zkvm_txs).unwrap();
        zkvm_prover.add_input(&state_update).unwrap();
        zkvm_prover.add_input(&header).unwrap();
        zkvm_prover.add_input(&header_store).unwrap();
        let mut proof = zkvm_prover.prove()?;

        let result: NexusHeader = proof.public_inputs()?;
        (proof, result)
    };

    header_store.push_front(&result);

    Ok((proof, result, tx_result, tree_update_batch))
}

pub async fn execution_engine_handle(
    receiver: Arc<Mutex<UnboundedReceiver<Header>>>,
    node_db: Arc<Mutex<NodeDB>>,
    mempool: Mempool,
    mut state_machine: StateMachine<ZKVM, Proof>,
    prover_mode: ProverMode,
    mut shutdown_rx: watch::Receiver<bool>,
    state: Arc<Mutex<VmState>>,
) -> Result<(), anyhow::Error> {
    const MAX_HEADERS: usize = 5;
    let mut header_array: Vec<Header> = Vec::new();
    loop {
        if *shutdown_rx.borrow() {
            println!("Shutdown signal received. Stopping execution engine...");
            break;
        }

        let header_opt = {
            let mut lock = receiver.lock().await;
            lock.try_recv().ok()
        };

        if let Some(header) = header_opt {
            println!("Processing header {}", header.number);
            header_array.push(header.clone());

            // Ensure the array does not exceed the maximum size
            // if header_array.len() >= MAX_HEADERS {
            //     // Write the headers to the file
            //     let json_path = "./tests/data/avail_headers.json";
            //     if let Err(e) =
            //         fs::write(json_path, serde_json::to_string(&header_array).unwrap()).await
            //     {
            //         eprintln!("Failed to write headers to file: {:?}", e);
            //     }
            // }

            let mut old_headers: HeaderStore = {
                let db_lock = node_db.lock().await;
                match db_lock.get(b"previous_headers") {
                    Ok(Some(i)) => i,
                    Ok(None) => HeaderStore::new(32),
                    Err(_) => {
                        return Err(anyhow!(
                            "DB Call failed to get previous headers. Restart required."
                        ));
                    }
                }
            };
            // let mut txs = Vec::new();
            // let mut index = None;

            // // Continuously fetch transactions until the length is non-zero
            // while txs.is_empty() {
            //     let (current_txs, current_index) = mempool.get_current_txs().await;
            //     txs = current_txs;
            //     index = current_index;

            //     if header.number == 10000 {
            //         break;
            //     }
            // }

            let (txs, index) = mempool.get_current_txs().await;

            println!(
                "Number of txs for height {} -- {}",
                header.number,
                txs.len()
            );

            match execute_batch::<Prover, Proof, ZKVM>(
                &txs,
                &mut state_machine,
                &AvailHeader::from(&header),
                &mut old_headers,
                prover_mode.clone(),
            )
            .await
            {
                Ok((_, result, tx_result, tree_update_batch)) => {
                    //The execute_batch method on state machine would have updated the version in the storage.
                    let updated_version = state.lock().await.get_version(false)?;

                    save_batch_information(
                        &node_db,
                        &mempool,
                        &mut state_machine,
                        ProcessedBatchInfo {
                            avail_header: &header,
                            header: &result,
                            txs_result: &tx_result,
                            tree_update_batch,
                            txs: &txs,
                            mempool_index: &index,
                            updated_header_store: &old_headers,
                            jmt_version: match updated_version {
                                Some(i) => i,
                                None => 0,
                            },
                        },
                    )
                    .await?;
                    // let nexus_hash: H256 = result.hash();
                    // let mut batch_transaction = BatchTransaction::new();

                    // batch_transaction.put(b"previous_headers", &old_headers);
                    // batch_transaction.put(
                    //     result.avail_header_hash.as_slice(),
                    //     &AvailToNexusPointer {
                    //         number: header.number,
                    //         nexus_hash: nexus_hash.clone(),
                    //     },
                    // );
                    // for (tx_hash, success) in tx_result.iter() {
                    //     if let Some(tx) = txs.iter().find(|t| t.hash() == *tx_hash) {
                    //         batch_transaction.put(tx_hash.as_slice(), tx)?;
                    //     }
                    // }
                    // batch_transaction.put(nexus_hash.as_slice(), &result);
                    // let db_lock = node_db.lock().await;
                    // db_lock.put_batch(batch_transaction)?;
                    // // db_lock.put(b"previous_headers", &old_headers).unwrap();
                    // // db_lock
                    // //     .put(
                    // //         result.avail_header_hash.as_slice(),
                    // //         &AvailToNexusPointer {
                    // //             number: header.number,
                    // //             nexus_hash: nexus_hash.clone(),
                    // //         },
                    // //     )
                    // //     .unwrap();
                    // // db_lock.put(nexus_hash.as_slice(), &result).unwrap();

                    // db_lock.set_current_root(&result.state_root).unwrap();
                    // if let Some(i) = index {
                    //     mempool.clear_upto_tx(i).await;
                    // }

                    println!(
                        "✅ Processed batch: {:?}, avail height: {:?}",
                        result, header.number
                    );
                }
                Err(e) => {
                    println!("Breaking because of error {:?}", e);
                    return Err(e);
                }
            }
        } else {
            // No header available; allow loop to continue
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    println!("Exited execution handle");

    Ok(())
}

pub struct ProcessedBatchInfo<'a> {
    avail_header: &'a Header,
    header: &'a NexusHeader,
    txs_result: &'a HashMap<H256, bool>,
    tree_update_batch: Option<TreeUpdateBatch>,
    txs: &'a Vec<Transaction>,
    mempool_index: &'a Option<usize>,
    updated_header_store: &'a HeaderStore,
    jmt_version: u64,
}

pub async fn save_batch_information<'a>(
    node_db: &Arc<Mutex<NodeDB>>,
    mempool: &Mempool,
    state_machine: &mut StateMachine<ZKVM, Proof>,
    processed_batch_info: ProcessedBatchInfo<'a>,
) -> Result<(), Error> {
    match processed_batch_info.tree_update_batch {
        Some(i) => {
            state_machine
                .commit_state(&processed_batch_info.header.state_root, &i.node_batch)
                .await?;
        }
        None => (),
    }
    let nexus_hash: H256 = processed_batch_info.header.hash();
    let mut batch_transaction = BatchTransaction::new();

    batch_transaction.put(
        b"previous_headers",
        &processed_batch_info.updated_header_store,
    );
    batch_transaction.put(
        processed_batch_info.header.avail_header_hash.as_slice(),
        &AvailToNexusPointer {
            number: processed_batch_info.avail_header.number,
            nexus_hash: nexus_hash.clone(),
        },
    );

    let mut txs_result_vec: Vec<TransactionResult> = vec![];

    for (tx_hash, success) in processed_batch_info.txs_result.iter() {
        let db_lock = node_db.lock().await;
        let mut tx: TransactionWithStatus =
            match db_lock.get::<TransactionWithStatus>(tx_hash.as_slice())? {
                Some(i) => i,
                None => return Err(anyhow!("Tx not in db to modify.")),
            };

        tx.block_hash = Some(nexus_hash.clone());
        tx.status = if success.clone() {
            TransactionStatus::Successful
        } else {
            TransactionStatus::Failed
        };

        batch_transaction.put(tx_hash.as_slice(), &tx);
        txs_result_vec.push(TransactionResult {
            hash: tx_hash.clone(),
            result: success.clone(),
        });
    }
    batch_transaction.put(nexus_hash.as_slice(), &processed_batch_info.header);
    batch_transaction.put(
        &[nexus_hash.as_slice(), b"-block"].concat(),
        &NexusBlockWithPointers {
            block: NexusBlock {
                header: processed_batch_info.header.clone(),
                transactions: txs_result_vec,
            },
            jmt_version: processed_batch_info.jmt_version,
        },
    );
    let db_lock = node_db.lock().await;
    db_lock.put_batch(batch_transaction)?;

    db_lock
        .set_current_root(&processed_batch_info.header.state_root)
        .unwrap();
    if let Some(i) = processed_batch_info.mempool_index {
        mempool.clear_upto_tx(i.clone()).await;
    };

    Ok(())
}

pub fn run_server(
    mempool: Mempool,
    node_db: Arc<Mutex<NodeDB>>,
    state: Arc<Mutex<VmState>>,
    mut shutdown_rx: watch::Receiver<bool>,
    port: u32,
) -> tokio::task::JoinHandle<()> {
    let routes = routes(mempool, node_db, state.clone());
    let cors = warp::cors()
        .allow_any_origin()
        .allow_methods(vec!["POST"])
        .allow_headers(vec!["content-type"]);
    let routes = routes.with(cors);
    tokio::spawn(async move {
        let address =
            SocketAddr::from_str(format!("{}:{}", String::from("0.0.0.0"), port).as_str())
                .context("Unable to parse host address from config")
                .unwrap();

        println!("RPC Server running on: {:?}", &address);

        let (_, server) = warp::serve(routes).bind_with_graceful_shutdown(address, async move {
            shutdown_rx.changed().await.ok();
            println!("Shutdown signal received. Stopping server...");
        });

        server.await;

        println!("Exited server handle");
    })
}

pub async fn run_nexus(
    relayer_mutex: Arc<Mutex<impl Relayer + Send + 'static>>,
    node_db: Arc<Mutex<NodeDB>>,
    mut state_machine: StateMachine<ZKVM, Proof>,
    (prover_mode, server_port): (ProverMode, u32),
    state: Arc<Mutex<VmState>>,
    mut shutdown_rx: watch::Receiver<bool>,
) -> Result<(), Error> {
    let mut shutdown_rx_1 = shutdown_rx.clone();
    let mut shutdown_rx_2 = shutdown_rx.clone();
    let db_clone = node_db.clone();
    let db_clone_2 = node_db.clone();
    let state_2 = state.clone();

    let receiver = {
        let mut relayer = relayer_mutex.lock().await;

        relayer.receiver()
    };
    let mempool = Mempool::new(node_db.clone());
    let mempool_clone = mempool.clone();
    let relayer_handle = tokio::spawn(async move {
        relayer_handle(relayer_mutex, db_clone_2, shutdown_rx_1.clone()).await
    });

    let execution_engine = tokio::spawn(async move {
        execution_engine_handle(
            receiver,
            node_db,
            mempool_clone,
            state_machine,
            prover_mode,
            shutdown_rx_2.clone(),
            state_2.clone(),
        )
        .await
    });

    let server_handle = run_server(mempool, db_clone, state, shutdown_rx, server_port);

    let result = tokio::try_join!(server_handle, execution_engine, relayer_handle);

    match result {
        Ok((_, execution_engine_result, _)) => {
            println!("Exited node gracefully");

            match execution_engine_result {
                Ok(()) => Ok(()),

                Err(e) => {
                    println!("Execution engine handle has error");
                    Err(e)
                }
            }
        }
        Err(e) => {
            println!(
                "Exiting node with an error, should not have happened. {:?}",
                e
            );

            Err(anyhow!(e))
        }
    }
}
