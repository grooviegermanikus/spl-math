// Mark this test as BPF-only due to current `ProgramTest` limitations when
// CPIing into the system program
#![cfg(feature = "test-sbf")]

use {
    solana_program_test::*,
    solana_sdk::{signature::Signer, transaction::Transaction},
    spl_math_example::{id, instruction, processor::process_instruction},
};
use spl_math_example::instruction::SqrtAlgorithm;
use spl_math_example::processor::{CU_CORRECTION};


#[tokio::test]
async fn test_noop() {
    let mut pc = ProgramTest::new("spl_math_example", id(), processor!(process_instruction));

    pc.set_compute_max_units(1_000_000);

    let (banks_client, payer, recent_blockhash) = pc.start().await;

    let mut transaction = Transaction::new_with_payer(
        &[instruction::noop()],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);

    let result = banks_client.process_transaction_with_metadata(transaction).await.unwrap();

    let consumed_compute_units = parse_compute_units_from_logs(&result).unwrap();

    assert_eq!(consumed_compute_units, 0);
}

#[tokio::test]
async fn test_newton_sqrt_u64_max() {
    let mut pc = ProgramTest::new("spl_math_example", id(), processor!(process_instruction));

    pc.set_compute_max_units(1_000_000);

    let (banks_client, payer, recent_blockhash) = pc.start().await;

    let mut transaction = Transaction::new_with_payer(
        &[instruction::precise_sqrt(u64::MAX, SqrtAlgorithm::Newton)],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);

    let result = banks_client.process_transaction_with_metadata(transaction).await.unwrap();

    let consumed_compute_units = parse_compute_units_from_logs(&result).unwrap();
    // assert_eq!(consumed_compute_units, 363278);
    // assert_eq!(consumed_compute_units, 149571);// before improvement 2026-01-04
    assert_eq!(consumed_compute_units, 121527);
}

#[tokio::test]
async fn test_cordic_sqrt_u32_max() {
    let mut pc = ProgramTest::new("spl_math_example", id(), processor!(process_instruction));

    pc.set_compute_max_units(1_000_000);

    let (banks_client, payer, recent_blockhash) = pc.start().await;

    let mut transaction = Transaction::new_with_payer(
        &[instruction::precise_sqrt(u32::MAX as u64, SqrtAlgorithm::Cordic)],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);
    let result = banks_client.process_transaction_with_metadata(transaction).await.unwrap();
    let consumed_compute_units = parse_compute_units_from_logs(&result).unwrap();
    // assert_eq!(consumed_compute_units, 184943);
    // assert_eq!(consumed_compute_units, 64791); // before improvement 2026-01-04
    assert_eq!(consumed_compute_units, 69132);
}

#[tokio::test]
async fn test_cordic_sqrt_u64() {
    let mut pc = ProgramTest::new("spl_math_example", id(), processor!(process_instruction));

    pc.set_compute_max_units(1_000_000);

    let (banks_client, payer, recent_blockhash) = pc.start().await;

    let mut transaction =
        Transaction::new_with_payer(&[instruction::sqrt_u64(u64::MAX)], Some(&payer.pubkey()));
    transaction.sign(&[&payer], recent_blockhash);
    let result = banks_client.process_transaction_with_metadata(transaction).await.unwrap();
    let consumed_compute_units = parse_compute_units_from_logs(&result).unwrap();
    // assert_eq!(consumed_compute_units, 816); // before improvement 2026-01-04
    assert_eq!(consumed_compute_units, 566);
}



#[tokio::test]
async fn test_newton_sqrt_array() {
    let mut pc = ProgramTest::new("spl_math_example", id(), processor!(process_instruction));

    pc.set_compute_max_units(5_000_000);

    let (banks_client, payer, recent_blockhash) = pc.start().await;

    let mut transaction = Transaction::new_with_payer(
        &[instruction::precise_sqrt_array(1000.0, 100.0, SqrtAlgorithm::Newton)],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);
    let result = banks_client.process_transaction_with_metadata(transaction).await.unwrap();
    let consumed_compute_units = parse_compute_units_from_logs(&result).unwrap();
    assert_eq!(consumed_compute_units, 252473);
}

#[tokio::test]
async fn test_cordic_sqrt_array() {
    let mut pc = ProgramTest::new("spl_math_example", id(), processor!(process_instruction));

    pc.set_compute_max_units(5_000_000);

    let (banks_client, payer, recent_blockhash) = pc.start().await;

    let mut transaction = Transaction::new_with_payer(
        &[instruction::precise_sqrt_array(1000.0, 100.0, SqrtAlgorithm::Cordic)],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);
    let result = banks_client.process_transaction_with_metadata(transaction).await.unwrap();
    let consumed_compute_units = parse_compute_units_from_logs(&result).unwrap();
    assert_eq!(consumed_compute_units, 466542);
}

#[tokio::test]
async fn test_sqrt_u128() {
    let mut pc = ProgramTest::new("spl_math_example", id(), processor!(process_instruction));

    pc.set_compute_max_units(1_000_000);

    let (banks_client, payer, recent_blockhash) = pc.start().await;

    let mut transaction = Transaction::new_with_payer(
        &[instruction::sqrt_u128(u64::MAX as u128)],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);
    let result = banks_client.process_transaction_with_metadata(transaction).await.unwrap();
    let consumed_compute_units = parse_compute_units_from_logs(&result).unwrap();
    // assert_eq!(consumed_compute_units, 2905); // before improvement 2026-01-04
    assert_eq!(consumed_compute_units, 1967);
}

#[tokio::test]
async fn test_sqrt_u128_max() {
    let mut pc = ProgramTest::new("spl_math_example", id(), processor!(process_instruction));

    pc.set_compute_max_units(1_000_000);

    let (banks_client, payer, recent_blockhash) = pc.start().await;

    let mut transaction =
        Transaction::new_with_payer(&[instruction::sqrt_u128(u128::MAX)], Some(&payer.pubkey()));
    transaction.sign(&[&payer], recent_blockhash);
    let result = banks_client.process_transaction_with_metadata(transaction).await.unwrap();
    let consumed_compute_units = parse_compute_units_from_logs(&result).unwrap();
    // assert_eq!(consumed_compute_units, 5678); // before improvement 2026-01-04
    assert_eq!(consumed_compute_units, 3912);
}

#[tokio::test]
async fn test_log10_u64_max() {
    let mut pc = ProgramTest::new("spl_math_example", id(), processor!(process_instruction));

    pc.set_compute_max_units(1_000_000);

    let (banks_client, payer, recent_blockhash) = pc.start().await;

    let mut transaction = Transaction::new_with_payer(
        &[instruction::precise_log10(u64::MAX)],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);

    let result = banks_client.process_transaction_with_metadata(transaction).await.unwrap();

    let consumed_compute_units = parse_compute_units_from_logs(&result).unwrap();
    println!("log10 u64::MAX consumed {} CUs", consumed_compute_units);
    assert!(consumed_compute_units > 0);
}

#[tokio::test]
async fn test_log10_u32_max() {
    let mut pc = ProgramTest::new("spl_math_example", id(), processor!(process_instruction));

    pc.set_compute_max_units(1_000_000);

    let (banks_client, payer, recent_blockhash) = pc.start().await;

    let mut transaction = Transaction::new_with_payer(
        &[instruction::precise_log10(u32::MAX as u64)],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);
    let result = banks_client.process_transaction_with_metadata(transaction).await.unwrap();
    let consumed_compute_units = parse_compute_units_from_logs(&result).unwrap();
    println!("log10 u32::MAX consumed {} CUs", consumed_compute_units);
    assert!(consumed_compute_units > 0);
}

#[tokio::test]
async fn test_log10_array() {
    let mut pc = ProgramTest::new("spl_math_example", id(), processor!(process_instruction));

    pc.set_compute_max_units(5_000_000);

    let (banks_client, payer, recent_blockhash) = pc.start().await;

    let mut transaction = Transaction::new_with_payer(
        &[instruction::precise_log10_array(1000.0, 100.0)],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);
    let result = banks_client.process_transaction_with_metadata(transaction).await.unwrap();
    let consumed_compute_units = parse_compute_units_from_logs(&result).unwrap();
    println!("log10 array consumed {} CUs", consumed_compute_units);
    assert!(consumed_compute_units > 0);
}

#[tokio::test]
async fn test_muldiv_u64() {
    let mut pc = ProgramTest::new("spl_math_example", id(), processor!(process_instruction));

    pc.set_compute_max_units(1_000_000);

    let (banks_client, payer, recent_blockhash) = pc.start().await;

    let mut transaction =
        Transaction::new_with_payer(&[instruction::precise_muldiv(42, 84, 7)], Some(&payer.pubkey()));
    transaction.sign(&[&payer], recent_blockhash);
    let result = banks_client.process_transaction_with_metadata(transaction).await.unwrap();
    let consumed_compute_units = parse_compute_units_from_logs(&result).unwrap();
    assert_eq!(consumed_compute_units, 2976);
}

#[tokio::test]
async fn test_u64_multiply() {
    let mut pc = ProgramTest::new("spl_math_example", id(), processor!(process_instruction));

    pc.set_compute_max_units(1_000_000);

    let (banks_client, payer, recent_blockhash) = pc.start().await;

    let mut transaction =
        Transaction::new_with_payer(&[instruction::u64_multiply(42, 84)], Some(&payer.pubkey()));
    transaction.sign(&[&payer], recent_blockhash);
    let result = banks_client.process_transaction_with_metadata(transaction).await.unwrap();
    let consumed_compute_units = parse_compute_units_from_logs(&result).unwrap();
    assert_eq!(consumed_compute_units, 9);
}

#[tokio::test]
async fn test_u64_divide() {
    let mut pc = ProgramTest::new("spl_math_example", id(), processor!(process_instruction));

    pc.set_compute_max_units(1_000_000);

    let (banks_client, payer, recent_blockhash) = pc.start().await;

    let mut transaction =
        Transaction::new_with_payer(&[instruction::u64_divide(3, 1)], Some(&payer.pubkey()));
    transaction.sign(&[&payer], recent_blockhash);
    let result = banks_client.process_transaction_with_metadata(transaction).await.unwrap();
    let consumed_compute_units = parse_compute_units_from_logs(&result).unwrap();
    assert_eq!(consumed_compute_units, 9);
}

#[tokio::test]
async fn test_f32_multiply() {
    let mut pc = ProgramTest::new("spl_math_example", id(), processor!(process_instruction));

    pc.set_compute_max_units(1_000_000);

    let (banks_client, payer, recent_blockhash) = pc.start().await;

    let mut transaction = Transaction::new_with_payer(
        &[instruction::f32_multiply(1.5_f32, 2.0_f32)],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);
    let result = banks_client.process_transaction_with_metadata(transaction).await.unwrap();
    let consumed_compute_units = parse_compute_units_from_logs(&result).unwrap();
    assert_eq!(consumed_compute_units, 72);
}

#[tokio::test]
async fn test_f32_divide() {
    let mut pc = ProgramTest::new("spl_math_example", id(), processor!(process_instruction));

    pc.set_compute_max_units(1_000_000);

    let (banks_client, payer, recent_blockhash) = pc.start().await;

    let mut transaction = Transaction::new_with_payer(
        &[instruction::f32_divide(3_f32, 1.5_f32)],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);
    let result = banks_client.process_transaction_with_metadata(transaction).await.unwrap();
    let consumed_compute_units = parse_compute_units_from_logs(&result).unwrap();
    assert_eq!(consumed_compute_units, 114);
}

#[tokio::test]
async fn test_f32_exponentiate() {
    let mut pc = ProgramTest::new("spl_math_example", id(), processor!(process_instruction));

    pc.set_compute_max_units(1_000_000);

    let (banks_client, payer, recent_blockhash) = pc.start().await;

    let mut transaction = Transaction::new_with_payer(
        &[instruction::f32_exponentiate(4_f32, 2_f32)],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);
    let result = banks_client.process_transaction_with_metadata(transaction).await.unwrap();
    let consumed_compute_units = parse_compute_units_from_logs(&result).unwrap();
    assert_eq!(consumed_compute_units, 107);
}

#[tokio::test]
async fn test_f32_natural_log() {
    let mut pc = ProgramTest::new("spl_math_example", id(), processor!(process_instruction));

    pc.set_compute_max_units(1_000_000);

    let (banks_client, payer, recent_blockhash) = pc.start().await;

    let mut transaction = Transaction::new_with_payer(
        &[instruction::f32_natural_log(1_f32.exp())],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);
    let result = banks_client.process_transaction_with_metadata(transaction).await.unwrap();
    let consumed_compute_units = parse_compute_units_from_logs(&result).unwrap();
    assert_eq!(consumed_compute_units, 1900);
}

#[tokio::test]
async fn test_f32_normal_cdf() {
    let mut pc = ProgramTest::new("spl_math_example", id(), processor!(process_instruction));

    pc.set_compute_max_units(1_000_000);

    let (banks_client, payer, recent_blockhash) = pc.start().await;

    let mut transaction =
        Transaction::new_with_payer(&[instruction::f32_normal_cdf(0_f32)], Some(&payer.pubkey()));
    transaction.sign(&[&payer], recent_blockhash);
    let result = banks_client.process_transaction_with_metadata(transaction).await.unwrap();
    let consumed_compute_units = parse_compute_units_from_logs(&result).unwrap();
    assert_eq!(consumed_compute_units, 862);
}

#[tokio::test]
async fn test_f64_pow() {
    let mut pc = ProgramTest::new("spl_math_example", id(), processor!(process_instruction));

    pc.set_compute_max_units(1_000_000);

    let (banks_client, payer, recent_blockhash) = pc.start().await;

    let mut transaction = Transaction::new_with_payer(
        &[instruction::f64_pow(50_f64, 10.5_f64)],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);
    let result = banks_client.process_transaction_with_metadata(transaction).await.unwrap();
    let consumed_compute_units = parse_compute_units_from_logs(&result).unwrap();
    // not sure why this is 0
    assert_eq!(consumed_compute_units, 0);
}

#[tokio::test]
async fn test_u128_multiply() {
    let mut pc = ProgramTest::new("spl_math_example", id(), processor!(process_instruction));

    pc.set_compute_max_units(1_000_000);

    let (banks_client, payer, recent_blockhash) = pc.start().await;

    let mut transaction = Transaction::new_with_payer(
        &[instruction::u128_multiply(u64::MAX.into(), u64::MAX.into())],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();
    // TODO
}

#[tokio::test]
async fn test_u128_divide() {
    let mut pc = ProgramTest::new("spl_math_example", id(), processor!(process_instruction));

    pc.set_compute_max_units(1_000_000);

    let (banks_client, payer, recent_blockhash) = pc.start().await;

    let mut transaction = Transaction::new_with_payer(
        &[instruction::u128_divide(u128::MAX, u128::MAX / 69)],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);
    let result = banks_client.process_transaction_with_metadata(transaction).await.unwrap();
    let consumed_compute_units = parse_compute_units_from_logs(&result).unwrap();
    assert_eq!(consumed_compute_units, 310);
}

#[tokio::test]
async fn test_f64_multiply() {
    let mut pc = ProgramTest::new("spl_math_example", id(), processor!(process_instruction));

    pc.set_compute_max_units(1_000_000);

    let (banks_client, payer, recent_blockhash) = pc.start().await;

    let mut transaction = Transaction::new_with_payer(
        &[instruction::f64_multiply(f64::powf(2., 42.), 1e-4)],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);
    let result = banks_client.process_transaction_with_metadata(transaction).await.unwrap();
    let consumed_compute_units = parse_compute_units_from_logs(&result).unwrap();
    assert_eq!(consumed_compute_units, 113);
}

#[tokio::test]
async fn test_f64_divide() {
    let mut pc = ProgramTest::new("spl_math_example", id(), processor!(process_instruction));

    pc.set_compute_max_units(1_000_000);

    let (banks_client, payer, recent_blockhash) = pc.start().await;

    let mut transaction = Transaction::new_with_payer(
        &[instruction::f64_divide(f64::powf(2., 42.), 420420.6969)],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);
    let result = banks_client.process_transaction_with_metadata(transaction).await.unwrap();
    let consumed_compute_units = parse_compute_units_from_logs(&result).unwrap();
    assert_eq!(consumed_compute_units, 178);
}


// e.g. Program log: cu_bench_consumed 149570
fn parse_compute_units_from_logs(result: &BanksTransactionResultWithMetadata) -> Option<u64> {
    let logs = &result.metadata.as_ref().unwrap().log_messages;

    for log in logs {
        // only one
        if log.starts_with("Program log: cu_bench_consumed ") {
            let parts: Vec<&str> = log.split("cu_bench_consumed").collect();
            if let Some(units_str) = parts.get(1) {
                if let Ok(units) = units_str.trim().parse::<u64>() {
                    match units.checked_sub(CU_CORRECTION) {
                        Some(corrected_units) => return Some(corrected_units),
                        None => {
                            println!("Compute units underflow after correction");
                            return Some(0)
                        },
                    }
                }
            }
        }
    }
    None
}
