use cosmwasm_std::{
    attr, to_binary, Attribute, CosmosMsg, Decimal, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult, Uint128, WasmMsg,
};

use olympus_pro::{
    custom_bond::{Adjustment, Terms},
    custom_treasury::ExecuteMsg as CustomTreasuryExecuteMsg,
    utils::get_value_of_token,
};
use terraswap::asset::Asset;

use crate::{
    state::{read_config, read_state, store_config, store_state},
    utils::{adjust, decay_debt, get_max_payout, get_payout_for, get_true_bond_price},
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
    buffer: u64,
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

pub fn deposit(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
    max_price: Uint128,
    depositor: String,
) -> StdResult<Response> {
    let config = read_config(deps.storage)?;
    let mut state = read_state(deps.storage)?;

    let current_time = env.block.time.seconds();

    decay_debt(env.clone(), &mut state);

    let native_price =
        get_true_bond_price(deps.as_ref(), config.clone(), state.clone(), current_time)?;
    if max_price < native_price {
        return Err(StdError::generic_err("slippage limit: more than max price"));
    }

    let value = get_value_of_token(
        Asset {
            info: config.principal_token.to_normal(deps.api)?,
            amount,
        },
        config.payout_decimals,
        config.principal_decimals,
    );

    let mut amount_without_fee = amount;

    let (payout, fee) = if config.fee_in_payout {
        get_payout_for(
            deps.as_ref(),
            config.clone(),
            state.clone(),
            value,
            current_time,
        )?
    } else {
        get_payout_for(
            deps.as_ref(),
            config.clone(),
            state.clone(),
            amount,
            current_time,
        )?
    };

    let mut payout_from_treasury = payout;

    if config.fee_in_payout {
        payout_from_treasury += fee;
    } else {
        amount_without_fee = amount_without_fee.checked_sub(fee)?;
    }

    if payout
        < Uint128::from(
            10u128
                .checked_pow((config.payout_decimals - 2) as u32)
                .unwrap(),
        )
    {
        return Err(StdError::generic_err("bond too small"));
    } else if payout > get_max_payout(deps.as_ref(), config.clone(), state.clone())? {
        return Err(StdError::generic_err("bond too large"));
    }

    state.total_debt += value;
    if state.total_debt > state.terms.max_debt {
        return Err(StdError::generic_err("max capacity reached"));
    }

    state.total_principal_bonded += amount_without_fee;
    state.total_payout_given += payout;
    state.payout_since_last_subsidy += payout;

    let mut messages: Vec<CosmosMsg> = vec![];
    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: deps.api.addr_humanize(&config.custom_treasury)?.to_string(),
        funds: vec![],
        msg: to_binary(&CustomTreasuryExecuteMsg::SendPayoutTokens {
            amount: payout_from_treasury,
        })
        .unwrap(),
    }));
    if !fee.is_zero() {
        if config.fee_in_payout {
            let asset = Asset {
                info: config.payout_token.to_normal(deps.api)?,
                amount: fee,
            };
            messages.push(asset.into_msg(
                &deps.querier,
                deps.api.addr_humanize(&config.olympus_treasury)?,
            )?)
        } else {
            // TODO safe transfer from
        }
    }

    let mut attrs: Vec<Attribute> = vec![
        attr("action", "deposit"),
        attr("amount", amount.to_string()),
        attr("payout", payout.to_string()),
        attr(
            "expires",
            (current_time + state.terms.vesting_term).to_string(),
        ),
        // attr("bond_price", "bond_price"),
        // attr("debt_ratio", "debt_ratio"),
    ];

    let (adjusted, initial_value) = adjust(&mut state, current_time)?;
    if adjusted {
        attrs.push(attr("action", "control_variable_adjusted"));
        attrs.push(attr("initial", initial_value.to_string()));
        attrs.push(attr(
            "control_variable",
            state.terms.control_variable.to_string(),
        ));
        attrs.push(attr("rate", state.adjustment.rate.to_string()));
    }

    store_state(deps.storage, &state)?;

    Ok(Response::new().add_attributes(attrs).add_messages(messages))
}
