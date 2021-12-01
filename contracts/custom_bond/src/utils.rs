use cosmwasm_std::{
    attr, Decimal, DepsMut, Env, MessageInfo, Response, StdError, StdResult, Uint128,
};

use olympus_pro::custom_bond::{Adjustment, FeeTier, State, Terms};

use crate::state::{read_config, read_state, store_config, store_state, Config};

fn debt_decay(current_time: u64, state: State) -> Uint128 {
    let time_since_last = current_time - state.last_decay;
    if time_since_last > state.terms.vesting_term {
        state.total_debt
    } else {
        state.total_debt
            * Decimal::from_ratio(time_since_last as u128, state.terms.vesting_term as u128)
    }
}

fn current_debt(current_time: u64, state: State) -> Uint128 {
    state.total_debt - debt_decay(current_time, state)
}

pub fn decay_debt(env: Env, state: &mut State) {
    state.total_debt = state.total_debt - debt_decay(env.block.time.seconds(), state.clone());
    state.last_decay = env.block.time.seconds();
}

pub fn current_olympus_fee(config: Config, state: State) -> Decimal {
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
