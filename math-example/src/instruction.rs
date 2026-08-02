//! Program instructions, used for end-to-end testing and instruction counts

use {
    crate::id,
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::instruction::Instruction,
};

/// Algorithms supported for square root calculation
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub enum SqrtAlgorithm {
    /// Babylonian method
    Newton,
    /// CORDIC method
    Cordic,
}

/// Instructions supported by the math program, used for testing instruction
/// counts
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub enum MathInstruction {
    /// Calculate the square root of the given u64 with decimals
    ///
    /// No accounts required for this instruction
    PreciseSquareRoot {
        /// Number underneath the square root sign, whose square root will be
        /// calculated
        radicand: Vec<f64>,
        /// Algorithm to use for square root calculation
        algorithm: SqrtAlgorithm,
    },
    /// Muldiv three u64 values
    ///
    /// No accounts required for this instruction
    PreciseMulDiv {
        /// The value to be multiplied and divided
        val: u64,
        /// The numerator
        num: u64,
        /// The denominator
        denom: u64,
    },
    /// Calculate the integer square root of the given u64
    ///
    /// No accounts required for this instruction
    SquareRootU64 {
        /// Number underneath the square root sign, whose square root will be
        /// calculated
        radicand: u64,
    },
    /// Calculate the integer square root of the given u128
    ///
    /// No accounts required for this instruction
    SquareRootU128 {
        /// Number underneath the square root sign, whose square root will be
        /// calculated
        radicand: u128,
    },
    /// Multiply two u64 values
    ///
    /// No accounts required for this instruction
    U64Multiply {
        /// The multiplicand
        multiplicand: u64,
        /// The multipier
        multiplier: u64,
    },
    /// Divide two u64 values
    ///
    /// No accounts required for this instruction
    U64Divide {
        /// The dividend
        dividend: u64,
        /// The divisor
        divisor: u64,
    },
    /// Multiply two float values
    ///
    /// No accounts required for this instruction
    F32Multiply {
        /// The multiplicand
        multiplicand: f32,
        /// The multipier
        multiplier: f32,
    },
    /// Divide two float values
    ///
    /// No accounts required for this instruction
    F32Divide {
        /// The dividend
        dividend: f32,
        /// The divisor
        divisor: f32,
    },

    /// Exponentiate a float base by a power
    ///
    /// No accounts required for this instruction
    F32Exponentiate {
        /// The base
        base: f32,
        /// The exponent
        exponent: f32,
    },

    /// Natural Log of a float
    ///
    /// No accounts required for this instruction
    F32NaturalLog {
        /// The argument
        argument: f32,
    },

    /// The Normal CDF of a float
    ///
    /// No accounts required for this instruction
    F32NormalCDF {
        /// The argument
        argument: f32,
    },

    /// Pow two float values
    ///
    /// No accounts required for this instruction
    F64Pow {
        /// The base
        base: f64,
        /// The exponent
        exponent: f64,
    },

    /// Multiply two u128 values
    ///
    /// No accounts required for this instruction
    U128Multiply {
        /// The multiplicand
        multiplicand: u128,
        /// The multipier
        multiplier: u128,
    },
    /// Divide two u128 values
    ///
    /// No accounts required for this instruction
    U128Divide {
        /// The dividend
        dividend: u128,
        /// The divisor
        divisor: u128,
    },
    /// Multiply two f64 values
    ///
    /// No accounts required for this instruction
    F64Multiply {
        /// The multiplicand
        multiplicand: f64,
        /// The multipier
        multiplier: f64,
    },
    /// Divide two f64 values
    ///
    /// No accounts required for this instruction
    F64Divide {
        /// The dividend
        dividend: f64,
        /// The divisor
        divisor: f64,
    },

    /// Calculate log10 of the given value(s) using PreciseNumber
    ///
    /// No accounts required for this instruction
    PreciseLog10 {
        /// Values to compute log10 of
        values: Vec<f64>,
    },

    /// Don't do anything for comparison
    ///
    /// No accounts required for this instruction
    Noop,
}

/// Create SquareRoot instruction
pub fn precise_sqrt(radicand: u64, sqrt_algorithm: SqrtAlgorithm) -> Instruction {
    Instruction {
        program_id: id(),
        accounts: vec![],
        data: borsh::to_vec(&MathInstruction::PreciseSquareRoot { radicand: vec![radicand as f64], algorithm: sqrt_algorithm }).unwrap(),
    }
}

/// Create SquareRoot instruction for array of f64
pub fn precise_sqrt_array(start: f64, step: f64, sqrt_algorithm: SqrtAlgorithm) -> Instruction {

    let mut radicand: Vec<f64> = Vec::new();
    for i in 0..8 {
        radicand.push(start + step * (i as f64));
    }

    Instruction {
        program_id: id(),
        accounts: vec![],
        data: borsh::to_vec(&MathInstruction::PreciseSquareRoot { radicand, algorithm: sqrt_algorithm }).unwrap(),
    }
}

/// Create PreciseMulDiv instruction
pub fn precise_muldiv(val: u64, num: u64, denom: u64) -> Instruction {
    Instruction {
        program_id: id(),
        accounts: vec![],
        data: borsh::to_vec(&MathInstruction::PreciseMulDiv { val, num, denom }).unwrap(),
    }
}

/// Create U64 SquareRoot instruction
pub fn sqrt_u64(radicand: u64) -> Instruction {
    Instruction {
        program_id: id(),
        accounts: vec![],
        data: borsh::to_vec(&MathInstruction::SquareRootU64 { radicand }).unwrap(),
    }
}

/// Create U128 SquareRoot instruction
pub fn sqrt_u128(radicand: u128) -> Instruction {
    Instruction {
        program_id: id(),
        accounts: vec![],
        data: borsh::to_vec(&MathInstruction::SquareRootU128 { radicand }).unwrap(),
    }
}

/// Create U64 Multiplication instruction
pub fn u64_multiply(multiplicand: u64, multiplier: u64) -> Instruction {
    Instruction {
        program_id: id(),
        accounts: vec![],
        data: borsh::to_vec(&MathInstruction::U64Multiply {
            multiplicand,
            multiplier,
        })
        .unwrap(),
    }
}

/// Create U64 Division instruction
pub fn u64_divide(dividend: u64, divisor: u64) -> Instruction {
    Instruction {
        program_id: id(),
        accounts: vec![],
        data: borsh::to_vec(&MathInstruction::U64Divide { dividend, divisor }).unwrap(),
    }
}

/// Create F32 Multiplication instruction
pub fn f32_multiply(multiplicand: f32, multiplier: f32) -> Instruction {
    Instruction {
        program_id: id(),
        accounts: vec![],
        data: borsh::to_vec(&MathInstruction::F32Multiply {
            multiplicand,
            multiplier,
        })
        .unwrap(),
    }
}

/// Create F32 Division instruction
pub fn f32_divide(dividend: f32, divisor: f32) -> Instruction {
    Instruction {
        program_id: id(),
        accounts: vec![],
        data: borsh::to_vec(&MathInstruction::F32Divide { dividend, divisor }).unwrap(),
    }
}

/// Create F32 Exponentiate instruction
pub fn f32_exponentiate(base: f32, exponent: f32) -> Instruction {
    Instruction {
        program_id: id(),
        accounts: vec![],
        data: borsh::to_vec(&MathInstruction::F32Exponentiate { base, exponent }).unwrap(),
    }
}

/// Create F32 Natural Log instruction
pub fn f32_natural_log(argument: f32) -> Instruction {
    Instruction {
        program_id: id(),
        accounts: vec![],
        data: borsh::to_vec(&MathInstruction::F32NaturalLog { argument }).unwrap(),
    }
}

/// Create F32 Normal CDF instruction
pub fn f32_normal_cdf(argument: f32) -> Instruction {
    Instruction {
        program_id: id(),
        accounts: vec![],
        data: borsh::to_vec(&MathInstruction::F32NormalCDF { argument }).unwrap(),
    }
}

/// Create F64Pow instruction
pub fn f64_pow(base: f64, exponent: f64) -> Instruction {
    Instruction {
        program_id: id(),
        accounts: vec![],
        data: borsh::to_vec(&MathInstruction::F64Pow { base, exponent }).unwrap(),
    }
}

/// Create U128 Multiplication instruction
pub fn u128_multiply(multiplicand: u128, multiplier: u128) -> Instruction {
    Instruction {
        program_id: id(),
        accounts: vec![],
        data: borsh::to_vec(&MathInstruction::U128Multiply {
            multiplicand,
            multiplier,
        })
        .unwrap(),
    }
}

/// Create U128 Division instruction
pub fn u128_divide(dividend: u128, divisor: u128) -> Instruction {
    Instruction {
        program_id: id(),
        accounts: vec![],
        data: borsh::to_vec(&MathInstruction::U128Divide { dividend, divisor }).unwrap(),
    }
}

/// Create F64 Multiplication instruction
pub fn f64_multiply(multiplicand: f64, multiplier: f64) -> Instruction {
    Instruction {
        program_id: id(),
        accounts: vec![],
        data: borsh::to_vec(&MathInstruction::F64Multiply {
            multiplicand,
            multiplier,
        })
        .unwrap(),
    }
}

/// Create F64 Division instruction
pub fn f64_divide(dividend: f64, divisor: f64) -> Instruction {
    Instruction {
        program_id: id(),
        accounts: vec![],
        data: borsh::to_vec(&MathInstruction::F64Divide { dividend, divisor }).unwrap(),
    }
}

/// Create PreciseLog10 instruction for a single value
pub fn precise_log10(value: u64) -> Instruction {
    Instruction {
        program_id: id(),
        accounts: vec![],
        data: borsh::to_vec(&MathInstruction::PreciseLog10 { values: vec![value as f64] }).unwrap(),
    }
}

/// Create PreciseLog10 instruction for array of f64
pub fn precise_log10_array(start: f64, step: f64) -> Instruction {
    let mut values: Vec<f64> = Vec::new();
    for i in 0..8 {
        values.push(start + step * (i as f64));
    }

    Instruction {
        program_id: id(),
        accounts: vec![],
        data: borsh::to_vec(&MathInstruction::PreciseLog10 { values }).unwrap(),
    }
}

/// Create Noop instruction
pub fn noop() -> Instruction {
    Instruction {
        program_id: id(),
        accounts: vec![],
        data: borsh::to_vec(&MathInstruction::Noop).unwrap(),
    }
}
