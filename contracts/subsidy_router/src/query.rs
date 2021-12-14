use cosmwasm_std::{Deps, StdResult};

use olympus_pro::subsidy_router::ConfigResponse;

use crate::state::{read_config, read_subsidy_controller};

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = read_config(deps.storage)?;

    let resp = ConfigResponse {
        policy: deps.api.addr_humanize(&config.policy)?.to_string(),
    };

    Ok(resp)
}

pub fn query_bond(deps: Deps, subsidy_controller: String) -> StdResult<String> {
    let bond = read_subsidy_controller(
        deps.storage,
        &deps.api.addr_canonicalize(&subsidy_controller)?,
    )?;

    Ok(deps.api.addr_humanize(&bond)?.to_string())
}
