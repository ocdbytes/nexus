use adapter_sdk::{state::AdapterState, types::AdapterConfig};
use demo_rollup_core::DemoProof;
use methods::{ADAPTER_ELF, ADAPTER_ID};
use nexus_core::types::{AppId, StatementDigest};

fn main() {
    let mut adapter: AdapterState<DemoProof> = AdapterState::new(
        String::from("adapter_store"),
        AdapterConfig {
            app_id: AppId(100),
            elf: ADAPTER_ELF.to_vec(),
            adapter_elf_id: StatementDigest(ADAPTER_ID),
            vk: [0u8; 32],
            rollup_start_height: 606460,
        },
    );
    let rt = tokio::runtime::Runtime::new().unwrap();

    rt.block_on(adapter.run());
}
