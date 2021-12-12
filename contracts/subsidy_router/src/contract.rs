#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_std::{
    attr, to_binary, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, QueryRequest, Response,
    StdError, StdResult, WasmMsg, WasmQuery,
};

use olympus_pro::{
    custom_bond::{
        ExecuteMsg as CustomBondExecuteMsg, QueryMsg as CustomBondQueryMsg,
        State as CustomBondState,
    },
    subsidy_router::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
};

use crate::query::{query_bond, query_config};
use crate::state::{
    read_config, read_subsidy_controller,
    remove_subsidy_controller as remove_subsidy_controller_state, store_config,
    store_subsidy_controller, Config,
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    store_config(
        deps.storage,
        &Config {
            policy: deps.api.addr_canonicalize(&msg.policy)?,
        },
    )?;

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
        ExecuteMsg::PaySubsidy {} => pay_subsidy(deps, info),
        _ => {
            assert_policy_privilege(deps.as_ref(), info)?;
            match msg {
                ExecuteMsg::UpdateConfig { policy } => update_config(deps, policy),
                ExecuteMsg::AddSubsidyController {
                    subsidy_controller,
                    bond,
                } => add_subsidy_controller(deps, subsidy_controller, bond),
                ExecuteMsg::RemoveSubsidyController { subsidy_controller } => {
                    remove_subsidy_controller(deps, subsidy_controller)
                }
                _ => panic!("do not enter here"),
            }
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::BondForController { subsidy_controller } => {
            to_binary(&query_bond(deps, subsidy_controller)?)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}

fn assert_policy_privilege(deps: Deps, info: MessageInfo) -> StdResult<()> {
    if read_config(deps.storage)?.policy != deps.api.addr_canonicalize(info.sender.as_str())? {
        return Err(StdError::generic_err("unauthorized"));
    }

    Ok(())
}

fn pay_subsidy(deps: DepsMut, info: MessageInfo) -> StdResult<Response> {
    let bond = read_subsidy_controller(
        deps.storage,
        &deps.api.addr_canonicalize(info.sender.as_str())?,
    )?;

    let custom_bond_state: CustomBondState =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: deps.api.addr_humanize(&bond)?.to_string(),
            msg: to_binary(&CustomBondQueryMsg::State {})?,
        }))?;

    Ok(Response::new()
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps.api.addr_humanize(&bond)?.to_string(),
            funds: vec![],
            msg: to_binary(&CustomBondExecuteMsg::PaySubsidy {}).unwrap(),
        }))
        .add_attributes(vec![
            attr("action", "pay_subsidy"),
            attr("amount", custom_bond_state.payout_since_last_subsidy),
        ]))
}

fn add_subsidy_controller(
    deps: DepsMut,
    subsidy_controller: String,
    bond: String,
) -> StdResult<Response> {
    store_subsidy_controller(
        deps.storage,
        &deps.api.addr_canonicalize(&subsidy_controller)?,
        &deps.api.addr_canonicalize(&bond)?,
    )?;

    Ok(Response::new().add_attributes(vec![
        attr("action", "add_subsidy_controller"),
        attr("subsidy_controller", subsidy_controller),
        attr("bond", bond),
    ]))
}

fn remove_subsidy_controller(deps: DepsMut, subsidy_controller: String) -> StdResult<Response> {
    remove_subsidy_controller_state(
        deps.storage,
        deps.api.addr_canonicalize(&subsidy_controller)?,
    );

    Ok(Response::new().add_attributes(vec![
        attr("action", "remove_subsidy_controller"),
        attr("subsidy_controller", subsidy_controller),
    ]))
}

fn update_config(deps: DepsMut, policy: Option<String>) -> StdResult<Response> {
    let mut config = read_config(deps.storage)?;

    if let Some(policy) = policy {
        config.policy = deps.api.addr_canonicalize(&policy)?;
    }

    store_config(deps.storage, &config)?;

    Ok(Response::new().add_attributes(vec![attr("action", "update_config")]))
}
