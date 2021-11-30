use cosmwasm_std::{
    to_binary, Decimal, Deps, QuerierWrapper, QueryRequest, StdResult, Uint128, WasmQuery,
};

use cw20::{Cw20QueryMsg, TokenInfoResponse};
use olympus_pro::custom_treasury::ConfigResponse;
use terraswap::asset::{Asset, AssetInfo};

use crate::state::{read_bond_whitelist, read_config};

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = read_config(deps.storage)?;

    let resp = ConfigResponse {
        payout_token: config.payout_token.to_normal(deps.api)?,
        policy: deps.api.addr_humanize(&config.policy)?.to_string(),
    };

    Ok(resp)
}

pub fn query_bond_whitelist(deps: Deps, bond: String) -> StdResult<bool> {
    let whitelist = read_bond_whitelist(deps.storage, &deps.api.addr_canonicalize(&bond)?);

    Ok(whitelist.unwrap_or_default())
}

pub fn query_value_of_token(deps: Deps, principal_asset: Asset) -> StdResult<Uint128> {
    let config = read_config(deps.storage)?;

    let payout_decimals = query_decimals(&deps.querier, config.payout_token.to_normal(deps.api)?)?;
    let principal_decimals = query_decimals(&deps.querier, principal_asset.info)?;

    if payout_decimals == principal_decimals {
        Ok(principal_asset.amount)
    } else if payout_decimals > principal_decimals {
        Ok(principal_asset.amount
            * Uint128::from(
                10u128
                    .checked_pow((payout_decimals - principal_decimals) as u32)
                    .unwrap(),
            ))
    } else {
        Ok(principal_asset.amount
            * Decimal::from_ratio(
                1u128,
                10u128
                    .checked_pow((principal_decimals - payout_decimals) as u32)
                    .unwrap(),
            ))
    }
}

fn query_decimals(querier: &QuerierWrapper, asset: AssetInfo) -> StdResult<u8> {
    match asset {
        AssetInfo::NativeToken { .. } => Ok(6u8),
        AssetInfo::Token { contract_addr } => query_token_decimals(querier, contract_addr),
    }
}

fn query_token_decimals(querier: &QuerierWrapper, contract_addr: String) -> StdResult<u8> {
    let res: TokenInfoResponse = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: contract_addr.to_string(),
        msg: to_binary(&Cw20QueryMsg::TokenInfo {})?,
    }))?;

    Ok(res.decimals)
}
