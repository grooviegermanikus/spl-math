#![allow(clippy::arithmetic_side_effects)]
//! Program state processor

use crate::instruction::SqrtAlgorithm;
use solana_program::compute_units::sol_remaining_compute_units;
use spl_math::precise_number::PreciseNumber256D18;
use {
    crate::{
        approximations::{f32_normal_cdf, sqrt},
        instruction::MathInstruction,
        precise_number::PreciseNumber,
    },
    borsh::BorshDeserialize,
    solana_program::{
        account_info::AccountInfo, entrypoint::ProgramResult, log::sol_log_compute_units, msg,
        pubkey::Pubkey,
    },
};

/// Compensate for compute units used syscall overhead; checked by Noop instruction
pub const CU_CORRECTION: u64 = 102;

/// u64_multiply
#[inline(never)]
fn u64_multiply(multiplicand: u64, multiplier: u64) -> u64 {
    multiplicand * multiplier
}

/// u64_divide
#[inline(never)]
fn u64_divide(dividend: u64, divisor: u64) -> u64 {
    dividend / divisor
}

/// f32_multiply
#[inline(never)]
fn f32_multiply(multiplicand: f32, multiplier: f32) -> f32 {
    multiplicand * multiplier
}

/// f32_divide
#[inline(never)]
fn f32_divide(dividend: f32, divisor: f32) -> f32 {
    dividend / divisor
}

/// f32_exponentiate
#[inline(never)]
fn f32_exponentiate(base: f32, exponent: f32) -> f32 {
    base.powf(exponent)
}

/// f32_natural_log
#[inline(never)]
fn f32_natural_log(argument: f32) -> f32 {
    argument.ln()
}

/// u128_multiply
#[inline(never)]
fn u128_multiply(multiplicand: u128, multiplier: u128) -> u128 {
    multiplicand * multiplier
}

/// u128_divide
#[inline(never)]
fn u128_divide(dividend: u128, divisor: u128) -> u128 {
    dividend / divisor
}

/// f64_multiply
#[inline(never)]
fn f64_multiply(multiplicand: f64, multiplier: f64) -> f64 {
    multiplicand * multiplier
}

/// f64_divide
#[inline(never)]
fn f64_divide(dividend: f64, divisor: f64) -> f64 {
    dividend / divisor
}

/// Instruction processor
pub fn process_instruction(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    input: &[u8],
) -> ProgramResult {
    let instruction = MathInstruction::try_from_slice(input).unwrap();
    match instruction {
        MathInstruction::PreciseSquareRoot { radicand: radicands, algorithm } => {
            msg!("Calculating square root using PreciseNumber");
            let radicands: Vec<PreciseNumber256D18> = radicands.iter().map(|x| PreciseNumber256D18::new_from_f64(*x).unwrap()).collect();

                match algorithm {
                    SqrtAlgorithm::Newton => {
                        sol_log_compute_units();
                        let cu_before = sol_remaining_compute_units();
                        // collect sum to avoid optimizer removing the loop
                        let mut sum_result: i128 = 0;
                        for radicand in radicands {
                            let result = radicand.sqrt_newton().unwrap();
                            sum_result += result.value.0[0] as i128;
                        }
                        let cu_after = sol_remaining_compute_units();
                        sol_log_compute_units();
                        msg!("cu_bench_consumed {}", cu_before - cu_after);
                        msg!("{}", sum_result);
                    }
                    SqrtAlgorithm::Cordic => {
                        sol_log_compute_units();
                        let cu_before = sol_remaining_compute_units();
                        // collect sum to avoid optimizer removing the loop
                        let mut sum_result: i128 = 0;
                        for radicand in radicands {
                            let result = radicand.sqrt_cordic().unwrap();
                            sum_result += result.value.0[0] as i128;
                        }
                        let cu_after = sol_remaining_compute_units();
                        sol_log_compute_units();
                        msg!("cu_bench_consumed {}", cu_before - cu_after);
                        msg!("{}", sum_result);
                    }
                }

            Ok(())
        }
        MathInstruction::PreciseMulDiv { val, num, denom } => {
            msg!("Calculating muldiv using PreciseNumber");
            let val = PreciseNumber::new(val as u128).unwrap();
            let num = PreciseNumber::new(num as u128).unwrap();
            let denom = PreciseNumber::new(denom as u128).unwrap();
            sol_log_compute_units();
            let cu_before = sol_remaining_compute_units();
            let result = val.mul_div_floor(num, denom).unwrap().to_imprecise().unwrap();
            let cu_after = sol_remaining_compute_units();
            sol_log_compute_units();
            msg!("cu_bench_consumed {}", cu_before - cu_after);
            msg!("{}", result);
            Ok(())
        }
        MathInstruction::SquareRootU64 { radicand } => {
            msg!("Calculating u64 square root");
            sol_log_compute_units();
            let cu_before = sol_remaining_compute_units();
            let result = sqrt(radicand).unwrap();
            let cu_after = sol_remaining_compute_units();
            sol_log_compute_units();
            msg!("cu_bench_consumed {}", cu_before - cu_after);
            msg!("{}", result);
            Ok(())
        }
        MathInstruction::SquareRootU128 { radicand } => {
            msg!("Calculating u128 square root");
            sol_log_compute_units();
            let cu_before = sol_remaining_compute_units();
            let result = sqrt(radicand).unwrap();
            let cu_after = sol_remaining_compute_units();
            sol_log_compute_units();
            msg!("cu_bench_consumed {}", cu_before - cu_after);
            msg!("{}", result);
            Ok(())
        }
        MathInstruction::U64Multiply {
            multiplicand,
            multiplier,
        } => {
            msg!("Calculating U64 Multiply");
            sol_log_compute_units();
            let cu_before = sol_remaining_compute_units();
            let result = u64_multiply(multiplicand, multiplier);
            let cu_after = sol_remaining_compute_units();
            sol_log_compute_units();
            msg!("cu_bench_consumed {}", cu_before - cu_after);
            msg!("{}", result);
            Ok(())
        }
        MathInstruction::U64Divide { dividend, divisor } => {
            msg!("Calculating U64 Divide");
            sol_log_compute_units();
            let cu_before = sol_remaining_compute_units();
            let result = u64_divide(dividend, divisor);
            let cu_after = sol_remaining_compute_units();
            sol_log_compute_units();
            msg!("cu_bench_consumed {}", cu_before - cu_after);
            msg!("{}", result);
            Ok(())
        }
        MathInstruction::F32Multiply {
            multiplicand,
            multiplier,
        } => {
            msg!("Calculating f32 Multiply");
            sol_log_compute_units();
            let cu_before = sol_remaining_compute_units();
            let result = f32_multiply(multiplicand, multiplier);
            let cu_after = sol_remaining_compute_units();
            sol_log_compute_units();
            msg!("cu_bench_consumed {}", cu_before - cu_after);
            msg!("{}", result as u64);
            Ok(())
        }
        MathInstruction::F32Divide { dividend, divisor } => {
            msg!("Calculating f32 Divide");
            sol_log_compute_units();
            let cu_before = sol_remaining_compute_units();
            let result = f32_divide(dividend, divisor);
            let cu_after = sol_remaining_compute_units();
            sol_log_compute_units();
            msg!("cu_bench_consumed {}", cu_before - cu_after);
            msg!("{}", result as u64);
            Ok(())
        }
        MathInstruction::F32Exponentiate { base, exponent } => {
            msg!("Calculating f32 Exponent");
            sol_log_compute_units();
            let cu_before = sol_remaining_compute_units();
            let result = f32_exponentiate(base, exponent);
            let cu_after = sol_remaining_compute_units();
            sol_log_compute_units();
            msg!("cu_bench_consumed {}", cu_before - cu_after);
            msg!("{}", result as u64);
            Ok(())
        }
        MathInstruction::F32NaturalLog { argument } => {
            msg!("Calculating f32 Natural Log");
            sol_log_compute_units();
            let cu_before = sol_remaining_compute_units();
            let result = f32_natural_log(argument);
            let cu_after = sol_remaining_compute_units();
            sol_log_compute_units();
            msg!("cu_bench_consumed {}", cu_before - cu_after);
            msg!("{}", result as u64);
            Ok(())
        }
        MathInstruction::F32NormalCDF { argument } => {
            msg!("Calculating f32 Normal CDF");
            sol_log_compute_units();
            let cu_before = sol_remaining_compute_units();
            let result = f32_normal_cdf(argument);
            let cu_after = sol_remaining_compute_units();
            sol_log_compute_units();
            msg!("cu_bench_consumed {}", cu_before - cu_after);
            msg!("{}", result as u64);
            Ok(())
        }
        MathInstruction::F64Pow { base, exponent } => {
            msg!("Calculating f64 Pow");
            sol_log_compute_units();
            let cu_before = sol_remaining_compute_units();
            let result1 = base.powi(exponent as i32);
            let result2 = base.powf(exponent);
            let cu_after = sol_remaining_compute_units();
            sol_log_compute_units();
            msg!("cu_bench_consumed {}", cu_before - cu_after);
            msg!("{}", result1 as u64);
            msg!("{}", result2 as u64);
            Ok(())
        }
        MathInstruction::U128Multiply {
            multiplicand,
            multiplier,
        } => {
            msg!("Calculating u128 Multiply");
            sol_log_compute_units();
            let cu_before = sol_remaining_compute_units();
            let result = u128_multiply(multiplicand, multiplier);
            let cu_after = sol_remaining_compute_units();
            sol_log_compute_units();
            msg!("cu_bench_consumed {}", cu_before - cu_after);
            msg!("{}", result);
            Ok(())
        }
        MathInstruction::U128Divide { dividend, divisor } => {
            msg!("Calculating u128 Divide");
            sol_log_compute_units();
            let cu_before = sol_remaining_compute_units();
            let result = u128_divide(dividend, divisor);
            let cu_after = sol_remaining_compute_units();
            sol_log_compute_units();
            msg!("cu_bench_consumed {}", cu_before - cu_after);
            msg!("{}", result);
            Ok(())
        }
        MathInstruction::F64Multiply {
            multiplicand,
            multiplier,
        } => {
            msg!("Calculating f64 Multiply");
            sol_log_compute_units();
            let cu_before = sol_remaining_compute_units();
            let result = f64_multiply(multiplicand, multiplier);
            let cu_after = sol_remaining_compute_units();
            sol_log_compute_units();
            msg!("cu_bench_consumed {}", cu_before - cu_after);
            msg!("{}", result as u64);
            Ok(())
        }
        MathInstruction::F64Divide { dividend, divisor } => {
            msg!("Calculating f64 Divide");
            sol_log_compute_units();
            let cu_before = sol_remaining_compute_units();
            let result = f64_divide(dividend, divisor);
            let cu_after = sol_remaining_compute_units();
            sol_log_compute_units();
            msg!("cu_bench_consumed {}", cu_before - cu_after);
            msg!("{}", result as u64);
            Ok(())
        }
        MathInstruction::PreciseLog10 { values } => {
            msg!("Calculating log10 using PreciseNumber");
            let values: Vec<PreciseNumber256D18> = values.iter().map(|x| PreciseNumber256D18::new_from_f64(*x).unwrap()).collect();

            sol_log_compute_units();
            let cu_before = sol_remaining_compute_units();
            let mut sum_result: i128 = 0;
            for value in values {
                let result = value.log10().unwrap();
                sum_result += result.value.0[0] as i128;
            }
            let cu_after = sol_remaining_compute_units();
            sol_log_compute_units();
            msg!("cu_bench_consumed {}", cu_before - cu_after);
            msg!("{}", sum_result);
            Ok(())
        }
        MathInstruction::Noop => {
            msg!("Perform NOOP");
            let cu_before = sol_remaining_compute_units();
            // no-op
            let cu_after = sol_remaining_compute_units();
            msg!("cu_bench_consumed {}", cu_before - cu_after);
            msg!("{}", "noop");
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u64_multiply() {
        assert_eq!(2 * 2, u64_multiply(2, 2));
        assert_eq!(4 * 3, u64_multiply(4, 3));
    }

    #[test]
    fn test_u64_divide() {
        assert_eq!(1, u64_divide(2, 2));
        assert_eq!(2, u64_divide(2, 1));
    }

    #[test]
    fn test_f32_multiply() {
        assert_eq!(2.0 * 2.0, f32_multiply(2.0, 2.0));
        assert_eq!(4.0 * 3.0, f32_multiply(4.0, 3.0));
    }

    #[test]
    fn test_f32_divide() {
        assert_eq!(1.0, f32_divide(2.0, 2.0));
        assert_eq!(2.0, f32_divide(2.0, 1.0));
    }

    #[test]
    fn test_f32_exponentiate() {
        assert_eq!(16.0, f32_exponentiate(4.0, 2.0));
        assert_eq!(4.0, f32_exponentiate(16.0, 0.5))
    }

    #[test]
    fn test_f32_natural_log() {
        let one = 1.0f32;
        // e^1
        let e = one.exp();

        // ln(e) - 1 == 0
        let abs_difference = (f32_natural_log(e) - 1.0).abs();

        assert!(abs_difference <= f32::EPSILON);
    }

}
