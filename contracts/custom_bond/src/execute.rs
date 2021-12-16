use cosmwasm_std::{
    attr, to_binary, Addr, Attribute, CosmosMsg, Decimal, DepsMut, Env, MessageInfo, Response,
    StdError, StdResult, Uint128, WasmMsg,
};

use olympus_pro::{
    custom_bond::{Adjustment, Terms},
    custom_treasury::ExecuteMsg as CustomTreasuryExecuteMsg,
    querier::query_token_supply,
    utils::get_value_of_token,
};
use terraswap::asset::{Asset, AssetInfo};

use crate::{
    state::{
        read_bond_info, read_config, read_state, remove_bond_info, store_bond_info, store_config,
        store_state,
    },
    utils::{
        adjust, decay_debt, decimal_multiplication_in_256, get_current_debt, get_debt_ratio,
        get_max_payout, get_payout_for, get_pending_payout, get_true_bond_price,
    },
};

pub fn update_policy(deps: DepsMut, policy: String) -> StdResult<Response> {
    let mut config = read_config(deps.storage)?;

    config.policy = deps.api.addr_canonicalize(&policy)?;

    store_config(deps.storage, &config)?;

    Ok(Response::new().add_attributes(vec![
        attr("action", "update_policy"),
        attr("policy", policy),
    ]))
}

pub fn update_olympus_treasury(
    deps: DepsMut,
    info: MessageInfo,
    olympus_treasury: String,
) -> StdResult<Response> {
    let mut config = read_config(deps.storage)?;

    if config.olympus_dao != deps.api.addr_canonicalize(info.sender.as_str())? {
        return Err(StdError::generic_err("unauthorized"));
    }

    config.olympus_treasury = deps.api.addr_canonicalize(&olympus_treasury)?;

    store_config(deps.storage, &config)?;

    Ok(Response::new().add_attributes(vec![
        attr("action", "update_olympus_treasury"),
        attr("olympus_treasury", olympus_treasury),
    ]))
}

pub fn initialize_bond(
    deps: DepsMut,
    env: Env,
    terms: Terms,
    initial_debt: Uint128,
) -> StdResult<Response> {
    let mut state = read_state(deps.storage)?;

    let current_time = env.block.time.seconds();

    if !get_current_debt(state.clone(), current_time).is_zero() {
        return Err(StdError::generic_err("debt must be 0 for initialization"));
    }

    if terms.vesting_term < 129600 {
        return Err(StdError::generic_err(
            "vesting must be longer than 36 hours",
        ));
    }
    if terms.max_payout >= Decimal::percent(1) {
        return Err(StdError::generic_err("payout cannot be above 1 percent"));
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
    max_payout: Option<Decimal>,
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
        if max_payout >= Decimal::percent(1) {
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
    increment: Decimal,
    target: Decimal,
    buffer: u64,
) -> StdResult<Response> {
    let mut state = read_state(deps.storage)?;

    if increment
        > decimal_multiplication_in_256(state.terms.control_variable, Decimal::percent(3u64))
    {
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
    amount: Uint128,
    max_price: Decimal,
    depositor: String,
) -> StdResult<Response> {
    if amount.is_zero() {
        return Err(StdError::generic_err("amount is zero"));
    }

    let config = read_config(deps.storage)?;
    let mut state = read_state(deps.storage)?;

    let current_time = env.block.time.seconds();

    decay_debt(&mut state, current_time);

    let payout_total_supply = query_token_supply(
        &deps.querier,
        deps.api.addr_humanize(&config.payout_token)?.to_string(),
    )?;

    let native_price = get_true_bond_price(
        config.clone(),
        state.clone(),
        payout_total_supply,
        current_time,
    );

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

    let (payout, fee) = if config.fee_in_payout {
        get_payout_for(
            deps.as_ref(),
            config.clone(),
            state.clone(),
            value,
            payout_total_supply,
            current_time,
        )?
    } else {
        get_payout_for(
            deps.as_ref(),
            config.clone(),
            state.clone(),
            amount,
            payout_total_supply,
            current_time,
        )?
    };

    if payout
        < Uint128::from(
            10u128
                .checked_pow((config.payout_decimals - 2) as u32)
                .unwrap(),
        )
    {
        return Err(StdError::generic_err("bond too small"));
    } else if payout > get_max_payout(state.clone(), payout_total_supply)? {
        return Err(StdError::generic_err("bond too large"));
    }

    state.total_debt += value;
    if state.total_debt > state.terms.max_debt {
        return Err(StdError::generic_err("max capacity reached"));
    }

    let mut payout_from_treasury = payout;
    let mut amount_without_fee = amount;

    if config.fee_in_payout {
        payout_from_treasury += fee;
    } else {
        amount_without_fee = amount_without_fee.checked_sub(fee)?;
    }

    let mut bond_info =
        read_bond_info(deps.storage, deps.api.addr_canonicalize(&depositor)?).unwrap_or_default();
    bond_info.payout += payout;
    bond_info.vesting = state.terms.vesting_term;
    bond_info.last_time = current_time;
    bond_info.true_price_paid = native_price;
    store_bond_info(
        deps.storage,
        &bond_info,
        deps.api.addr_canonicalize(&depositor)?,
    )?;

    state.total_principal_bonded += amount_without_fee;
    state.total_payout_given += payout;
    state.payout_since_last_subsidy += payout;

    let mut messages: Vec<CosmosMsg> = vec![];
    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: deps.api.addr_humanize(&config.custom_treasury)?.to_string(),
        funds: vec![],
        msg: to_binary(&CustomTreasuryExecuteMsg::SendPayoutTokens {
            amount: payout_from_treasury,
        })?,
    }));

    if !fee.is_zero() {
        let asset = Asset {
            info: if config.fee_in_payout {
                AssetInfo::Token {
                    contract_addr: deps.api.addr_humanize(&config.payout_token)?.to_string(),
                }
            } else {
                config.principal_token.to_normal(deps.api)?
            },
            amount: fee,
        };
        messages.push(asset.into_msg(
            &deps.querier,
            deps.api.addr_humanize(&config.olympus_treasury)?,
        )?)
    }

    let mut bond_price = decimal_multiplication_in_256(
        state.terms.control_variable,
        get_debt_ratio(state.clone(), payout_total_supply, current_time),
    );
    if bond_price < state.terms.minimum_price {
        bond_price = state.terms.minimum_price;
    } else {
        state.terms.minimum_price = Decimal::zero();
    }

    let mut attrs: Vec<Attribute> = vec![
        attr("action", "deposit"),
        attr("amount", amount.to_string()),
        attr("payout", payout.to_string()),
        attr(
            "expires",
            (current_time + state.terms.vesting_term).to_string(),
        ),
        attr("bond_price", bond_price.to_string()),
        attr(
            "debt_ratio",
            get_debt_ratio(state.clone(), payout_total_supply, current_time).to_string(),
        ),
    ];

    let (adjusted, initial_value) = adjust(&mut state, current_time)?;
    if adjusted {
        attrs.push(attr("action", "adjust"));
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

pub fn redeem(deps: DepsMut, env: Env, user: String) -> StdResult<Response> {
    let mut bond_info = read_bond_info(deps.storage, deps.api.addr_canonicalize(&user)?)?;

    let time_since_last = env.block.time.seconds() - bond_info.last_time;

    let payout = get_pending_payout(bond_info.clone(), time_since_last);

    if payout.is_zero() {
        return Err(StdError::generic_err("nothing to redeem"));
    }

    bond_info.payout = bond_info.payout.checked_sub(payout)?;
    if bond_info.payout.is_zero() {
        remove_bond_info(deps.storage, deps.api.addr_canonicalize(&user)?);
    } else {
        bond_info.vesting = bond_info.vesting - time_since_last;
        bond_info.last_time = env.block.time.seconds();
        store_bond_info(deps.storage, &bond_info, deps.api.addr_canonicalize(&user)?)?;
    }

    let config = read_config(deps.storage)?;
    let asset = Asset {
        info: AssetInfo::Token {
            contract_addr: deps.api.addr_humanize(&config.payout_token)?.to_string(),
        },
        amount: payout,
    };

    Ok(Response::new()
        .add_message(asset.into_msg(&deps.querier, Addr::unchecked(user))?)
        .add_attributes(vec![
            attr("action", "redeem"),
            attr("amount", payout.to_string()),
        ]))
}
