use cosmwasm_std::{
    attr, Decimal, DepsMut, Env, MessageInfo, Response, StdError, StdResult, Uint128,
};

use olympus_pro::custom_bond::{Adjustment, Terms};

use crate::{
    state::{read_config, read_state, store_config, store_state},
    utils::decay_debt,
};

pub fn update_config(
    deps: DepsMut,
    policy: Option<String>,
    olympus_treasury: Option<String>,
) -> StdResult<Response> {
    let mut config = read_config(deps.storage)?;

    if let Some(policy) = policy {
        config.policy = deps.api.addr_canonicalize(&policy)?;
    }

    if let Some(olympus_treasury) = olympus_treasury {
        config.olympus_treasury = deps.api.addr_canonicalize(&olympus_treasury)?;
    }

    store_config(deps.storage, &config)?;

    Ok(Response::new().add_attributes(vec![attr("action", "update_config")]))
}

pub fn initialize_bond(
    deps: DepsMut,
    env: Env,
    terms: Terms,
    initial_debt: Uint128,
) -> StdResult<Response> {
    let mut state = read_state(deps.storage)?;

    if !state.current_debt.is_zero() {
        return Err(StdError::generic_err("debt must be 0 for initialization"));
    }

    state.terms = terms;
    state.last_decay = env.block.time.seconds();
    state.total_debt = initial_debt;

    store_state(deps.storage, &state)?;

    Ok(Response::new().add_attributes(vec![attr("action", "initialize_bond")]))
}

pub fn set_bond_terms(
    deps: DepsMut,
    vesting_term: Option<u64>,
    max_payout: Option<Uint128>,
    max_debt: Option<Uint128>,
) -> StdResult<Response> {
    let mut state = read_state(deps.storage)?;

    if let Some(vesting_term) = vesting_term {
        if vesting_term < 129600 {
            return Err(StdError::generic_err(
                "vesting must be longer than 36 hours",
            ));
        }
        state.terms.vesting_term = vesting_term;
    }

    if let Some(max_payout) = max_payout {
        if max_payout < Uint128::from(1000u128) {
            return Err(StdError::generic_err("payout cannot be above 1 percent"));
        }
        state.terms.max_payout = max_payout;
    }

    if let Some(max_debt) = max_debt {
        state.terms.max_debt = max_debt;
    }

    store_state(deps.storage, &state)?;

    Ok(Response::new().add_attributes(vec![attr("action", "set_bond_terms")]))
}

pub fn set_adjustment(
    deps: DepsMut,
    env: Env,
    addition: bool,
    increment: Uint128,
    target: Uint128,
    buffer: Uint128,
) -> StdResult<Response> {
    let mut state = read_state(deps.storage)?;

    if increment > state.terms.control_variable * Decimal::percent(3u64) {
        return Err(StdError::generic_err("increment too large"));
    }

    state.adjustment = Adjustment {
        addition,
        rate: increment,
        target,
        buffer,
        last_time: env.block.time.seconds(),
    };

    store_state(deps.storage, &state)?;

    Ok(Response::new().add_attributes(vec![attr("action", "set_adjustment")]))
}

pub fn pay_subsidy(deps: DepsMut, info: MessageInfo) -> StdResult<Response> {
    if read_config(deps.storage)?.subsidy_router
        != deps.api.addr_canonicalize(info.sender.as_str())?
    {
        return Err(StdError::generic_err("only subsidy controller"));
    }

    let mut state = read_state(deps.storage)?;

    state.payout_since_last_subsidy = Uint128::zero();

    store_state(deps.storage, &state)?;

    Ok(Response::new().add_attributes(vec![attr("action", "pay_subsidy")]))
}

pub fn deposit(deps: DepsMut, env: Env, info: MessageInfo) -> StdResult<Response> {
    let config = read_config(deps.storage)?;
    let mut state = read_state(deps.storage)?;

    decay_debt(env, &mut state);

    store_state(deps.storage, &state)?;

    Ok(Response::new().add_attributes(vec![attr("action", "pay_subsidy")]))
}
