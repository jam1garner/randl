#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    #[error("the file could not be read")]
    FileReadFail,
    #[error("could not parse: invalid KDL")]
    ParseFail(KdlError),
    #[error("The value did not match the specified type")]
    IncorrectType,
    #[error("Type name invalid")]
    InvalidType,
    #[error("Value must follow type declaration in set")]
    NoValue,
    #[error("No percent provided for the given chance expression")]
    NoPercent,
    #[error("Chance expressions cannot be mixed with other expressions")]
    MixedExprs,
    #[error("Too many expressions were provided. Only multiple `chance` expessions may be used")]
    TooManyExprs,
    #[error("An expressions is required for every parameter entry specified")]
    ExprRequired,
    #[error("Invalid return: {0}")]
    InvalidReturn(&'static str),
    #[error("Invalid chance: {0}")]
    InvalidChance(&'static str),
    #[error("Invalid expression: {0}")]
    InvalidExpr(String),
    #[error("Invalid top-level entry: {0}")]
    InvalidRandlEntry(&'static str),
}