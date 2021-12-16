use cosmwasm_std::{
    to_binary, Decimal, Deps, Env, QuerierWrapper, QueryRequest, StdResult, Uint128, WasmQuery,
};

use olympus_pro::{
    custom_bond::{BondInfoResponse, ConfigResponse, State},
    custom_treasury::{
        ConfigResponse as CustomTreasuryConfigResponse, QueryMsg as CustomTreasuryQueryMsg,
    },
    querier::query_token_supply,
};

use crate::{
    state::{read_bond_info, read_config, read_state},
    utils::{
        get_bond_price, get_current_debt, get_current_olympus_fee, get_payout_for,
        get_pending_payout,
    },
};

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = read_config(deps.storage)?;

    let resp = ConfigResponse {
        custom_treasury: deps.api.addr_humanize(&config.custom_treasury)?.to_string(),
        payout_token: deps.api.addr_humanize(&config.payout_token)?.to_string(),
        principal_token: config.principal_token.to_normal(deps.api)?,
        olympus_treasury: deps
            .api
            .addr_humanize(&config.olympus_treasury)?
            .to_string(),
        subsidy_router: deps.api.addr_humanize(&config.subsidy_router)?.to_string(),
        policy: deps.api.addr_humanize(&config.policy)?.to_string(),
        olympus_dao: deps.api.addr_humanize(&config.olympus_dao)?.to_string(),
        fee_tiers: config.fee_tiers,
        fee_in_payout: config.fee_in_payout,
    };

    Ok(resp)
}

pub fn query_state(deps: Deps) -> StdResult<State> {
    let state = read_state(deps.storage)?;

    Ok(state)
}

pub fn query_custom_treasury_config(
    querier: &QuerierWrapper,
    custom_treasury: String,
) -> StdResult<CustomTreasuryConfigResponse> {
    let res: CustomTreasuryConfigResponse =
        querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: custom_treasury,
            msg: to_binary(&CustomTreasuryQueryMsg::Config {})?,
        }))?;

    Ok(res)
}

pub fn query_bond_info(deps: Deps, env: Env, user: String) -> StdResult<BondInfoResponse> {
    let bond_info = read_bond_info(deps.storage, deps.api.addr_canonicalize(&user)?)?;

    let time_since_last = env.block.time.seconds() - bond_info.last_time;

    let pending_payout = get_pending_payout(bond_info.clone(), time_since_last);

    Ok(BondInfoResponse {
        info: bond_info,
        pending_payout,
    })
}

pub fn query_current_olympus_fee(deps: Deps) -> StdResult<Decimal> {
    let config = read_config(deps.storage)?;
    let state = read_state(deps.storage)?;
    Ok(get_current_olympus_fee(config, state))
}

pub fn query_bond_price(deps: Deps, env: Env) -> StdResult<Decimal> {
    let config = read_config(deps.storage)?;
    let state = read_state(deps.storage)?;

    let payout_total_supply = query_token_supply(
        &deps.querier,
        deps.api.addr_humanize(&config.payout_token)?.to_string(),
    )?;

    Ok(get_bond_price(
        state,
        payout_total_supply,
        env.block.time.seconds(),
    ))
}

pub fn query_payout_for(deps: Deps, env: Env, value: Uint128) -> StdResult<(Uint128, Uint128)> {
    let config = read_config(deps.storage)?;
    let state = read_state(deps.storage)?;

    let payout_total_supply = query_token_supply(
        &deps.querier,
        deps.api.addr_humanize(&config.payout_token)?.to_string(),
    )?;

    Ok(get_payout_for(
        deps,
        config,
        state,
        value,
        payout_total_supply,
        env.block.time.seconds(),
    )?)
}

pub fn query_current_debt(deps: Deps, env: Env) -> StdResult<Uint128> {
    let state = read_state(deps.storage)?;

    Ok(get_current_debt(state, env.block.time.seconds()))
}
