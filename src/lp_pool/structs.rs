pub struct TokenAmount(pub u64);

pub struct StakedTokenAmount(pub u64);

pub struct LpTokenAmount(pub u64);

pub struct Price(pub u64);

#[derive(Debug, Clone, Copy)]
pub struct Percentage(pub u64);

pub struct LpPool {
    pub price: Price,
    pub token_amount: TokenAmount,
    pub st_token_amount: StakedTokenAmount,
    pub lp_token_amount: LpTokenAmount,
    pub liquidity_target: TokenAmount,
    pub min_fee: Percentage,
    pub max_fee: Percentage,
}
