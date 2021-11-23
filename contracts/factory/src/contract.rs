#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_std::{
    attr, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, ReplyOn, Response, StdError,
    StdResult, SubMsg, WasmMsg,
};

use olympus_pro::custom_bond::InstantiateMsg as CustomBondInstantiateMsg;
use olympus_pro::custom_treasury::InstantiateMsg as CustomTreasuryInstantiateMsg;
use olympus_pro::factory::{ConfigResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use protobuf::Message;
use terraswap::asset::AssetInfo;

use crate::response::MsgInstantiateContractResponse;
use crate::state::{read_config, store_config, Config};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    store_config(
        deps.storage,
        &Config {
            custom_bond_id: msg.custom_bond_id,
            custom_treasury_id: msg.custom_treasury_id,
            treasury: deps.api.addr_canonicalize(&msg.treasury)?,
            subsidy_router: deps.api.addr_canonicalize(&msg.subsidy_router)?,
            olympus_dao: deps.api.addr_canonicalize(&msg.olympus_dao)?,
            policy: deps.api.addr_canonicalize(&info.sender.as_str())?,
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
        ExecuteMsg::UpdateConfig {
            custom_bond_id,
            custom_treasury_id,
            policy,
        } => update_config(deps, info, custom_bond_id, custom_treasury_id, policy),
        ExecuteMsg::CreateTreasury {
            payout_token,
            initial_owner,
        } => create_treasury(deps, info, payout_token, initial_owner),
        ExecuteMsg::CreateBond {
            principal_token,
            custom_treasury,
            initial_owner,
            tier_ceilings,
            fees,
            fee_in_payout,
        } => create_bond(
            deps,
            info,
            principal_token,
            custom_treasury,
            initial_owner,
            tier_ceilings,
            fees,
            fee_in_payout,
        ),
    }
}

/// This just stores the result for future query
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, _msg: Reply) -> StdResult<Response> {
    // TODO store data
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
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

fn update_config(
    deps: DepsMut,
    info: MessageInfo,
    custom_bond_id: Option<u64>,
    custom_treasury_id: Option<u64>,
    policy: Option<String>,
) -> StdResult<Response> {
    assert_policy_privilege(deps.as_ref(), info)?;

    let mut config = read_config(deps.storage)?;

    if let Some(custom_bond_id) = custom_bond_id {
        config.custom_bond_id = custom_bond_id;
    }

    if let Some(custom_treasury_id) = custom_treasury_id {
        config.custom_treasury_id = custom_treasury_id;
    }

    if let Some(policy) = policy {
        config.policy = deps.api.addr_canonicalize(&policy)?;
    }

    store_config(deps.storage, &config)?;

    Ok(Response::new().add_attributes(vec![attr("action", "update_config")]))
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = read_config(deps.storage)?;

    let resp = ConfigResponse {
        custom_bond_id: config.custom_bond_id,
        custom_treasury_id: config.custom_bond_id,
        treasury: deps.api.addr_humanize(&config.treasury)?.to_string(),
        subsidy_router: deps.api.addr_humanize(&config.subsidy_router)?.to_string(),
        olympus_dao: deps.api.addr_humanize(&config.olympus_dao)?.to_string(),
        policy: deps.api.addr_humanize(&config.policy)?.to_string(),
    };

    Ok(resp)
}

fn create_treasury(
    deps: DepsMut,
    info: MessageInfo,
    payout_token: AssetInfo,
    initial_owner: String,
) -> StdResult<Response> {
    assert_policy_privilege(deps.as_ref(), info)?;

    let config = read_config(deps.storage)?;

    Ok(Response::new()
        .add_attributes(vec![("action", "create_treasury")])
        .add_submessage(SubMsg {
            id: 1,
            gas_limit: None,
            msg: WasmMsg::Instantiate {
                code_id: config.custom_treasury_id,
                funds: vec![],
                admin: None,
                label: "".to_string(),
                msg: to_binary(&CustomTreasuryInstantiateMsg {
                    payout_token,
                    initial_owner,
                })?,
            }
            .into(),
            reply_on: ReplyOn::Success,
        }))
}

fn create_bond(
    deps: DepsMut,
    info: MessageInfo,
    principal_token: AssetInfo,
    custom_treasury: String,
    initial_owner: String,
    tier_ceilings: Vec<u64>,
    fees: Vec<u64>,
    fee_in_payout: bool,
) -> StdResult<Response> {
    assert_policy_privilege(deps.as_ref(), info)?;

    let config = read_config(deps.storage)?;

    Ok(Response::new()
        .add_attributes(vec![("action", "create_bond")])
        .add_submessage(SubMsg {
            id: 1,
            gas_limit: None,
            msg: WasmMsg::Instantiate {
                code_id: config.custom_bond_id,
                funds: vec![],
                admin: None,
                label: "".to_string(),
                msg: to_binary(&CustomBondInstantiateMsg {
                    custom_treasury: custom_treasury.clone(),
                    principal_token,
                    olympus_treasury: custom_treasury,
                    subsidy_router: deps.api.addr_humanize(&config.subsidy_router)?.to_string(),
                    initial_owner,
                    tier_ceilings,
                    fees,
                    fee_in_payout,
                })?,
            }
            .into(),
            reply_on: ReplyOn::Success,
        }))
}
