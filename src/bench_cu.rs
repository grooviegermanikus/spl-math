use solana_instruction::{AccountMeta, Instruction};
use {
    mollusk_svm_bencher::MolluskComputeUnitBencher,
    mollusk_svm::Mollusk,
    /* ... */
};

#[test]
fn cu_bench() {

    // Optionally disable logging.
    solana_logger::setup_with("");

    /* Instruction & accounts setup ... */

    let program_id = solana_pubkey::Pubkey::new_unique();

    let mollusk = Mollusk::new(&program_id, "counter");


    let instruction = Instruction::new_with_bytes(
        program_id,
        &[],
        vec![
            // AccountMeta::new(key1, false),
            // AccountMeta::new_readonly(key2, false),
        ],
    );

    let accounts = vec![
        // (key1, Account::default()),
        // (key2, Account::default()),
    ];



    MolluskComputeUnitBencher::new(mollusk)
        .bench(("bench0", &instruction, &accounts))
        .must_pass(false)
        .out_dir("./results/cu_benches")
        .execute();


}
