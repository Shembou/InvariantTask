use invariant_task_lib::lp_pool::*;
fn main() {
    // Example usage
    let price = Price(1500); // 1.5 in fixed-point
    let min_fee = Percentage(100); // 1%
    let max_fee = Percentage(500); // 5%
    let liquidity_target = TokenAmount(10000); // 10,000 Tokens

    let pool = LpPool::init(price, min_fee, max_fee, liquidity_target);

    //     // Liquidity Provider adds tokens
    //     let lp_tokens = pool.add_liquidity(TokenAmount(5000));
    //     println!("LP Tokens Minted: {}", lp_tokens.0);

    //     // Swapper exchanges staked tokens for tokens
    //     let tokens_received = pool.swap(StakedTokenAmount(1000));
    //     println!("Tokens Received from Swap: {}", tokens_received.0);

    //     // Liquidity Provider removes liquidity
    //     let (tokens_redeemed, st_tokens_redeemed) = pool.remove_liquidity(lp_tokens);
    //     println!("Tokens Redeemed: {}", tokens_redeemed.0);
    //     println!("Staked Tokens Redeemed: {}", st_tokens_redeemed.0);
}
