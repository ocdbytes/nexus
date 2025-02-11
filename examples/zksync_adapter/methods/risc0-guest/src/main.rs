#![no_main]
use nexus_core::zkvm::risczero::ZKVM;
use risc0_zkvm::guest::env;
use zksync_core::prover::run;
risc0_zkvm::guest::entry!(main);

fn main() {
    let before_cycle = env::cycle_count();
    run::<ZKVM>();
    let after_cycle = env::cycle_count();
    println!("Proving took {} cycles", after_cycle - before_cycle);
}
