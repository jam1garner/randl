use kdl::KdlError;
#[derive(thiserror::Error, Debug, Clone)]
pub enum ParseError {
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
    #[error("Invalid `value` statement: {0}")]
    InvalidValueStmt(&'static str),
    #[error("Invalid chance: {0}")]
    InvalidChance(&'static str),
    #[error("Invalid expression: {0}")]
    InvalidExpr(String),
    #[error("Invalid top-level entry: {0}")]
    InvalidRandlEntry(&'static str),
    #[error("Only `value` nodes allowed in sets, not `{0}`")]
    NonValueInSet(String),
}

#[derive(thiserror::Error, Debug, Clone)]
pub enum EvalError {
    #[error("Index {0} was outside of the bounds of the param list")]
    IndexOutOfBounds(usize),
    #[error("{0}")]
    MissingField(String),
    #[error("Invalid field: {0}")]
    InvalidField(&'static str),
    #[error("{0} cannot be assigned to {1}")]
    InvalidAssignment(&'static str, &'static str),
    #[error("The given value was too large to fit")]
    IntTooBig,
    #[error("The set {0:?} could not be found")]
    InvalidSet(String),
}
