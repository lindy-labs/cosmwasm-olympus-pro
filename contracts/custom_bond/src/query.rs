use cosmwasm_std::{to_binary, Deps, QuerierWrapper, QueryRequest, StdResult, WasmQuery};

use olympus_pro::{
    custom_bond::{ConfigResponse, State},
    custom_treasury::{
        ConfigResponse as CustomTreasuryConfigResponse, QueryMsg as CustomTreasuryQueryMsg,
    },
};

use crate::state::{read_config, read_state};

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = read_config(deps.storage)?;

    let resp = ConfigResponse {
        custom_treasury: deps.api.addr_humanize(&config.custom_treasury)?.to_string(),
        payout_token: config.payout_token.to_normal(deps.api)?,
        principal_token: config.principal_token.to_normal(deps.api)?,
        olympus_treasury: deps
            .api
            .addr_humanize(&config.olympus_treasury)?
            .to_string(),
        subsidy_router: deps.api.addr_humanize(&config.subsidy_router)?.to_string(),
        policy: deps.api.addr_humanize(&config.policy)?.to_string(),
        olympus_dao: deps.api.addr_humanize(&config.olympus_dao)?.to_string(),
        tier_ceilings: config.tier_ceilings,
        fees: config.fees,
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
