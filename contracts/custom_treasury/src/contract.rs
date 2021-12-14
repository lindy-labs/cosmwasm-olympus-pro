#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_std::{
    attr, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
    Uint128,
};

use olympus_pro::custom_treasury::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use terraswap::asset::{Asset, AssetInfo};

use crate::query::{query_bond_whitelist, query_config};
use crate::state::{read_bond_whitelist, read_config, store_bond_whitelist, store_config, Config};

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
            payout_token: deps.api.addr_canonicalize(&msg.payout_token)?,
            policy: deps.api.addr_canonicalize(&msg.initial_owner)?,
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
        ExecuteMsg::SendPayoutTokens { amount } => send_payout_token(deps, info, amount),
        _ => {
            assert_policy_privilege(deps.as_ref(), info)?;
            match msg {
                ExecuteMsg::UpdateConfig { policy } => update_config(deps, policy),
                ExecuteMsg::Withdraw { asset, recipient } => withdraw(deps, asset, recipient),
                ExecuteMsg::WhitelistBond { bond, whitelist } => {
                    whitelist_bond(deps, bond, whitelist)
                }
                _ => panic!("Do not enter here"),
            }
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::BondWhitelist { bond } => to_binary(&query_bond_whitelist(deps, bond)?),
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

fn update_config(deps: DepsMut, policy: Option<String>) -> StdResult<Response> {
    let mut config = read_config(deps.storage)?;

    if let Some(policy) = policy {
        config.policy = deps.api.addr_canonicalize(&policy)?;
    }

    store_config(deps.storage, &config)?;

    Ok(Response::new().add_attributes(vec![attr("action", "update_config")]))
}

fn send_payout_token(deps: DepsMut, info: MessageInfo, amount: Uint128) -> StdResult<Response> {
    let whitelist = read_bond_whitelist(
        deps.storage,
        &deps.api.addr_canonicalize(info.sender.as_str())?,
    )
    .unwrap_or_default();

    if whitelist {
        let config = read_config(deps.storage)?;

        let asset = Asset {
            info: AssetInfo::Token {
                contract_addr: deps.api.addr_humanize(&config.payout_token)?.to_string(),
            },
            amount: amount,
        };

        Ok(Response::new()
            .add_message(asset.clone().into_msg(&deps.querier, info.sender.clone())?)
            .add_attributes(vec![
                attr("action", "send_payout_token"),
                attr("amount", amount),
                attr("recipient", info.sender.to_string()),
            ]))
    } else {
        Err(StdError::generic_err("not whitelisted"))
    }
}

fn withdraw(deps: DepsMut, asset: Asset, recipient: String) -> StdResult<Response> {
    Ok(Response::new()
        .add_message(
            asset
                .clone()
                .into_msg(&deps.querier, Addr::unchecked(recipient.clone()))?,
        )
        .add_attributes(vec![
            attr("action", "withdraw"),
            attr("amount", asset.amount),
            attr("recipient", recipient),
        ]))
}

fn whitelist_bond(deps: DepsMut, bond: String, whitelist: bool) -> StdResult<Response> {
    store_bond_whitelist(
        deps.storage,
        &deps.api.addr_canonicalize(&bond)?,
        &whitelist,
    )?;

    Ok(Response::new().add_attributes(vec![
        attr("action", "whitelist_bond"),
        attr("bond", bond),
        attr("whitelist", whitelist.to_string()),
    ]))
}
