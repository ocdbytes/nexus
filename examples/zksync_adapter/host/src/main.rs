use adapter_sdk::{api::NexusAPI, types::AdapterConfig};
use anyhow::{anyhow, Context, Error};
use nexus_core::db::NodeDB;
use nexus_core::state::sparse_merkle_tree::traits::Value;
use nexus_core::types::{
    AccountState, AccountWithProof, AppAccountId, AppId, InitAccount, Proof, RollupPublicInputsV2,
    StatementDigest, SubmitProof, TransactionV2, TxParamsV2, TxSignature, H256,
};
use nexus_core::zkvm::risczero::RiscZeroProof;
use nexus_core::zkvm::traits::ZKVMProof;
use proof_api::ProofAPIResponse;
use risc0_zkvm::guest::env;
use risc0_zkvm::serde::to_vec;
use risc0_zkvm::{default_prover, ExecutorEnv};
use serde::{Deserialize, Serialize};
use std::env::args;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};
use zksync_core::{L1BatchWithMetadata, MockProof, STF};
use zksync_methods::{ZKSYNC_ADAPTER_ELF, ZKSYNC_ADAPTER_ID};

mod proof_api;
// Your NodeDB struct and methods implementation here

#[derive(Clone, Serialize, Deserialize, Debug)]
struct AdapterStateData {
    last_height: u32,
    adapter_config: AdapterConfig,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Retrieve Ethereum node URL and --dev flag from command line arguments
    let args: Vec<String> = args().collect();

    if args.len() <= 2 {
        if args.len() == 2 && args[1] == "--dev" {
            eprintln!("Usage: cargo run -- <zksync_proof_api_url> [--dev]");
            return Ok(());
        }

        if args.len() < 2 {
            eprintln!("Usage: cargo run -- <zksync_proof_api_url> [--dev]");
            return Ok(());
        }
    }
    let zksync_proof_api_url = &args[1];
    let dev_flag = args.iter().any(|arg| arg == "--dev");
    let nexus_api = NexusAPI::new(&"http://127.0.0.1:7000");

    // Create or open the database
    let db_path = "db";
    let db = NodeDB::from_path(db_path);

    // If --dev flag is used, purge the database
    if dev_flag {
        db.delete(b"adapter_state_data")?;
    }

    // Retrieve or initialize the adapter state data from the database
    let adapter_state_data = if let Some(data) = db.get::<AdapterStateData>(b"adapter_state")? {
        data
    } else {
        // Initialize with default values if no data found in the database
        let adapter_config = AdapterConfig {
            app_id: AppId(100),
            elf: ZKSYNC_ADAPTER_ELF.to_vec(),
            adapter_elf_id: StatementDigest(ZKSYNC_ADAPTER_ID),
            vk: [0u8; 32],
            rollup_start_height: 606460,
        };
        AdapterStateData {
            last_height: 0,
            adapter_config,
        }
    };

    // Main loop to fetch headers and run adapter
    let mut last_height = adapter_state_data.last_height;
    let mut start_nexus_hash: Option<H256> = None;
    let stf = STF::new(ZKSYNC_ADAPTER_ID, ZKSYNC_ADAPTER_ELF.to_vec());

    println!(
        "Starting nexus with AppAccountId: {:?} \n",
        AppAccountId::from(adapter_state_data.adapter_config.app_id.clone())
    );

    let proof_api = proof_api::ProofAPI::new(zksync_proof_api_url);
    loop {
        println!("Processing L1 batch number: {}", last_height + 1);

        match proof_api.get_proof_for_l1_batch(last_height + 1).await {
            Ok(ProofAPIResponse::Found((batch_metadata, proof))) => {
                let current_height = batch_metadata.header.number.0;

                let app_account_id =
                    AppAccountId::from(adapter_state_data.adapter_config.app_id.clone());
                let account_with_proof: AccountWithProof =
                    match nexus_api.get_account_state(&app_account_id.as_h256()).await {
                        Ok(i) => i,
                        Err(e) => {
                            println!("{:?}", e);

                            continue;
                        }
                    };
                let height_on_nexus = account_with_proof.account.height;

                if adapter_state_data.adapter_config.adapter_elf_id.clone()
                    != account_with_proof.account.statement.clone()
                {
                    if account_with_proof.account != AccountState::zero() {
                        println!(
                            "❌ ❌ ❌, statement digest not matching \n{:?} \n== \n{:?}",
                            &adapter_state_data.adapter_config.adapter_elf_id,
                            &account_with_proof.account.statement
                        );
                    }
                }

                //Commenting below, as last height should be last height known to adapter, and should create proofs from that point.
                //last_height = account_with_proof.account.height;

                if account_with_proof.account == AccountState::zero() {
                    let tx = TransactionV2 {
                        signature: TxSignature([0u8; 64]),
                        params: TxParamsV2::InitAccount(InitAccount {
                            app_id: app_account_id.clone(),
                            statement: StatementDigest(ZKSYNC_ADAPTER_ID),
                            start_nexus_hash: account_with_proof.nexus_header.hash(),
                        }),
                    };

                    match nexus_api.send_tx(tx).await {
                        Ok(i) => {
                            start_nexus_hash = Some(account_with_proof.nexus_header.hash());
                            println!(
                                "Initiated account on nexus. AppAccountId: {:?} Response: {:?}",
                                &app_account_id, i,
                            )
                        }
                        Err(e) => {
                            println!("Error when iniating account: {:?}", e);

                            continue;
                        }
                    }
                }

                let (prev_proof_with_pi, init_account): (
                    Option<RiscZeroProof>,
                    Option<InitAccount>,
                ) = if last_height == 0 {
                    (
                        None,
                        Some(InitAccount {
                            app_id: app_account_id.clone(),
                            statement: StatementDigest(ZKSYNC_ADAPTER_ID),
                            start_nexus_hash: account_with_proof.nexus_header.hash(),
                        }),
                    )
                } else {
                    match db.get(&last_height.to_be_bytes())? {
                    Some(i) => (Some(i), None),
                    None => {
                        return Err(anyhow!("previous proof and metadata not found for last height as per adapter state"))
                    }
                }
                };
                let range = match nexus_api.get_range().await {
                    Ok(i) => i,
                    Err(e) => {
                        println!("{:?}", e);
                        continue;
                    }
                };

                if range.is_empty() {
                    println!("Nexus does not have a valid range, retrying.");

                    continue;
                }

                let recursive_proof = stf.create_recursive_proof(
                    prev_proof_with_pi,
                    init_account,
                    proof,
                    batch_metadata.clone(),
                    range[0],
                )?;

                match recursive_proof.0.verify(ZKSYNC_ADAPTER_ID) {
                    Ok(()) => {
                        println!("Proof verification successful");

                        ()
                    }
                    Err(e) => return Err(anyhow!("Proof generated is invalid.")),
                }

                if current_height > height_on_nexus {
                    let public_inputs = RollupPublicInputsV2 {
                        nexus_hash: range[0],
                        state_root: H256::from(
                            batch_metadata.metadata.root_hash.as_fixed_bytes().clone(),
                        ),
                        //TODO: remove unwrap
                        height: current_height,
                        start_nexus_hash: start_nexus_hash.unwrap_or_else(|| {
                            H256::from(account_with_proof.account.start_nexus_hash)
                        }),
                        app_id: app_account_id.clone(),
                        img_id: StatementDigest(ZKSYNC_ADAPTER_ID),
                    };

                    let tx = TransactionV2 {
                        signature: TxSignature([0u8; 64]),
                        params: TxParamsV2::SubmitProof(SubmitProof {
                            app_id: app_account_id.clone(),
                            nexus_hash: range[0],
                            state_root: public_inputs.state_root.clone(),
                            proof: match recursive_proof.clone().try_into() {
                                Ok(i) => i,
                                Err(e) => {
                                    println!("Unable to serialise proof: {:?}", e);

                                    continue;
                                }
                            },
                            height: public_inputs.height,
                        }),
                    };
                    match nexus_api.send_tx(tx).await {
                        Ok(i) => {
                            println!(
                                "Submitted proof to update state root on nexus. AppAccountId: {:?} Response: {:?} Stateroot: {:?}",
                                &app_account_id, i, &public_inputs.state_root
                            )
                        }
                        Err(e) => {
                            println!("Error when iniating account: {:?}", e);

                            continue;
                        }
                    }
                } else {
                    println!("Current height is lesser than height on nexus. current height: {} nexus height: {}", current_height, height_on_nexus);
                }

                // Persist adapter state data to the database
                db.put(&current_height.to_be_bytes(), &recursive_proof)?;
                db.put(
                    b"adapter_state_data",
                    &AdapterStateData {
                        last_height: last_height + 1,
                        adapter_config: adapter_state_data.adapter_config.clone(),
                    },
                )?;

                last_height = current_height;

                if last_height < height_on_nexus {
                    //No need to wait, can continue loop, as still need to catch up with latest height.
                    continue;
                }
            }
            Ok(ProofAPIResponse::Pending) => {
                println!("Got no header, sleeping for 10 seconds to try fetching");
            }
            Ok(ProofAPIResponse::Pruned) => {
                println!("Error fetching proof - Already pruned. Need to fetch from indexer");

                return Err(anyhow!("Error fetching proof - Already pruned. Need to fetch from indexer which is not implemented, exiting"));
            }
            Err(e) => {
                println!("Err while fetching proof {:?}", e);
            }
        }

        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}
