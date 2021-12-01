use cosmwasm_std::{
    attr, Decimal, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult, Uint128,
};

use olympus_pro::{
    custom_bond::{Adjustment, FeeTier, State, Terms},
    querier::query_total_supply,
    utils::get_value_of_token,
};
use terraswap::asset::Asset;

use crate::state::{read_config, read_state, store_config, store_state, Config};

fn get_debt_decay(state: State, current_time: u64) -> Uint128 {
    let time_since_last = current_time - state.last_decay;
    if time_since_last > state.terms.vesting_term {
        state.total_debt
    } else {
        state.total_debt
            * Decimal::from_ratio(time_since_last as u128, state.terms.vesting_term as u128)
    }
}

fn get_current_debt(state: State, current_time: u64) -> Uint128 {
    state.total_debt - get_debt_decay(state, current_time)
}

pub fn decay_debt(env: Env, state: &mut State) {
    state.total_debt = state.total_debt - get_debt_decay(state.clone(), env.block.time.seconds());
    state.last_decay = env.block.time.seconds();
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

pub fn get_debt_ratio(
    deps: Deps,
    config: Config,
    state: State,
    current_time: u64,
) -> StdResult<Uint128> {
    let current_debt = get_current_debt(state, current_time);
    let payout_total_supply =
        query_total_supply(&deps.querier, &config.payout_token.to_normal(deps.api)?)?;

    Ok(current_debt
        * Decimal::from_ratio(
            Uint128::from(10u128.checked_pow(config.payout_decimals as u32).unwrap()),
            payout_total_supply,
        )
        * Decimal::from_ratio(
            Uint128::from(1u128),
            Uint128::from(10u128.checked_pow(18u32).unwrap()),
        ))
}

pub fn get_bond_price(
    deps: Deps,
    config: Config,
    state: State,
    current_time: u64,
) -> StdResult<Uint128> {
    let price = state.terms.control_variable
        * Decimal::from_ratio(
            get_debt_ratio(deps, config.clone(), state.clone(), current_time)?,
            Uint128::from(
                10u128
                    .checked_pow((config.payout_decimals - 5) as u32)
                    .unwrap(),
            ),
        );
    if price < state.terms.minimum_price {
        Ok(price)
    } else {
        Ok(state.terms.minimum_price)
    }
}

pub fn get_true_bond_price(
    deps: Deps,
    config: Config,
    state: State,
    current_time: u64,
) -> StdResult<Uint128> {
    let bond_price = get_bond_price(deps, config.clone(), state.clone(), current_time)?;
    Ok(bond_price
        + bond_price
            * get_current_olympus_fee(config, state)
            * Decimal::from_ratio(Uint128::from(1u128), Uint128::from(1000000u128)))
}

pub fn get_payout_for(
    deps: Deps,
    config: Config,
    state: State,
    value: Uint128,
    current_time: u64,
) -> StdResult<(Uint128, Uint128)> {
    let current_olympus_fee = get_current_olympus_fee(config.clone(), state.clone());

    if config.fee_in_payout {
        let total = value
            * Decimal::from_ratio(
                Uint128::from(1u128),
                get_bond_price(deps, config.clone(), state.clone(), current_time)?
                    * Uint128::from(100000000000u128),
            );
        let fee = total
            * current_olympus_fee
            * Decimal::from_ratio(Uint128::from(1u128), Uint128::from(1000000u128));
        Ok((total.checked_sub(fee)?, fee))
    } else {
        let fee = value
            * current_olympus_fee
            * Decimal::from_ratio(Uint128::from(1u128), Uint128::from(1000000u128));
        let payout = get_value_of_token(
            Asset {
                info: config.principal_token.to_normal(deps.api)?,
                amount: value.checked_sub(fee)?,
            },
            config.payout_decimals,
            config.principal_decimals,
        ) * Decimal::from_ratio(
            Uint128::from(1u128),
            get_bond_price(deps, config.clone(), state.clone(), current_time)?
                * Uint128::from(100000000000u128),
        );
        Ok((payout, fee))
    }
}

pub fn get_max_payout(deps: Deps, config: Config, state: State) -> StdResult<Uint128> {
    let payout_total_supply =
        query_total_supply(&deps.querier, &config.payout_token.to_normal(deps.api)?)?;

    Ok(
        payout_total_supply
            * Decimal::from_ratio(state.terms.max_payout, Uint128::from(100000u128)),
    )
}

pub fn adjust(state: &mut State, current_time: u64) -> StdResult<(bool, Uint128)> {
    let time_can_adjust = state.adjustment.last_time + state.adjustment.buffer;
    if !state.adjustment.rate.is_zero() && current_time >= time_can_adjust {
        let inital = state.terms.control_variable;
        if state.adjustment.addition {
            state.terms.control_variable += state.adjustment.rate;
            if state.terms.control_variable >= state.adjustment.target {
                state.adjustment.rate = Uint128::zero();
            }
        } else {
            state.terms.control_variable = state
                .terms
                .control_variable
                .checked_sub(state.adjustment.rate)?;
            if state.terms.control_variable <= state.adjustment.target {
                state.adjustment.rate = Uint128::zero();
            }
        }
        state.adjustment.last_time = current_time;
        Ok((true, inital))
    } else {
        Ok((false, Uint128::zero()))
    }
}
