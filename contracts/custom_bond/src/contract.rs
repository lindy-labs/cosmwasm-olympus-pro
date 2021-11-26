#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

use olympus_pro::custom_bond::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::InitializeBond {
            control_variable,
            vesting_term,
            minimum_price,
            max_payout,
            max_debt,
            initial_debt,
        } => Ok(Response::default()),
        ExecuteMsg::SetBondTerms { parameter, input } => Ok(Response::default()),
        ExecuteMsg::SetAdjustment {
            addition,
            increment,
            target,
            buffer,
        } => Ok(Response::default()),
        ExecuteMsg::UpdateConfig { olympus_treasury } => Ok(Response::default()),
        ExecuteMsg::PaySubsidy {} => Ok(Response::default()),
        ExecuteMsg::Deposit {
            amount,
            max_price,
            depositor,
        } => Ok(Response::default()),
        ExecuteMsg::Redeem { depositor } => Ok(Response::default()),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&"query_test"),
        QueryMsg::BondPrice {} => to_binary(&"query_test"),
        QueryMsg::MaxPayout {} => to_binary(&"query_test"),
        QueryMsg::PayoutFor { .. } => to_binary(&"query_test"),
        QueryMsg::Debt {} => to_binary(&"query_test"),
        QueryMsg::PayoutInfo { .. } => to_binary(&"query_test"),
        QueryMsg::CurrentOlympusFee {} => to_binary(&"query_test"),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}
