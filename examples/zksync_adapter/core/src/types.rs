use crate::constants::MAX_NUMBER_OF_BLOBS;
use serde::{Deserialize, Serialize};
use zksync_basic_types::{
    ethabi::ethereum_types::Bloom as H2048, ethabi::Bytes, protocol_version::ProtocolVersionId,
    web3::keccak256, Address, H160, H256, U256,
};
#[cfg(any(feature = "native"))]
use zksync_types::commitment::SerializeCommitment;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LogProcessingOutput {
    pub number_of_layer1_txs: U256,
    pub chained_priority_txs_hash: Bytes,
    pub previous_batch_hash: H256,
    pub pubdata_hash: H256,
    pub state_diff_hash: H256,
    pub l2_logs_tree_root: H256,
    pub packed_batch_and_l2_block_timestamp: U256,
    pub blob_hashes: [H256; MAX_NUMBER_OF_BLOBS as usize],
}

pub type H256Vec = [H256; MAX_NUMBER_OF_BLOBS as usize];

impl LogProcessingOutput {
    pub fn new() -> LogProcessingOutput {
        Self {
            number_of_layer1_txs: U256::zero(),
            chained_priority_txs_hash: vec![],
            previous_batch_hash: H256::zero(),
            pubdata_hash: H256::zero(),
            state_diff_hash: H256::zero(),
            l2_logs_tree_root: H256::zero(),
            packed_batch_and_l2_block_timestamp: U256::zero(),
            blob_hashes: [H256::zero(); MAX_NUMBER_OF_BLOBS as usize],
        }
    }
}
#[derive(Clone, Debug, Deserialize, Serialize)]

pub struct CommitBatchInfo {
    pub batch_number: u64,
    pub timestamp: u64,
    pub index_repeated_storage_changes: u64,
    pub new_state_root: H256,
    pub number_of_layer1_txs: U256,
    pub priority_operations_hash: H256,
    pub bootloader_heap_initial_contents_hash: H256,
    pub events_queue_state_hash: H256,
    pub system_logs: Bytes,
    pub pubdata_commitments: Bytes,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct L1BatchWithMetadata {
    pub header: L1BatchHeader,
    pub metadata: L1BatchMetadata,
    pub raw_published_factory_deps: Vec<Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct L1BatchHeader {
    /// Numeric ID of the block. Starts from 1, 0 block is considered genesis block and has no transactions.
    pub number: L1BatchNumber,
    /// Timestamp when block was first created.
    pub timestamp: u64,
    /// Total number of processed priority operations in the block
    pub l1_tx_count: u16,
    /// Total number of processed txs that was requested offchain
    pub l2_tx_count: u16,
    /// The data of the processed priority operations hash which must be sent to the smart contract.
    pub priority_ops_onchain_data: Vec<PriorityOpOnchainData>,
    /// All user generated L2 -> L1 logs in the block.
    pub l2_to_l1_logs: Vec<UserL2ToL1Log>,
    /// Preimages of the hashes that were sent as value of L2 logs by special system L2 contract.
    pub l2_to_l1_messages: Vec<Vec<u8>>,
    /// Bloom filter for the event logs in the block.
    pub bloom: H2048,
    /// Hashes of contracts used this block
    pub used_contract_hashes: Vec<U256>,
    pub base_system_contracts_hashes: BaseSystemContractsHashes,
    /// System logs are those emitted as part of the Vm execution.
    pub system_logs: Vec<SystemL2ToL1Log>,
    /// Version of protocol used for the L1 batch.
    pub protocol_version: Option<ProtocolVersionId>,
    pub pubdata_input: Option<Vec<u8>>,
}

impl L1BatchHeader {
    pub fn new(
        number: L1BatchNumber,
        timestamp: u64,
        base_system_contracts_hashes: BaseSystemContractsHashes,
        protocol_version: ProtocolVersionId,
    ) -> L1BatchHeader {
        Self {
            number,
            timestamp,
            l1_tx_count: 0,
            l2_tx_count: 0,
            priority_ops_onchain_data: vec![],
            l2_to_l1_logs: vec![],
            l2_to_l1_messages: vec![],
            bloom: H2048::default(),
            used_contract_hashes: vec![],
            base_system_contracts_hashes,
            system_logs: vec![],
            protocol_version: Some(protocol_version),
            pubdata_input: Some(vec![]),
        }
    }

    /// Creates a hash of the priority ops data.
    pub fn priority_ops_onchain_data_hash(&self) -> H256 {
        let mut rolling_hash: H256 = keccak256(&[]).into();
        for onchain_data in &self.priority_ops_onchain_data {
            let mut preimage = Vec::new();
            preimage.extend(rolling_hash.as_fixed_bytes());
            preimage.extend(onchain_data.onchain_data_hash.as_fixed_bytes());

            rolling_hash = keccak256(&preimage).into();
        }

        rolling_hash
    }

    pub fn tx_count(&self) -> usize {
        (self.l1_tx_count + self.l2_tx_count) as usize
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct L1BatchMetadata {
    pub root_hash: H256,
    pub rollup_last_leaf_index: u64,
    pub initial_writes_compressed: Option<Vec<u8>>,
    pub repeated_writes_compressed: Option<Vec<u8>>,
    pub commitment: H256,
    pub l2_l1_merkle_root: H256,
    pub block_meta_params: L1BatchMetaParameters,
    pub aux_data_hash: H256,
    pub meta_parameters_hash: H256,
    pub pass_through_data_hash: H256,
    /// The commitment to the final events queue state after the batch is committed.
    /// Practically, it is a commitment to all events that happened on L2 during the batch execution.
    pub events_queue_commitment: Option<H256>,
    /// The commitment to the initial heap content of the bootloader. Practically it serves as a
    /// commitment to the transactions in the batch.
    pub bootloader_initial_content_commitment: Option<H256>,
    pub state_diffs_compressed: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct L1BatchMetaParameters {
    pub zkporter_is_available: bool,
    pub bootloader_code_hash: H256,
    pub default_aa_code_hash: H256,
    pub protocol_version: Option<ProtocolVersionId>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct L1BatchNumber(pub u32);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PriorityOpOnchainData {
    pub layer_2_tip_fee: U256,
    pub onchain_data_hash: H256,
}

// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
// pub struct U256(pub [u64; 4]);

// #[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default, Eq)]
// pub struct H256(pub [u8; 32]);

// #[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default, Eq)]
// pub struct H160(pub [u8; 20]);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default, Eq)]
pub struct UserL2ToL1Log(pub L2ToL1Log);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default, Eq)]
pub struct L2ToL1Log {
    pub shard_id: u8,
    pub is_service: bool,
    pub tx_number_in_block: u16,
    pub sender: Address,
    pub key: H256,
    pub value: H256,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
pub struct BaseSystemContractsHashes {
    pub bootloader: H256,
    pub default_aa: H256,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default, Eq)]
pub struct SystemL2ToL1Log(pub L2ToL1Log);

#[cfg(any(feature = "native"))]
impl SerializeCommitment for L2ToL1Log {
    const SERIALIZED_SIZE: usize = 88;

    fn serialize_commitment(&self, buffer: &mut [u8]) {
        buffer[0] = self.shard_id;
        buffer[1] = self.is_service as u8;
        buffer[2..4].copy_from_slice(&self.tx_number_in_block.to_be_bytes());
        buffer[4..24].copy_from_slice(self.sender.as_bytes());
        buffer[24..56].copy_from_slice(self.key.as_bytes());
        buffer[56..88].copy_from_slice(self.value.as_bytes());
    }
}

#[cfg(any(feature = "native"))]
impl SerializeCommitment for UserL2ToL1Log {
    const SERIALIZED_SIZE: usize = L2ToL1Log::SERIALIZED_SIZE;

    fn serialize_commitment(&self, buffer: &mut [u8]) {
        self.0.serialize_commitment(buffer);
    }
}

#[cfg(any(feature = "native"))]
impl SerializeCommitment for SystemL2ToL1Log {
    const SERIALIZED_SIZE: usize = L2ToL1Log::SERIALIZED_SIZE;

    fn serialize_commitment(&self, buffer: &mut [u8]) {
        self.0.serialize_commitment(buffer);
    }
}
