use cosmwasm_std::{
    to_binary, Binary, QuerierWrapper, QueryRequest, StdResult, Uint128, WasmQuery,
};

use cw20::{Cw20QueryMsg, TokenInfoResponse};
use protobuf::Message;
use terraswap::asset::AssetInfo;

pub fn query_decimals(querier: &QuerierWrapper, asset: &AssetInfo) -> StdResult<u8> {
    match asset {
        AssetInfo::NativeToken { .. } => Ok(6u8),
        AssetInfo::Token { contract_addr } => {
            query_token_decimals(querier, contract_addr.to_string())
        }
    }
}

pub fn query_total_supply(querier: &QuerierWrapper, asset: &AssetInfo) -> StdResult<Uint128> {
    match asset {
        AssetInfo::NativeToken { denom } => query_denom_supply(querier, denom.to_string()),
        AssetInfo::Token { contract_addr } => {
            query_token_supply(querier, contract_addr.to_string())
        }
    }
}

pub fn query_denom_supply(querier: &QuerierWrapper, denom: String) -> StdResult<Uint128> {
    // let res = querier.query(&QueryRequest::Stargate {
    //     path: "/cosmos.bank.v1beta1.Query/TotalSupply".to_string(),
    //     data: Binary::from(vec![]),
    // })?;
    let res: TokenInfoResponse = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: denom.to_string(),
        msg: to_binary(&Cw20QueryMsg::TokenInfo {})?,
    }))?;

    Ok(Uint128::from(res.total_supply.u128()))
}

fn query_token_supply(querier: &QuerierWrapper, contract_addr: String) -> StdResult<Uint128> {
    let res: TokenInfoResponse = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: contract_addr.to_string(),
        msg: to_binary(&Cw20QueryMsg::TokenInfo {})?,
    }))?;

    Ok(Uint128::from(res.total_supply.u128()))
}

pub fn query_token_decimals(querier: &QuerierWrapper, contract_addr: String) -> StdResult<u8> {
    let res: TokenInfoResponse = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: contract_addr.to_string(),
        msg: to_binary(&Cw20QueryMsg::TokenInfo {})?,
    }))?;

    Ok(res.decimals)
}
