use std::io::{Read, Write};
use std::path::PathBuf;
use std::{io, process};
use mollusk_svm::file;
use solana_instruction::{AccountMeta, Instruction};
use solana_pubkey::Pubkey;
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

    // build counter.so from litesvm/crates/litesvm/test_programs/target/sbpf-solana-solana/release/counter.so
    // cargo-build-sbf
    let mut mollusk = Mollusk::default();

    let elf_file_data = build_and_load_program_elf();

    mollusk.add_program_with_elf_and_loader(&program_id, &elf_file_data, &solana_sdk_ids::bpf_loader_upgradeable::id());
    // new(&program_id, "counter");


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


    let result = mollusk.process_instruction(&instruction, &accounts);

    assert_eq!(result.compute_units_consumed, 1548);

}

fn build_and_load_program_elf() -> Vec<u8> {

    assert!(PathBuf::from("test_programs").is_dir(), "test_programs directory does not exist or is not a directory");

    // note: we use v0 here
    let output = process::Command::new("cargo-build-sbf")
        .arg("--offline")
        .arg("--arch")
        .arg("v3")
        .current_dir("test_programs/counter")
        .output()
        .expect("Failed to build the program");

    println!("status: {}", output.status);
    io::stdout().write_all(&output.stdout).unwrap();
    io::stderr().write_all(&output.stderr).unwrap();

    let elf_file = PathBuf::from("test_programs/counter/target/sbpfv3-solana-solana/release/counter.so");
    let mut file = std::fs::File::open(elf_file).expect("Failed to open ELF file");

    let _build_elapsed = file.metadata().unwrap().created().unwrap().elapsed().unwrap();

    let mut file_data = Vec::new();
    file.read_to_end(&mut file_data).unwrap();

    file_data
}