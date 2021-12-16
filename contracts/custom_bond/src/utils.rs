use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::{Decimal, Deps, Fraction, MessageInfo, StdError, StdResult, Storage, Uint128};
use olympus_pro::{
    custom_bond::{BondInfo, FeeTier, State},
    utils::get_value_of_token,
};
use terraswap::asset::{Asset, AssetInfoRaw};

use crate::state::{read_config, Config};

fn get_debt_decay(state: State, current_time: u64) -> Uint128 {
    let time_since_last = current_time - state.last_decay;
    if time_since_last > state.terms.vesting_term {
        state.total_debt
    } else {
        state.total_debt
            * Decimal::from_ratio(time_since_last as u128, state.terms.vesting_term as u128)
    }
}

pub fn get_current_debt(state: State, current_time: u64) -> Uint128 {
    state
        .total_debt
        .checked_sub(get_debt_decay(state, current_time))
        .unwrap()
}

pub fn decay_debt(state: &mut State, current_time: u64) {
    state.total_debt = state
        .total_debt
        .checked_sub(get_debt_decay(state.clone(), current_time))
        .unwrap();
    state.last_decay = current_time;
}

pub fn get_current_olympus_fee(config: Config, state: State) -> Decimal {
    for fee_tier in config.fee_tiers.clone() {
        if state.total_principal_bonded < fee_tier.tier_ceiling {
            return fee_tier.fee_rate;
        }
    }

    config
        .fee_tiers
        .last()
        .unwrap_or(&FeeTier {
            tier_ceiling: Uint128::zero(),
            fee_rate: Decimal::zero(),
        })
        .fee_rate
}

pub fn get_debt_ratio(state: State, payout_total_supply: Uint128, current_time: u64) -> Decimal {
    let current_debt = get_current_debt(state, current_time);

    Decimal::from_ratio(current_debt, payout_total_supply)
}

pub fn get_bond_price(state: State, payout_total_supply: Uint128, current_time: u64) -> Decimal {
    let price = decimal_multiplication_in_256(
        state.terms.control_variable,
        get_debt_ratio(state.clone(), payout_total_supply, current_time),
    );
    std::cmp::max(price, state.terms.minimum_price)
}

pub fn get_true_bond_price(
    config: Config,
    state: State,
    payout_total_supply: Uint128,
    current_time: u64,
) -> Decimal {
    let bond_price = get_bond_price(state.clone(), payout_total_supply, current_time);
    decimal_summation_in_256(
        bond_price,
        decimal_multiplication_in_256(bond_price, get_current_olympus_fee(config, state)),
    )
}

pub fn get_payout_for(
    deps: Deps,
    config: Config,
    state: State,
    value: Uint128,
    payout_total_supply: Uint128,
    current_time: u64,
) -> StdResult<(Uint128, Uint128)> {
    let current_olympus_fee = get_current_olympus_fee(config.clone(), state.clone());

    let bond_price = get_bond_price(state.clone(), payout_total_supply, current_time);

    if config.fee_in_payout {
        let total = value * bond_price.inv().unwrap();
        let fee = total * current_olympus_fee;
        Ok((total.checked_sub(fee)?, fee))
    } else {
        let fee = value * current_olympus_fee;
        let payout = get_value_of_token(
            Asset {
                info: config.principal_token.to_normal(deps.api)?,
                amount: value.checked_sub(fee)?,
            },
            config.payout_decimals,
            config.principal_decimals,
        ) * bond_price.inv().unwrap();
        Ok((payout, fee))
    }
}

pub fn get_max_payout(state: State, payout_total_supply: Uint128) -> StdResult<Uint128> {
    Ok(payout_total_supply * state.terms.max_payout)
}

pub fn adjust(state: &mut State, current_time: u64) -> StdResult<(bool, Decimal)> {
    let time_can_adjust = state.adjustment.last_time + state.adjustment.buffer;
    if !state.adjustment.rate.is_zero() && current_time >= time_can_adjust {
        let inital = state.terms.control_variable;
        if state.adjustment.addition {
            state.terms.control_variable =
                decimal_summation_in_256(state.terms.control_variable, state.adjustment.rate);
            if state.terms.control_variable >= state.adjustment.target {
                state.adjustment.rate = Decimal::zero();
            }
        } else {
            state.terms.control_variable =
                decimal_subtraction_in_256(state.terms.control_variable, state.adjustment.rate);
            if state.terms.control_variable <= state.adjustment.target {
                state.adjustment.rate = Decimal::zero();
            }
        }
        state.adjustment.last_time = current_time;
        Ok((true, inital))
    } else {
        Ok((false, Decimal::zero()))
    }
}

pub fn get_received_native_fund(storage: &dyn Storage, info: MessageInfo) -> StdResult<Uint128> {
    let config = read_config(storage)?;

    if info.funds.len() != 1u64 as usize {
        return Err(StdError::generic_err("invalid denom received"));
    }
    if let AssetInfoRaw::NativeToken { denom } = config.principal_token {
        let amount: Uint128 = info
            .funds
            .iter()
            .find(|c| c.denom == *denom)
            .map(|c| Uint128::from(c.amount))
            .unwrap_or_else(Uint128::zero);
        Ok(amount)
    } else {
        Err(StdError::generic_err("not support cw20 token"))
    }
}

pub fn get_pending_payout(bond_info: BondInfo, time_since_last: u64) -> Uint128 {
    let mut payout = bond_info.payout
        * Decimal::from_ratio(
            Uint128::from(time_since_last as u128),
            Uint128::from(bond_info.vesting as u128),
        );
    if payout > bond_info.payout {
        payout = bond_info.payout;
    }

    payout
}

pub fn decimal_multiplication_in_256(a: Decimal, b: Decimal) -> Decimal {
    let a_u256: Decimal256 = a.into();
    let b_u256: Decimal256 = b.into();
    let c_u256: Decimal = (b_u256 * a_u256).into();
    c_u256
}

/// return a + b
pub fn decimal_summation_in_256(a: Decimal, b: Decimal) -> Decimal {
    let a_u256: Decimal256 = a.into();
    let b_u256: Decimal256 = b.into();
    let c_u256: Decimal = (b_u256 + a_u256).into();
    c_u256
}

/// return a - b
pub fn decimal_subtraction_in_256(a: Decimal, b: Decimal) -> Decimal {
    let a_u256: Decimal256 = a.into();
    let b_u256: Decimal256 = b.into();
    let c_u256: Decimal = (a_u256 - b_u256).into();
    c_u256
}
