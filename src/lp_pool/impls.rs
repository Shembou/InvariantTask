use super::enums::Errors;
use super::structs::*;

impl LpPool {
    //TODO: Remove if unnecessary

    const FIXED_POINTS_DECIMALS_MULTIPLIER: u64 = 1000000;

    fn proportional(&self, amount: u64, numerator: u64, denominator: u64) -> Result<u64, Errors> {
        if denominator == 0 {
            return Ok(amount);
        }
        let receive_amount = (amount * numerator) / denominator;
        if receive_amount == 0 {
            return Err(Errors::MultiplicationError);
        }
        // if receive_amount.is_nan() || receive_amount.is_infinite() {
        //     return Err(Errors::InvalidFeeRange);
        // }

        Ok(receive_amount)
    }
    fn calculate_added_liquidity_fee(&self, token_amount: u64) -> Result<Percentage, Errors> {
        if token_amount >= self.liquidity_target.0 {
            Ok(self.min_fee)
        } else {
            // let deficit = self.liquidity_target.0 - token_amount;
            // let deficit_ratio = deficit / self.liquidity_target.0;
            // let fee_range = self.max_fee.0 - self.min_fee.0;
            // let fee = self.min_fee.0 + (fee_range * deficit_ratio);
            let calculation_result = self.proportional(
                self.max_fee.0.saturating_sub(self.min_fee.0),
                token_amount,
                self.liquidity_target.0,
            )?;

            let fee = self.max_fee.0 - calculation_result;

            Ok(Percentage(fee))
        }
    }

    fn calculate_fee(&self) -> Result<Percentage, Errors> {
        if self.token_amount.0 >= self.liquidity_target.0 {
            Ok(self.min_fee)
        } else {
            let deficit = self.liquidity_target.0 - self.token_amount.0;
            let deficit_ratio = deficit / self.liquidity_target.0;
            let fee_range = self.max_fee.0 - self.min_fee.0;
            let multiply_values = Self::multiply(fee_range, deficit_ratio)?;
            let fee = self.min_fee.0 + multiply_values;

            Ok(Percentage(fee))
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

    fn multiply(first_variable: u64, second_variable: u64) -> Result<u64, Errors> {
        let multiplication_result =
            first_variable * second_variable / Self::FIXED_POINTS_DECIMALS_MULTIPLIER;
        if multiplication_result == 0 {
            return Err(Errors::MultiplicationError);
        }
        Ok(multiplication_result)
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
            //TODO: Change logic to match the end result
            let proportional_lp_tokens = token_amount.0 * (self.lp_token_amount.0 + token_amount.0)
                / (self.token_amount.0 + token_amount.0);
            let fee_percentage = self.calculate_added_liquidity_fee(token_amount.0)?;
            let fee_multiplier = (Self::FIXED_POINTS_DECIMALS_MULTIPLIER / 100) - fee_percentage.0;

            let calculate_result =
                (token_amount.0 * fee_multiplier) / (Self::FIXED_POINTS_DECIMALS_MULTIPLIER / 100);
            LpTokenAmount(calculate_result)
        };

        self.token_amount.0 += lp_tokens_minted.0;
        self.lp_token_amount.0 += lp_tokens_minted.0;

        Ok(lp_tokens_minted)
    }

    pub fn remove_liquidity(
        &mut self,
        lp_token_amount: LpTokenAmount,
    ) -> Result<(TokenAmount, StakedTokenAmount), Errors> {
        let token_share = (self.token_amount.0 * lp_token_amount.0) / self.lp_token_amount.0;

        let st_token_share = (self.st_token_amount.0 * lp_token_amount.0) / self.lp_token_amount.0;

        self.token_amount.0 -= token_share;
        self.st_token_amount.0 -= st_token_share;
        self.lp_token_amount.0 -= lp_token_amount.0;

        Ok((TokenAmount(token_share), StakedTokenAmount(st_token_share)))
    }

    pub fn swap(&mut self, staked_token_amount: StakedTokenAmount) -> Result<TokenAmount, Errors> {
        // let equivalent_token_amount = Self::di(staked_token_amount.0, self.price.0)?;

        let equivalent_token_amount =
            (staked_token_amount.0 * self.price.0) / Self::FIXED_POINTS_DECIMALS_MULTIPLIER;
        let fee_percentage: Result<Percentage, Errors> = if self.st_token_amount.0 == 0 {
            Ok(Percentage(self.min_fee.0))
        } else {
            Ok(self
                .calculate_added_liquidity_fee(self.lp_token_amount.0 - equivalent_token_amount)?)
        };
        // let fee_percentage = self.calculate_added_liquidity_fee(equivalent_token_amount)?;
        let fee_multiplier =
            (Self::FIXED_POINTS_DECIMALS_MULTIPLIER / 100) - fee_percentage.unwrap().0;
        let final_token_amount = (equivalent_token_amount * fee_multiplier)
            / (Self::FIXED_POINTS_DECIMALS_MULTIPLIER / 100);
        self.st_token_amount.0 += staked_token_amount.0;
        self.token_amount.0 -= final_token_amount;
        //self.lp_token_amount.0 -= equivalent_token_amount - final_token_amount;

        Ok(TokenAmount(final_token_amount))
    }
}
