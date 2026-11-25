//! Number formatting that matches Python's `format(x, "g")`, so reports read the same
//! as the Python tools they replace.

/// Format `value` like Python's `f"{value:g}"`: six significant digits, trailing zeros
/// removed, and scientific notation below 1e-4 or from 1e6 upwards.
pub fn general(value: f64) -> String {
    const PRECISION: i32 = 6;
    if value == 0.0 {
        return if value.is_sign_negative() { "-0" } else { "0" }.into();
    }
    if value.is_nan() {
        return "nan".into();
    }
    if value.is_infinite() {
        return if value < 0.0 { "-inf" } else { "inf" }.into();
    }
    // Round to the target number of significant digits first, then read the exponent.
    let scientific = format!("{:.*e}", (PRECISION - 1) as usize, value);
    let (mantissa, exponent) = scientific
        .split_once('e')
        .expect("Rust scientific formatting always contains 'e'");
    let exponent: i32 = exponent
        .parse()
        .expect("Rust scientific exponent is an integer");
    if (-4..PRECISION).contains(&exponent) {
        let decimals = (PRECISION - 1 - exponent).max(0) as usize;
        trim_fraction(&format!("{value:.decimals$}")).to_owned()
    } else {
        let sign = if exponent < 0 { '-' } else { '+' };
        format!("{}e{sign}{:02}", trim_fraction(mantissa), exponent.abs())
    }
}

fn trim_fraction(number: &str) -> &str {
    if number.contains('.') {
        number.trim_end_matches('0').trim_end_matches('.')
    } else {
        number
    }
}

#[cfg(test)]
mod tests {
    use super::general;

    #[test]
    fn matches_python_general_format() {
        let cases = [
            (15.0, "15"),
            (42.5, "42.5"),
            (90.0, "90"),
            (6.0, "6"),
            (2.0, "2"),
            (0.5, "0.5"),
            (0.0001, "0.0001"),
            (0.00001, "1e-05"),
            (123_456.0, "123456"),
            (1_234_567.0, "1.23457e+06"),
            (1.0e16, "1e+16"),
            (-3.25, "-3.25"),
            (0.1 + 0.2, "0.3"),
            (999_999.5, "1e+06"),
            (0.0, "0"),
        ];
        for (value, expected) in cases {
            assert_eq!(general(value), expected, "general({value:?})");
        }
    }
}
