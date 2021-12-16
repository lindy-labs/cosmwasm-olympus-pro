#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_std::{
    from_binary, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
};

use cw20::Cw20ReceiveMsg;
use olympus_pro::{
    custom_bond::{Cw20HookMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, State},
    querier::{query_decimals, query_token_decimals},
};
use terraswap::asset::AssetInfoRaw;

use crate::{
    execute::{
        deposit, initialize_bond, pay_subsidy, redeem, set_adjustment, set_bond_terms,
        update_olympus_treasury, update_policy,
    },
    query::{
        query_bond_info, query_bond_price, query_config, query_current_debt,
        query_current_olympus_fee, query_custom_treasury_config, query_payout_for, query_state,
    },
    state::{read_config, store_config, store_state, Config},
    utils::get_received_native_fund,
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let custom_treasury_config =
        query_custom_treasury_config(&deps.querier, msg.custom_treasury.clone())?;

    let payout_decimals =
        query_token_decimals(&deps.querier, custom_treasury_config.payout_token.clone())?;
    let principal_decimals = query_decimals(&deps.querier, &msg.principal_token)?;

    store_config(
        deps.storage,
        &Config {
            custom_treasury: deps.api.addr_canonicalize(&msg.custom_treasury)?,
            payout_token: deps
                .api
                .addr_canonicalize(&custom_treasury_config.payout_token)?,
            principal_token: msg.principal_token.to_raw(deps.api)?,
            olympus_treasury: deps.api.addr_canonicalize(&msg.olympus_treasury)?,
            subsidy_router: deps.api.addr_canonicalize(&msg.subsidy_router)?,
            policy: deps.api.addr_canonicalize(&msg.initial_owner)?,
            olympus_dao: deps.api.addr_canonicalize(&msg.olympus_dao)?,
            fee_tiers: msg.fee_tiers,
            fee_in_payout: msg.fee_in_payout,
            payout_decimals,
            principal_decimals,
        },
    )?;

    store_state(deps.storage, &State::default())?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Receive(msg) => receive_cw20(deps, env, info, msg),
        ExecuteMsg::Deposit {
            max_price,
            depositor,
        } => {
            let amount = get_received_native_fund(deps.storage, info)?;
            deposit(deps, env, amount, max_price, depositor)
        }
        ExecuteMsg::Redeem {} => redeem(deps, env, info.sender.to_string()),
        ExecuteMsg::PaySubsidy {} => pay_subsidy(deps, info),
        ExecuteMsg::UpdateOlympusTreasury { olympus_treasury } => {
            update_olympus_treasury(deps, info, olympus_treasury)
        }
        _ => {
            assert_policy_privilege(deps.as_ref(), info)?;
            match msg {
                ExecuteMsg::UpdatePolicy { policy } => update_policy(deps, policy),
                ExecuteMsg::InitializeBond {
                    terms,
                    initial_debt,
                } => initialize_bond(deps, env, terms, initial_debt),
                ExecuteMsg::SetBondTerms {
                    vesting_term,
                    max_payout,
                    max_debt,
                } => set_bond_terms(deps, vesting_term, max_payout, max_debt),
                ExecuteMsg::SetAdjustment {
                    addition,
                    increment,
                    target,
                    buffer,
                } => set_adjustment(deps, env, addition, increment, target, buffer),
                _ => panic!("do not enter here"),
            }
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::State {} => to_binary(&query_state(deps)?),
        QueryMsg::BondPrice {} => to_binary(&query_bond_price(deps, env)?),
        QueryMsg::PayoutFor { value } => to_binary(&query_payout_for(deps, env, value)?),
        QueryMsg::CurrentDebt {} => to_binary(&query_current_debt(deps, env)?),
        QueryMsg::CurrentOlympusFee {} => to_binary(&query_current_olympus_fee(deps)?),
        QueryMsg::BondInfo { user } => to_binary(&query_bond_info(deps, env, user)?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}

pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> StdResult<Response> {
    match from_binary(&cw20_msg.msg)? {
        Cw20HookMsg::Deposit {
            max_price,
            depositor,
        } => {
            let config = read_config(deps.storage)?;
            if let AssetInfoRaw::Token { contract_addr } = config.principal_token {
                if deps.api.addr_humanize(&contract_addr)? == info.sender.clone() {
                    return deposit(deps, env, cw20_msg.amount, max_price, depositor);
                }
            }
            Err(StdError::generic_err("invalid cw20 token"))
        }
    }
}

fn assert_policy_privilege(deps: Deps, info: MessageInfo) -> StdResult<()> {
    if read_config(deps.storage)?.policy != deps.api.addr_canonicalize(info.sender.as_str())? {
        return Err(StdError::generic_err("unauthorized"));
    }

    Ok(())
}
