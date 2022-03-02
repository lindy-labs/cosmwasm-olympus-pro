use cosmwasm_std::{Deps, StdResult};

use olympus_pro::custom_treasury::ConfigResponse;

use crate::state::{BOND_WHITELISTS, CONFIGURATION};

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIGURATION.load(deps.storage)?;

    let resp = ConfigResponse {
        payout_token: deps.api.addr_humanize(&config.payout_token)?.to_string(),
        policy: deps.api.addr_humanize(&config.policy)?.to_string(),
    };

    Ok(resp)
}

pub fn query_bond_whitelist(deps: Deps, bond: String) -> StdResult<bool> {
    let whitelist =
        BOND_WHITELISTS.load(deps.storage, deps.api.addr_canonicalize(&bond)?.as_slice());

    Ok(whitelist.unwrap_or_default())
}
