/// Represents the various errors that can occur in the application.
/// Each variant corresponds to a specific type of error.
#[derive(thiserror::Error, Debug)]
pub enum AppErrors {
    /// An internal error with a specific message.
    #[error("Internal error: {0}")]
    Internal(String),

    /// An error indicating an arithmetic overflow.
    #[error("Overflow error")]
    Overflow,

    /// An error indicating invalid input with a specific message.
    #[error("invalid input: {0}")]
    InvalidInput(&'static str),

    /// An error related to input/output operations with a specific message.
    #[error("io: {0}")]
    Io(String),

    /// An error that wraps an `AmountParseError` and propagates it.
    #[error(transparent)]
    AmountParseError(#[from] AmountParseError),
}

/// Represents errors that can occur while parsing an amount.
/// Each variant corresponds to a specific parsing issue.
#[derive(Debug, thiserror::Error)]
pub enum AmountParseError {
    /// An error indicating that the amount is empty.
    #[error("empty amount")]
    Empty,

    /// An error indicating a malformed integer part in the amount.
    #[error("malformed integer part")]
    MalformedInt,

    /// An error indicating a malformed fractional part in the amount.
    #[error("malformed fractional part")]
    MalformedFrac,

    /// An error indicating an arithmetic overflow during parsing.
    #[error("overflow")]
    Overflow,
}

/// A type alias for results returned by the application.
/// Encapsulates a value of type `T` or an `AppErrors` error.
pub type AppResult<T> = Result<T, AppErrors>;
