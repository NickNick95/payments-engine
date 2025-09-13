use crate::consts::SCALE;
use crate::errors::{AmountParseError, AppErrors, AppResult};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Amount(pub i64);

impl Amount {
    #[inline]
    pub fn zero() -> Self {
        Amount(0)
    }
    #[inline]
    pub fn checked_add(self, other: Amount) -> Option<Amount> {
        self.0.checked_add(other.0).map(Amount)
    }
    #[inline]
    pub fn checked_sub(self, other: Amount) -> Option<Amount> {
        self.0.checked_sub(other.0).map(Amount)
    }
    #[inline]
    pub fn is_negative(self) -> bool {
        self.0 < 0
    }

    /// Parse decimal with up to 4 dp; rounds the 5th dp half-up.
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

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Amount::parse_4dp(s).map_err(|e| match e {
            AppErrors::AmountParseError(err) => err,
            _ => AmountParseError::MalformedInt,
        })
    }
}

impl Display for Amount {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let sign = if self.0 < 0 { "-" } else { "" };
        let abs = self.0.abs();
        let int = abs / SCALE;
        let frac = abs % SCALE;
        write!(f, "{sign}{int}.{:04}", frac)
    }
}
