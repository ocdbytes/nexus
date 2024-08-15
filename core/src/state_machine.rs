use std::collections::HashMap;

use crate::state::VmState;
use crate::stf::StateTransitionFunction;
use crate::types::{
    AccountState, AppAccountId, AvailHeader, HeaderStore, StateUpdate, TransactionV2,
    TransactionZKVM, TxParamsV2, H256,
};
use crate::zkvm::traits::{ZKVMEnv, ZKVMProof};
use anyhow::{anyhow, Error};
use jmt::storage::{NodeBatch, TreeUpdateBatch};
use jmt::Version;
use serde::Serialize;
use std::fmt::Debug as DebugTrait;
use std::marker::PhantomData;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct StateMachine<Z: ZKVMEnv, P: ZKVMProof + DebugTrait + Clone> {
    stf: StateTransitionFunction<Z>,
    state: Arc<Mutex<VmState>>,
    p: PhantomData<P>, //db: NodeDB,
}

impl<Z: ZKVMEnv, P: ZKVMProof + Serialize + DebugTrait + Clone> StateMachine<Z, P> {
    pub fn new(state: Arc<Mutex<VmState>>) -> Self {
        // let chain_state_path = format!("{}/chain_state", path);
        // let state = VmState::new(root, &chain_state_path);

        StateMachine {
            stf: StateTransitionFunction::new(),
            //      db: node_db,
            p: PhantomData,
            state,
        }
    }

    pub async fn commit_state(
        &mut self,
        state_root: &H256,
        node_batch: &NodeBatch,
        version: Version,
    ) -> Result<(), Error> {
        let mut state_lock = self.state.lock().await;
        state_lock.commit(node_batch)?;

        let root = state_lock.get_root(version)?;

        //TODO: Can remove as fixed slice from below
        if (root.as_fixed_slice() != state_root.as_fixed_slice()) {
            return Err(anyhow::anyhow!("State roots do not match to commit."));
        }

        Ok(())
    }

    pub async fn execute_batch(
        &mut self,
        avail_header: &AvailHeader,
        old_nexus_headers: &HeaderStore,
        txs: &Vec<TransactionV2>,
        prev_version: Version,
    ) -> Result<(Option<TreeUpdateBatch>, StateUpdate), Error> {
        //TODO: Increment version for each update.
        let version = prev_version;
        let mut pre_state: HashMap<[u8; 32], AccountState> = HashMap::new();

        let result = {
            let state_lock = self.state.lock().await;
            txs.iter().try_for_each(|tx| {
                let app_account_id: AppAccountId = match &tx.params {
                    TxParamsV2::SubmitProof(submit_proof) => submit_proof.app_id.clone(),
                    TxParamsV2::InitAccount(init_account) => {
                        AppAccountId::from(init_account.app_id.clone())
                    }
                };

                let account_state = match state_lock.get(&app_account_id.as_h256(), 0) {
                    Ok(Some(account)) => account,
                    Err(e) => return Err(anyhow!("{:?}", e)), // Exit and return the error
                    Ok(None) => AccountState::zero(),
                };

                pre_state.insert(app_account_id.0.clone(), account_state);
                Ok(()) // Continue iterating
            })
        };

        // Check the result and return an error if necessary
        match result {
            Ok(()) => { /* Continue with the rest of your code */ }
            Err(e) => return Err(e),
        }

        //TODO: Need to simplify this part.
        let zkvm_txs: Vec<TransactionZKVM> = txs
            .iter()
            .map(|tx| {
                return TransactionZKVM {
                    params: tx.params.clone(),
                    signature: tx.signature.clone(),
                };
            })
            .collect();
        let stf_result =
            self.stf
                .execute_batch(avail_header, old_nexus_headers, &zkvm_txs, &pre_state)?;

        let mut state_lock = self.state.lock().await;

        if !stf_result.is_empty() {
            let result = state_lock.update_set(
                stf_result
                    .into_iter()
                    .map(|(key, account_state)| {
                        if account_state == AccountState::zero() {
                            (H256::from(key), None)
                        } else {
                            (H256::from(key), Some(account_state))
                        }
                    })
                    .collect(),
                version,
            )?;

            Ok((Some(result.0), result.1))
        } else {
            let root = state_lock.get_root(version)?;
            Ok((
                None,
                StateUpdate {
                    pre_state_root: root,
                    post_state_root: root,
                    pre_state: HashMap::new(),
                },
            ))
        }
    }
}
