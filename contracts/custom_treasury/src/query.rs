use cosmwasm_std::{Decimal, Deps, StdResult, Uint128};

use olympus_pro::{custom_treasury::ConfigResponse, querier::query_decimals};
use terraswap::asset::Asset;

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
