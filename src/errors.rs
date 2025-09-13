#[derive(thiserror::Error, Debug)]
pub enum AppErrors {
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
