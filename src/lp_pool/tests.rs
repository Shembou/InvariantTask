#[cfg(test)]
mod tests {
    use crate::lp_pool::enums::Errors;
    use crate::lp_pool::*;

    fn setup_pool(
        price: u64,
        min_fee: u64,
        max_fee: u64,
        liquidity_target: u64,
    ) -> Result<LpPool, Errors> {
        LpPool::init(
            Price(price),
            Percentage(min_fee),
            Percentage(max_fee),
            TokenAmount(liquidity_target),
        )
    }

    fn add_liquidity(pool: &mut LpPool, token_amount: u64) -> Result<LpTokenAmount, Errors> {
        pool.add_liquidity(TokenAmount(token_amount))
    }

    fn swap_tokens(pool: &mut LpPool, staked_token_amount: u64) -> Result<TokenAmount, Errors> {
        pool.swap(StakedTokenAmount(staked_token_amount))
    }

    fn remove_liquidity(
        pool: &mut LpPool,
        lp_token_amount: u64,
    ) -> Result<(TokenAmount, StakedTokenAmount), Errors> {
        pool.remove_liquidity(LpTokenAmount(lp_token_amount))
    }

    #[test]
    fn test_init_success() {
        let pool = setup_pool(1500, 10, 900, 90000);
        assert!(pool.is_ok(), "Expected pool initialization to succeed.");
    }

    #[test]
    fn test_init_error() {
        let pool = setup_pool(1500, 5000, 1000, 0);
        assert!(
            pool.is_err(),
            "Expected pool initialization to fail due to incorrect fee settings."
        );
    }

    #[test]
    fn test_add_liquidity_once() {
        let mut pool = setup_pool(1500000, 10, 900, 90000000).expect("Failed to initialize pool");
        let lp_tokens = add_liquidity(&mut pool, 5000000).expect("Failed to add liquidity");

        assert_eq!(lp_tokens.0, 5000000, "Incorrect LP token amount");
        assert_eq!(pool.token_amount.0, 5000000, "Incorrect token reserve");
        assert_eq!(pool.lp_token_amount.0, 5000000, "Incorrect total LP tokens");
    }
    //TODO: The last part of this test throws error
    #[test]
    fn test_story() {
        let mut pool = setup_pool(1500000, 10, 900, 90000000).expect("Failed to initialize pool");

        add_liquidity(&mut pool, 100000000).expect("Failed to add initial liquidity");

        let swapped_tokens = swap_tokens(&mut pool, 6000000).expect("Failed to swap staked tokens");
        assert_eq!(
            swapped_tokens.0, 8991000,
            "Incorrect token amount after swap"
        );

        let new_lp_tokens = add_liquidity(&mut pool, 10000000).expect("Failed to add liquidity");
        assert_eq!(
            new_lp_tokens.0, 9999100,
            "Incorrect LP token amount after adding liquidity"
        );

        let swapped_tokens2 =
            swap_tokens(&mut pool, 30000000).expect("Failed to swap staked tokens");
        assert_eq!(
            swapped_tokens2.0, 43442370,
            "Incorrect LP token amount after adding liquidity"
        );

        let remove_liquidity =
            remove_liquidity(&mut pool, 109991000).expect("Error while removing tokens");
        assert_eq!(remove_liquidity.0 .0, 57566630, "Error");
        assert_eq!(remove_liquidity.1 .0, 36000000, "Error");
    }
}
