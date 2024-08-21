use super::enums::Errors;
use super::structs::*;

impl LpPool {
    const FIXED_POINTS_DECIMALS_MULTIPLIER: u64 = 1000000;

    fn proportional(&self, amount: u64, numerator: u64, denominator: u64) -> Result<f64, Errors> {
        if denominator == 0 {
            return Ok(amount as f64);
        }
        let receive_amount = (amount as f64 * numerator as f64) / denominator as f64;
        if receive_amount == 0.0 {
            return Err(Errors::MultiplicationError);
        }

        Ok(receive_amount as f64)
    }
    fn calculate_added_liquidity_fee(&self, token_amount: u64) -> Result<f64, Errors> {
        if token_amount >= self.liquidity_target.0 {
            Ok(self.min_fee.0 as f64)
        } else {
            let calculation_result = self.proportional(
                self.max_fee.0.saturating_sub(self.min_fee.0),
                token_amount,
                self.liquidity_target.0,
            )?;

            let fee = self.max_fee.0 as f64 - calculation_result;

            Ok(fee)
        }
    }

    fn validate_input(&self) -> Result<(), Errors> {
        if self.min_fee.0 > self.max_fee.0 {
            return Err(Errors::InvalidFeeRange);
        }

        if self.liquidity_target.0 == 0 {
            return Err(Errors::InvalidLiquidityTarget);
        }

        Ok(())
    }

    fn calculate_total_lp_tokens(self: &mut Self) -> Result<u64, Errors> {
        let equivalent_token_amount =
            (self.st_token_amount.0 * self.price.0) / Self::FIXED_POINTS_DECIMALS_MULTIPLIER;

        let fee_percentage: Result<f64, Errors> = if self.st_token_amount.0 == 0 {
            Ok(self.min_fee.0 as f64)
        } else {
            Ok(self
                .calculate_added_liquidity_fee(self.lp_token_amount.0 - equivalent_token_amount)?)
        };
        let fee_multiplier =
            (Self::FIXED_POINTS_DECIMALS_MULTIPLIER as f64 / 100.0) - fee_percentage?;
        let final_token_amount = (equivalent_token_amount as f64 * fee_multiplier as f64)
            / (Self::FIXED_POINTS_DECIMALS_MULTIPLIER as f64 / 100.0);

        let total_lp_tokens = (equivalent_token_amount as f64 - final_token_amount as f64)
            + self.lp_token_amount.0 as f64;
        return Ok(total_lp_tokens as u64);
    }

    fn round_up_to_nearest_ten(value: f64) -> u64 {
        ((value / 10.0).ceil() * 10.0) as u64
    }

    pub fn init(
        price: Price,
        min_fee: Percentage,
        max_fee: Percentage,
        liquidity_target: TokenAmount,
    ) -> Result<Self, Errors> {
        let pool = LpPool {
            price,
            token_amount: TokenAmount(0),
            st_token_amount: StakedTokenAmount(0),
            lp_token_amount: LpTokenAmount(0),
            liquidity_target,
            min_fee,
            max_fee,
        };

        pool.validate_input()?;

        Ok(pool)
    }

    pub fn add_liquidity(
        self: &mut Self,
        token_amount: TokenAmount,
    ) -> Result<LpTokenAmount, Errors> {
        let lp_tokens_minted = if self.lp_token_amount.0 == 0 {
            LpTokenAmount(token_amount.0)
        } else {
            let calculated_lp_tokens = self.calculate_total_lp_tokens()?;
            let multiplier: f64 = self.lp_token_amount.0 as f64 / calculated_lp_tokens as f64;
            let proportional_lp_tokens = token_amount.0 as f64 * multiplier;
            let final_token_amount = Self::round_up_to_nearest_ten(proportional_lp_tokens);
            LpTokenAmount(final_token_amount as u64)
        };

        self.token_amount.0 += lp_tokens_minted.0;
        self.lp_token_amount.0 += lp_tokens_minted.0;

        Ok(lp_tokens_minted)
    }

    pub fn remove_liquidity(
        &mut self,
        lp_token_amount: LpTokenAmount,
    ) -> Result<(TokenAmount, StakedTokenAmount), Errors> {
        //TODO: Add remove_liquidity logic
        if self.st_token_amount.0 != 0 {
            let equivalent_token_amount =
                (self.st_token_amount.0 * self.price.0) / Self::FIXED_POINTS_DECIMALS_MULTIPLIER;

            let fee_percentage =
                self.calculate_added_liquidity_fee(self.token_amount.0 - equivalent_token_amount)?;
            let fee_multiplier =
                (Self::FIXED_POINTS_DECIMALS_MULTIPLIER as f64 / 100.0) - fee_percentage;
            let f64_token_value = (equivalent_token_amount as f64 * fee_multiplier as f64)
                / (Self::FIXED_POINTS_DECIMALS_MULTIPLIER as f64 / 100.0);
            let mut final_token_amount = Self::round_up_to_nearest_ten(f64_token_value);
            if final_token_amount >= lp_token_amount.0 {
                final_token_amount -= lp_token_amount.0;

                let staked_tokens_left = final_token_amount / self.price.0;
                self.st_token_amount.0 -= staked_tokens_left;
                return Ok((TokenAmount(0), StakedTokenAmount(staked_tokens_left)));
            } else {
                let mut total_tokens = lp_token_amount.0;
                total_tokens -= final_token_amount;
                let calculated_fee = self.calculate_added_liquidity_fee(total_tokens)?;
                let fee_multiplier =
                    (Self::FIXED_POINTS_DECIMALS_MULTIPLIER as f64 / 100.0) - calculated_fee;
                let f64_token_value = (total_tokens as f64 * fee_multiplier as f64)
                    / (Self::FIXED_POINTS_DECIMALS_MULTIPLIER as f64 / 100.0);
                total_tokens = Self::round_up_to_nearest_ten(f64_token_value);

                return Ok((
                    TokenAmount(total_tokens),
                    StakedTokenAmount(self.st_token_amount.0),
                ));
            }
        } else {
            self.token_amount.0 -= lp_token_amount.0;
            return Ok((TokenAmount(self.token_amount.0), StakedTokenAmount(0)));
        }
    }

    pub fn swap(&mut self, staked_token_amount: StakedTokenAmount) -> Result<TokenAmount, Errors> {
        let equivalent_token_amount =
            (staked_token_amount.0 * self.price.0) / Self::FIXED_POINTS_DECIMALS_MULTIPLIER;
        let fee_percentage: Result<f64, Errors> = if self.st_token_amount.0 == 0 {
            Ok(self.min_fee.0 as f64)
        } else {
            Ok(self.calculate_added_liquidity_fee(self.token_amount.0 - equivalent_token_amount)?)
        };
        let fee_multiplier =
            (Self::FIXED_POINTS_DECIMALS_MULTIPLIER as f64 / 100.0) - fee_percentage?;
        let f64_token_value = (equivalent_token_amount as f64 * fee_multiplier as f64)
            / (Self::FIXED_POINTS_DECIMALS_MULTIPLIER as f64 / 100.0);

        let final_token_amount = Self::round_up_to_nearest_ten(f64_token_value);

        if self.token_amount.0 <= final_token_amount {
            return Err(Errors::InsufficientLiquidity);
        }

        self.st_token_amount.0 += staked_token_amount.0;
        self.token_amount.0 -= final_token_amount;
        self.lp_token_amount.0 -= equivalent_token_amount - final_token_amount;

        Ok(TokenAmount(final_token_amount))
    }
}
