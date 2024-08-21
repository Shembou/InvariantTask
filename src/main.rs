use invariant_task_lib::lp_pool::*;
fn main() {
    // Example usage
    let price = Price(1500000);
    let min_fee = Percentage(10);
    let max_fee = Percentage(900);
    let liquidity_target = TokenAmount(90000000);

    let mut pool =
        LpPool::init(price, min_fee, max_fee, liquidity_target).expect("Failed to initialize pool");

    let lp_tokens = pool
        .add_liquidity(TokenAmount(100000000))
        .expect("Error while adding liquidity");
    println!("Addedd: {} tokens to LpPool", lp_tokens.0);

    let tokens_received = pool
        .swap(StakedTokenAmount(6000000))
        .expect("Error while swapping tokens");
    println!("Tokens Received from Swap: {}", tokens_received.0);

    let new_lp_tokens = pool
        .add_liquidity(TokenAmount(10000000))
        .expect("Error while adding more liquidity");
    println!("New LP Tokens Added: {}", new_lp_tokens.0);

    let new_tokens_received = pool
        .swap(StakedTokenAmount(30000000))
        .expect("Error while swapping more tokens");
    println!(
        "Tokens Received from Second Swap: {}",
        new_tokens_received.0
    );

    let (tokens_redeemed, st_tokens_redeemed) = pool
        .remove_liquidity(LpTokenAmount(new_lp_tokens.0))
        .expect("Error while removing liquidity");
    println!("Tokens Redeemed: {}", tokens_redeemed.0);
    println!("Staked Tokens Redeemed: {}", st_tokens_redeemed.0);
}
