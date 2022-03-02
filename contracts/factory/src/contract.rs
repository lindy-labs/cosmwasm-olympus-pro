#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_std::{
    attr, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, ReplyOn, Response, StdError,
    StdResult, SubMsg, WasmMsg,
};

use olympus_pro::{
    custom_bond::{FeeTier, InstantiateMsg as CustomBondInstantiateMsg},
    custom_treasury::InstantiateMsg as CustomTreasuryInstantiateMsg,
    factory::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
};
use protobuf::Message;
use terraswap::asset::AssetInfo;

use crate::query::{query_bond_info, query_config, query_state};
use crate::response::MsgInstantiateContractResponse;
use crate::state::{
    BondInfo, Config, State, TempBondInfo, BOND_INFOS, CONFIGURATION, STATE, TEMP_BOND_INFO,
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    CONFIGURATION.save(
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

    STATE.save(deps.storage, &State { bond_length: 0 })?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    assert_policy_privilege(deps.as_ref(), info)?;
    match msg {
        ExecuteMsg::UpdateConfig {
            custom_bond_id,
            custom_treasury_id,
            policy,
        } => update_config(deps, custom_bond_id, custom_treasury_id, policy),
        ExecuteMsg::CreateBondAndTreasury {
            payout_token,
            principal_token,
            initial_owner,
            fee_tiers,
            fee_in_payout,
        } => create_bond_and_treasury(
            deps,
            env,
            payout_token,
            principal_token,
            initial_owner,
            fee_tiers,
            fee_in_payout,
        ),
        ExecuteMsg::CreateBond {
            principal_token,
            custom_treasury,
            initial_owner,
            fee_tiers,
            fee_in_payout,
        } => create_bond(
            deps,
            env,
            principal_token,
            custom_treasury,
            initial_owner,
            fee_tiers,
            fee_in_payout,
        ),
    }
}

/// This just stores the result for future query
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> StdResult<Response> {
    match msg.id {
        1 => {
            let res: MsgInstantiateContractResponse = Message::parse_from_bytes(
                msg.result.unwrap().data.unwrap().as_slice(),
            )
            .map_err(|_| {
                StdError::parse_err("MsgInstantiateContractResponse", "failed to parse data")
            })?;
            let treasury_addr = res.get_contract_address();

            create_bond_from_temp(deps, env, treasury_addr.to_string())
        }
        2 => {
            let res: MsgInstantiateContractResponse = Message::parse_from_bytes(
                msg.result.unwrap().data.unwrap().as_slice(),
            )
            .map_err(|_| {
                StdError::parse_err("MsgInstantiateContractResponse", "failed to parse data")
            })?;
            let bond_addr = res.get_contract_address();

            register_bond(deps, bond_addr.to_string())
        }
        _ => Err(StdError::generic_err("invalid reply id")),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::State {} => to_binary(&query_state(deps)?),
        QueryMsg::BondInfo { bond_id } => to_binary(&query_bond_info(deps, bond_id)?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}

fn assert_policy_privilege(deps: Deps, info: MessageInfo) -> StdResult<()> {
    if CONFIGURATION.load(deps.storage)?.policy
        != deps.api.addr_canonicalize(info.sender.as_str())?
    {
        return Err(StdError::generic_err("unauthorized"));
    }

    Ok(())
}

fn update_config(
    deps: DepsMut,
    custom_bond_id: Option<u64>,
    custom_treasury_id: Option<u64>,
    policy: Option<String>,
) -> StdResult<Response> {
    let mut config = CONFIGURATION.load(deps.storage)?;

    if let Some(custom_bond_id) = custom_bond_id {
        config.custom_bond_id = custom_bond_id;
    }

    if let Some(custom_treasury_id) = custom_treasury_id {
        config.custom_treasury_id = custom_treasury_id;
    }

    if let Some(policy) = policy {
        config.policy = deps.api.addr_canonicalize(&policy)?;
    }

    CONFIGURATION.save(deps.storage, &config)?;

    Ok(Response::new().add_attributes(vec![attr("action", "update_config")]))
}

fn create_bond_and_treasury(
    deps: DepsMut,
    env: Env,
    payout_token: String,
    principal_token: AssetInfo,
    initial_owner: String,
    fee_tiers: Vec<FeeTier>,
    fee_in_payout: bool,
) -> StdResult<Response> {
    let config = CONFIGURATION.load(deps.storage)?;

    TEMP_BOND_INFO.save(
        deps.storage,
        &TempBondInfo {
            principal_token: principal_token.to_raw(deps.api)?,
            custom_treasury: None,
            initial_owner: deps.api.addr_canonicalize(&initial_owner)?,
            fee_tiers: fee_tiers.clone(),
            fee_in_payout,
        },
    )?;

    Ok(Response::new()
        .add_attributes(vec![("action", "create_treasury")])
        .add_submessage(SubMsg {
            id: 1,
            gas_limit: None,
            msg: WasmMsg::Instantiate {
                code_id: config.custom_treasury_id,
                funds: vec![],
                admin: Some(env.contract.address.to_string()),
                label: "OlympusPro Custom Treasury".to_string(),
                msg: to_binary(&CustomTreasuryInstantiateMsg {
                    payout_token,
                    initial_owner,
                })?,
            }
            .into(),
            reply_on: ReplyOn::Success,
        }))
}

fn create_bond_from_temp(deps: DepsMut, env: Env, custom_treasury: String) -> StdResult<Response> {
    let mut temp_bond_info = TEMP_BOND_INFO.load(deps.storage)?;

    temp_bond_info.custom_treasury = Some(deps.api.addr_canonicalize(&custom_treasury)?);

    TEMP_BOND_INFO.save(deps.storage, &temp_bond_info)?;

    let config = CONFIGURATION.load(deps.storage)?;

    Ok(Response::new()
        .add_attributes(vec![("action", "create_bond")])
        .add_submessage(SubMsg {
            id: 2,
            gas_limit: None,
            msg: WasmMsg::Instantiate {
                code_id: config.custom_bond_id,
                funds: vec![],
                admin: Some(env.contract.address.to_string()),
                label: "OlympusPro Custom Bond".to_string(),
                msg: to_binary(&CustomBondInstantiateMsg {
                    custom_treasury: custom_treasury.clone(),
                    principal_token: temp_bond_info.principal_token.to_normal(deps.api)?,
                    olympus_treasury: deps.api.addr_humanize(&config.treasury)?.to_string(),
                    subsidy_router: deps.api.addr_humanize(&config.subsidy_router)?.to_string(),
                    initial_owner: deps
                        .api
                        .addr_humanize(&temp_bond_info.initial_owner)?
                        .to_string(),
                    olympus_dao: deps.api.addr_humanize(&config.olympus_dao)?.to_string(),
                    fee_tiers: temp_bond_info.fee_tiers,
                    fee_in_payout: temp_bond_info.fee_in_payout,
                })?,
            }
            .into(),
            reply_on: ReplyOn::Success,
        }))
}

fn create_bond(
    deps: DepsMut,
    env: Env,
    principal_token: AssetInfo,
    custom_treasury: String,
    initial_owner: String,
    fee_tiers: Vec<FeeTier>,
    fee_in_payout: bool,
) -> StdResult<Response> {
    let config = CONFIGURATION.load(deps.storage)?;

    TEMP_BOND_INFO.save(
        deps.storage,
        &TempBondInfo {
            principal_token: principal_token.to_raw(deps.api)?,
            custom_treasury: Some(deps.api.addr_canonicalize(&custom_treasury)?),
            initial_owner: deps.api.addr_canonicalize(&initial_owner)?,
            fee_tiers: fee_tiers.clone(),
            fee_in_payout,
        },
    )?;

    Ok(Response::new()
        .add_attributes(vec![("action", "create_bond")])
        .add_submessage(SubMsg {
            id: 2,
            gas_limit: None,
            msg: WasmMsg::Instantiate {
                code_id: config.custom_bond_id,
                funds: vec![],
                admin: Some(env.contract.address.to_string()),
                label: "OlympusPro Custom Bond".to_string(),
                msg: to_binary(&CustomBondInstantiateMsg {
                    custom_treasury: custom_treasury.clone(),
                    principal_token,
                    olympus_treasury: custom_treasury,
                    subsidy_router: deps.api.addr_humanize(&config.subsidy_router)?.to_string(),
                    initial_owner,
                    olympus_dao: deps.api.addr_humanize(&config.olympus_dao)?.to_string(),
                    fee_tiers,
                    fee_in_payout,
                })?,
            }
            .into(),
            reply_on: ReplyOn::Success,
        }))
}

fn register_bond(deps: DepsMut, bond: String) -> StdResult<Response> {
    let temp_bond_info = TEMP_BOND_INFO.load(deps.storage)?;

    let mut state = STATE.load(deps.storage)?;

    BOND_INFOS.save(
        deps.storage,
        &state.bond_length.to_be_bytes(),
        &BondInfo {
            principal_token: temp_bond_info.principal_token,
            custom_treasury: temp_bond_info.custom_treasury.unwrap(),
            bond: deps.api.addr_canonicalize(&bond)?,
            initial_owner: temp_bond_info.initial_owner,
            fee_tiers: temp_bond_info.fee_tiers,
        },
    )?;

    state.bond_length += 1;

    STATE.save(deps.storage, &state)?;

    TEMP_BOND_INFO.remove(deps.storage);

    Ok(Response::default())
}
