use crate::consts::SCALE;
use crate::errors::{AmountParseError, AppErrors, AppResult};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

/// Represents a monetary amount as a 64-bit integer.
/// The value is stored in the smallest unit (e.g., cents) to avoid floating-point precision issues.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Amount(pub i64);

impl Amount {
    /// Creates a new `Amount` with a value of zero.
    #[inline]
    pub fn zero() -> Self {
        Amount(0)
    }

    /// Safely adds two `Amount` values, returning `None` if an overflow occurs.
    ///
    /// # Arguments
    ///
    /// * `other` - The `Amount` to add to the current value.
    ///
    /// # Returns
    ///
    /// * `Some(Amount)` if the addition is successful.
    /// * `None` if an overflow occurs.
    #[inline]
    pub fn checked_add(self, other: Amount) -> Option<Amount> {
        self.0.checked_add(other.0).map(Amount)
    }

    /// Safely subtracts one `Amount` from another, returning `None` if an overflow occurs.
    ///
    /// # Arguments
    ///
    /// * `other` - The `Amount` to subtract from the current value.
    ///
    /// # Returns
    ///
    /// * `Some(Amount)` if the subtraction is successful.
    /// * `None` if an overflow occurs.
    #[inline]
    pub fn checked_sub(self, other: Amount) -> Option<Amount> {
        self.0.checked_sub(other.0).map(Amount)
    }

    /// Checks if the `Amount` is negative.
    ///
    /// # Returns
    ///
    /// * `true` if the value is negative.
    /// * `false` otherwise.
    #[inline]
    pub fn is_negative(self) -> bool {
        self.0 < 0
    }

    /// Parses a string into an `Amount` with up to 4 decimal places.
    /// Rounds the 5th decimal place half-up.
    ///
    /// # Arguments
    ///
    /// * `s` - The string representation of the amount.
    ///
    /// # Returns
    ///
    /// * `Ok(Amount)` if parsing is successful.
    /// * `Err(AppErrors::AmountParseError)` if parsing fails.
    ///
    /// # Errors
    ///
    /// * `AmountParseError::Empty` if the input string is empty.
    /// * `AmountParseError::MalformedInt` if the integer part is invalid.
    /// * `AmountParseError::MalformedFrac` if the fractional part is invalid.
    /// * `AmountParseError::Overflow` if an overflow occurs during parsing.
    pub fn parse_4dp(s: &str) -> AppResult<Amount> {
        let s = s.trim();
        if s.is_empty() {
            return Err(AppErrors::AmountParseError(AmountParseError::Empty));
        }

        let neg = matches!(s.as_bytes()[0], b'-');
        let s = if neg || s.starts_with('+') {
            &s[1..]
        } else {
            s
        };

        let mut it = s.splitn(2, '.');
        let int_part: i64 = it
            .next()
            .ok_or(AmountParseError::MalformedInt)?
            .parse()
            .map_err(|_| AmountParseError::MalformedInt)?;

        let frac_src = it.next().unwrap_or("0");
        let (keep, rest) = if frac_src.len() > 4 {
            frac_src.split_at(4)
        } else {
            (frac_src, "")
        };

        let mut frac_str = keep.to_string();
        while frac_str.len() < 4 {
            frac_str.push('0');
        }
        let mut frac: i64 = frac_str
            .parse()
            .map_err(|_| AmountParseError::MalformedFrac)?;

        // round half-up based on first dropped digit, if any
        if !rest.is_empty() && rest.as_bytes()[0] >= b'5' {
            frac = frac.checked_add(1).ok_or(AmountParseError::Overflow)?;
        }

        let base = int_part
            .checked_mul(SCALE)
            .ok_or(AmountParseError::Overflow)?;
        let val = base.checked_add(frac).ok_or(AmountParseError::Overflow)?;
        Ok(if neg { Amount(-val) } else { Amount(val) })
    }
}

impl FromStr for Amount {
    type Err = AmountParseError;
    /// Parses a string into an `Amount` using the `parse_4dp` method.
    ///
    /// # Arguments
    ///
    /// * `s` - The string representation of the amount.
    ///
    /// # Returns
    ///
    /// * `Ok(Amount)` if parsing is successful.
    /// * `Err(AmountParseError)` if parsing fails.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Amount::parse_4dp(s).map_err(|e| match e {
            AppErrors::AmountParseError(err) => err,
            _ => AmountParseError::MalformedInt,
        })
    }
}

impl Display for Amount {
    /// Formats the `Amount` as a string with 4 decimal places.
    ///
    /// # Arguments
    ///
    /// * `f` - The formatter to write the output to.
    ///
    /// # Returns
    ///
    /// * `std::fmt::Result` indicating success or failure.
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let sign = if self.0 < 0 { "-" } else { "" };
        let abs = self.0.abs();
        let int = abs / SCALE;
        let frac = abs % SCALE;
        write!(f, "{sign}{int}.{:04}", frac)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn zero_amount_has_value_zero() {
        let amount = Amount::zero();
        assert_eq!(amount.0, 0);
    }

    #[test]
    fn checked_add_returns_correct_sum() {
        let a = Amount(100);
        let b = Amount(200);
        let result = a.checked_add(b);
        assert_eq!(result, Some(Amount(300)));
    }

    #[test]
    fn checked_add_returns_none_on_overflow() {
        let a = Amount(i64::MAX);
        let b = Amount(1);
        let result = a.checked_add(b);
        assert!(result.is_none());
    }

    #[test]
    fn checked_sub_returns_correct_difference() {
        let a = Amount(300);
        let b = Amount(200);
        let result = a.checked_sub(b);
        assert_eq!(result, Some(Amount(100)));
    }

    #[test]
    fn checked_sub_returns_none_on_underflow() {
        let a = Amount(i64::MIN);
        let b = Amount(1);
        let result = a.checked_sub(b);
        assert!(result.is_none());
    }

    #[test]
    fn is_negative_returns_true_for_negative_amount() {
        let amount = Amount(-1);
        assert!(amount.is_negative());
    }

    #[test]
    fn is_negative_returns_false_for_non_negative_amount() {
        let amount = Amount(0);
        assert!(!amount.is_negative());
    }

    #[test]
    fn parse_4dp_parses_valid_string() {
        let result = Amount::parse_4dp("123.4567").unwrap();
        assert_eq!(result, Amount(1234567));
    }

    #[test]
    fn parse_4dp_handles_negative_values() {
        let result = Amount::parse_4dp("-123.4567").unwrap();
        assert_eq!(result, Amount(-1234567));
    }

    #[test]
    fn parse_4dp_rounds_half_up() {
        let result = Amount::parse_4dp("123.45675").unwrap();
        assert_eq!(result, Amount(1234568));
    }

    #[test]
    fn parse_4dp_returns_error_on_empty_string() {
        let result = Amount::parse_4dp("");
        assert!(result.is_err());
    }

    #[test]
    fn parse_4dp_returns_error_on_malformed_int() {
        let result = Amount::parse_4dp("abc.1234");
        assert!(result.is_err());
    }

    #[test]
    fn parse_4dp_returns_error_on_malformed_frac() {
        let result = Amount::parse_4dp("123.abc");
        assert!(result.is_err());
    }

    #[test]
    fn parse_4dp_returns_error_on_overflow() {
        let result = Amount::parse_4dp(&format!("{}.1234", i64::MAX));
        assert!(result.is_err());
    }

    #[test]
    fn from_str_parses_valid_string() {
        let result = Amount::from_str("123.4567").unwrap();
        assert_eq!(result, Amount(1234567));
    }

    #[test]
    fn from_str_returns_error_on_invalid_string() {
        let result = Amount::from_str("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn display_formats_correctly() {
        let amount = Amount(1234567);
        assert_eq!(format!("{}", amount), "123.4567");
    }

    #[test]
    fn display_formats_negative_correctly() {
        let amount = Amount(-1234567);
        assert_eq!(format!("{}", amount), "-123.4567");
    }
}
