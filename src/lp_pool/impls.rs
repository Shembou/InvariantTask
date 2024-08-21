use super::enums::Errors;
use super::structs::*;
use super::utils::*;
use crate::lp_pool::utils::utils::FIXED_POINTS_DECIMALS_MULTIPLIER;

impl LpPool {
    fn calculate_total_staked_token_amount(self: &mut Self) -> Result<u64, Errors> {
        if self.price.0 == 0 {
            return Err(Errors::DivisionByZero);
        }

        let equivalent_token_amount = self
            .st_token_amount
            .0
            .checked_mul(self.price.0)
            .ok_or(Errors::MultiplicationError)?
            / FIXED_POINTS_DECIMALS_MULTIPLIER;

        let fee_percentage: Result<f64, Errors> = if self.st_token_amount.0 == 0 {
            Ok(self.min_fee.0 as f64)
        } else {
            Ok(utils::calculate_added_liquidity_fee(
                self.max_fee.0,
                self.min_fee.0,
                self.liquidity_target.0,
                self.lp_token_amount
                    .0
                    .checked_sub(equivalent_token_amount)
                    .ok_or(Errors::SubtractionOverflow)?,
            )?)
        };
        let fee_multiplier = (FIXED_POINTS_DECIMALS_MULTIPLIER as f64 / 100.0) - fee_percentage?;
        let final_token_amount = (equivalent_token_amount as f64 * fee_multiplier as f64)
            / (FIXED_POINTS_DECIMALS_MULTIPLIER as f64 / 100.0);

        let total_lp_tokens = (equivalent_token_amount as f64 - final_token_amount as f64)
            + self.lp_token_amount.0 as f64;

        if total_lp_tokens < 0.0 {
            return Err(Errors::CalculationError);
        }
        return Ok(total_lp_tokens as u64);
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

        utils::validate_input(pool.min_fee.0, pool.max_fee.0, pool.liquidity_target.0)?;
        Ok(pool)
    }

    pub fn add_liquidity(
        self: &mut Self,
        token_amount: TokenAmount,
    ) -> Result<LpTokenAmount, Errors> {
        let lp_tokens_minted = if self.lp_token_amount.0 == 0 {
            LpTokenAmount(token_amount.0)
        } else {
            let calculated_lp_tokens = self.calculate_total_staked_token_amount()?;
            let final_token_amount = utils::multiply_add_liquidity_token_amount(
                self.token_amount.0,
                calculated_lp_tokens,
            );
            LpTokenAmount(final_token_amount)
        };

        self.token_amount.0 = self
            .token_amount
            .0
            .checked_add(lp_tokens_minted.0)
            .ok_or(Errors::AdditionOverflow)?;
        self.lp_token_amount.0 = self
            .lp_token_amount
            .0
            .checked_add(lp_tokens_minted.0)
            .ok_or(Errors::AdditionOverflow)?;

        Ok(lp_tokens_minted)
    }

    pub fn remove_liquidity(
        &mut self,
        lp_token_amount: LpTokenAmount,
    ) -> Result<(TokenAmount, StakedTokenAmount), Errors> {
        //TODO: Add remove_liquidity logic
        if self.st_token_amount.0 != 0 {
            let equivalent_token_amount =
                (self.st_token_amount.0 * self.price.0) / FIXED_POINTS_DECIMALS_MULTIPLIER;

            let fee_percentage = utils::calculate_added_liquidity_fee(
                self.max_fee.0,
                self.min_fee.0,
                self.liquidity_target.0,
                self.token_amount.0 - equivalent_token_amount,
            )?;
            let fee_multiplier = (FIXED_POINTS_DECIMALS_MULTIPLIER as f64 / 100.0) - fee_percentage;
            let f64_token_value = (equivalent_token_amount as f64 * fee_multiplier as f64)
                / (FIXED_POINTS_DECIMALS_MULTIPLIER as f64 / 100.0);
            let mut final_token_amount = utils::round_up_to_nearest_ten(f64_token_value);
            if final_token_amount >= lp_token_amount.0 {
                final_token_amount -= lp_token_amount.0;

                let staked_tokens_left = final_token_amount / self.price.0;
                self.st_token_amount.0 -= staked_tokens_left;
                return Ok((TokenAmount(0), StakedTokenAmount(staked_tokens_left)));
            } else {
                let mut total_tokens = lp_token_amount.0;
                total_tokens -= final_token_amount;
                let calculated_fee = utils::calculate_added_liquidity_fee(
                    self.max_fee.0,
                    self.min_fee.0,
                    self.liquidity_target.0,
                    total_tokens,
                )?;
                let fee_multiplier =
                    (FIXED_POINTS_DECIMALS_MULTIPLIER as f64 / 100.0) - calculated_fee;
                let f64_token_value = (total_tokens as f64 * fee_multiplier as f64)
                    / (FIXED_POINTS_DECIMALS_MULTIPLIER as f64 / 100.0);
                total_tokens = utils::round_up_to_nearest_ten(f64_token_value);

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
        if staked_token_amount.0 == 0 {
            return Err(Errors::InvalidTokenAmount);
        }
        let equivalent_token_amount =
            utils::calculate_staked_tokens(staked_token_amount.0, self.price.0);
        let fee_percentage: Result<f64, Errors> = if self.st_token_amount.0 == 0 {
            Ok(self.min_fee.0 as f64)
        } else {
            Ok(utils::calculate_added_liquidity_fee(
                self.max_fee.0,
                self.min_fee.0,
                self.liquidity_target.0,
                self.token_amount
                    .0
                    .checked_sub(equivalent_token_amount)
                    .ok_or(Errors::SubtractionOverflow)?,
            )?)
        };

        let final_token_amount =
            utils::multiply_swap_token_amount(fee_percentage?, equivalent_token_amount);

        if self.token_amount.0 <= final_token_amount {
            return Err(Errors::InsufficientLiquidity);
        }

        self.st_token_amount.0 = self
            .st_token_amount
            .0
            .checked_add(staked_token_amount.0)
            .ok_or(Errors::AdditionOverflow)?;
        self.token_amount.0 = self
            .token_amount
            .0
            .checked_sub(final_token_amount)
            .ok_or(Errors::SubtractionOverflow)?;
        self.lp_token_amount.0 = self
            .lp_token_amount
            .0
            .checked_sub(equivalent_token_amount - final_token_amount)
            .ok_or(Errors::SubtractionOverflow)?;

        Ok(TokenAmount(final_token_amount))
    }
}
