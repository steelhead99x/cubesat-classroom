//! Number formatting that matches Python's `format(x, "g")`, so reports read the same
//! as the Python tools they replace.

/// Significant digits in Python's default `g` format.
const PRECISION: i32 = 6;

/// Format `value` like Python's `f"{value:g}"`: six significant digits, trailing zeros
/// removed, and scientific notation below 1e-4 or from 1e6 upwards.
#[must_use]
pub fn general(value: f64) -> String {
    if value == 0.0 {
        return if value.is_sign_negative() { "-0" } else { "0" }.into();
    }
    if value.is_nan() {
        return "nan".into();
    }
    if value.is_infinite() {
        return if value < 0.0 { "-inf" } else { "inf" }.into();
    }
    // Round to six significant digits first, then read the exponent: 999999.5 rounds
    // up to 1e+06 and so switches to scientific notation, as in Python.
    let scientific = format!("{value:.5e}");
    // Rust writes every finite number in `{:e}` as `<mantissa>e<exponent>`.
    let Some((mantissa, exponent)) = scientific
        .split_once('e')
        .and_then(|(m, e)| Some((m, e.parse::<i32>().ok()?)))
    else {
        return scientific;
    };
    if (-4..PRECISION).contains(&exponent) {
        // 0 to 9 decimals: enough for six significant digits at this exponent.
        let decimals = usize::try_from(PRECISION - 1 - exponent).unwrap_or(0);
        trim_fraction(&format!("{value:.decimals$}")).to_owned()
    } else {
        let sign = if exponent < 0 { '-' } else { '+' };
        format!(
            "{}e{sign}{:02}",
            trim_fraction(mantissa),
            exponent.unsigned_abs()
        )
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
        // Expected strings are Python 3.12's f"{value:g}".
        let cases = [
            (15.0, "15"),
            (42.5, "42.5"),
            (90.0, "90"),
            (6.0, "6"),
            (2.0, "2"),
            (0.5, "0.5"),
            (0.0001, "0.0001"),
            (0.000_123_456_7, "0.000123457"),
            (0.00001, "1e-05"),
            (123_456.0, "123456"),
            (1_234_567.0, "1.23457e+06"),
            (1.0e16, "1e+16"),
            (-3.25, "-3.25"),
            (0.1 + 0.2, "0.3"),
            (999_999.5, "1e+06"),
            (1e100, "1e+100"),
            (0.0, "0"),
            (-0.0, "-0"),
            (f64::NAN, "nan"),
            (f64::INFINITY, "inf"),
        ];
        for (value, expected) in cases {
            assert_eq!(general(value), expected, "general({value:?})");
        }
    }
}
