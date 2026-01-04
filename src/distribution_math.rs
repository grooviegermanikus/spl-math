/// Calculate the normal cdf of the given number
///
/// The approximation is accurate to 3 digits
///
/// Code lovingly adapted from the excellent work at:
///
/// <https://www.hrpub.org/download/20140305/MS7-13401470.pdf>
///
/// The algorithm is based on the implementation in the paper above.
#[inline(never)]
pub fn f32_normal_cdf(argument: f32) -> f32 {
    const PI: f32 = std::f32::consts::PI;

    let mod_argument = if argument < 0.0 {
        -1.0 * argument
    } else {
        argument
    };
    let tabulation_numerator: f32 =
        (1.0 / (1.0 * (2.0 * PI).sqrt())) * (-1.0 * (mod_argument * mod_argument) / 2.0).exp();
    let tabulation_denominator: f32 =
        0.226 + 0.64 * mod_argument + 0.33 * (mod_argument * mod_argument + 3.0).sqrt();
    let y: f32 = 1.0 - tabulation_numerator / tabulation_denominator;
    if argument < 0.0 {
        1.0 - y
    } else {
        y
    }
}


#[cfg(test)]
mod tests {
    use {super::*, proptest::prelude::*};

    fn check_normal_cdf_f32(argument: f32) {
        let result = f32_normal_cdf(argument);
        let check_result = 0.5 * (1.0 + libm::erff(argument / std::f32::consts::SQRT_2));
        let abs_difference: f32 = (result - check_result).abs();
        assert!(abs_difference <= 0.000_2);
    }

    #[test]
    fn test_normal_cdf_f32_min_max() {
        let test_arguments: [f32; 2] = [f32::MIN, f32::MAX];
        for i in test_arguments.iter() {
            check_normal_cdf_f32(*i)
        }
    }

    proptest! {
        #[test]
        fn test_normal_cdf(a in -1000..1000) {

            check_normal_cdf_f32((a as f32)*0.005);
        }
    }
}

