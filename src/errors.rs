#[derive(thiserror::Error, Debug)]
pub enum AppErrors {
    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Overflow error")]
    Overflow,

    #[error("invalid input: {0}")]
    InvalidInput(&'static str),

    #[error("io: {0}")]
    Io(String),

    #[error(transparent)]
    AmountParseError(#[from] AmountParseError),
}

#[derive(Debug, thiserror::Error)]
pub enum AmountParseError {
    #[error("empty amount")]
    Empty,
    #[error("malformed integer part")]
    MalformedInt,
    #[error("malformed fractional part")]
    MalformedFrac,
    #[error("overflow")]
    Overflow,
}

pub type AppResult<T> = Result<T, AppErrors>;
