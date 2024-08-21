#[derive(Debug)]
pub enum Errors {
    InvalidFeeRange,
    InvalidLiquidityTarget,
    MultiplicationError,
    InsufficientLiquidity,
    DivisionByZero,
    SubtractionOverflow,
    CalculationError,
    AdditionOverflow,
    InvalidTokenAmount,
}
