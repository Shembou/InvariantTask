pub mod utils {

    use crate::lp_pool::enums::Errors;

    pub const FIXED_POINTS_DECIMALS_MULTIPLIER: u64 = 1000000;

    pub fn proportional(amount: u64, numerator: u64, denominator: u64) -> Result<f64, Errors> {
        if denominator == 0 {
            return Ok(amount as f64);
        }
        let receive_amount = (amount as f64 * numerator as f64) / denominator as f64;
        if receive_amount == 0.0 {
            return Err(Errors::MultiplicationError);
        }

        Ok(receive_amount as f64)
    }
    pub fn calculate_added_liquidity_fee(
        max_fee: u64,
        min_fee: u64,
        liquidity_target: u64,
        token_amount: u64,
    ) -> Result<f64, Errors> {
        if token_amount >= liquidity_target {
            Ok(min_fee as f64)
        } else {
            let calculation_result = proportional(
                max_fee.saturating_sub(min_fee),
                token_amount,
                liquidity_target,
            )?;

            let fee = max_fee as f64 - calculation_result;

            Ok(fee)
        }
    }

    pub fn validate_input(min_fee: u64, max_fee: u64, liquidity_target: u64) -> Result<(), Errors> {
        if min_fee > max_fee {
            return Err(Errors::InvalidFeeRange);
        }

        if liquidity_target == 0 {
            return Err(Errors::InvalidLiquidityTarget);
        }

        Ok(())
    }

    pub fn round_up_to_nearest_ten(value: f64) -> u64 {
        ((value / 10.0).ceil() * 10.0) as u64
    }

    pub fn multiply_swap_token_amount(fee_percentage: f64, token_amount: u64) -> u64 {
        let fee_multiplier = (FIXED_POINTS_DECIMALS_MULTIPLIER as f64 / 100.0) - fee_percentage;
        let f64_token_value = (token_amount as f64 * fee_multiplier as f64)
            / (FIXED_POINTS_DECIMALS_MULTIPLIER as f64 / 100.0);

        let final_token_amount = round_up_to_nearest_ten(f64_token_value);

        final_token_amount
    }

    pub fn multiply_add_liquidity_token_amount(lp_token_amount: u64, calculated_lp_tokens: u64) -> u64 {
        let multiplier: f64 = lp_token_amount as f64 / calculated_lp_tokens as f64;
        let proportional_lp_tokens = lp_token_amount as f64 * multiplier;
        let result = round_up_to_nearest_ten(proportional_lp_tokens);
        result
    }

    pub fn calculate_staked_tokens(amount: u64, price: u64) -> u64 {
        (amount * price) / FIXED_POINTS_DECIMALS_MULTIPLIER
    }
}
