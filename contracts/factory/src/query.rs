use cosmwasm_std::{Deps, StdResult};

use crate::state::{read_bond_info, read_config, read_state, State};
use olympus_pro::factory::{BondInfoResponse, ConfigResponse};

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = read_config(deps.storage)?;

    let resp = ConfigResponse {
        custom_bond_id: config.custom_bond_id,
        custom_treasury_id: config.custom_treasury_id,
        treasury: deps.api.addr_humanize(&config.treasury)?.to_string(),
        subsidy_router: deps.api.addr_humanize(&config.subsidy_router)?.to_string(),
        olympus_dao: deps.api.addr_humanize(&config.olympus_dao)?.to_string(),
        policy: deps.api.addr_humanize(&config.policy)?.to_string(),
    };

    Ok(resp)
}

pub fn query_state(deps: Deps) -> StdResult<State> {
    let state = read_state(deps.storage)?;

    Ok(state)
}

pub fn query_bond_info(deps: Deps, bond_id: u64) -> StdResult<BondInfoResponse> {
    let bond_info = read_bond_info(deps.storage, bond_id)?;

    let resp = BondInfoResponse {
        principal_token: bond_info.principal_token.to_normal(deps.api)?,
        custom_treasury: deps
            .api
            .addr_humanize(&bond_info.custom_treasury)?
            .to_string(),
        bond: deps.api.addr_humanize(&bond_info.bond)?.to_string(),
        initial_owner: deps
            .api
            .addr_humanize(&bond_info.initial_owner)?
            .to_string(),
        fee_tiers: bond_info.fee_tiers,
    };

    Ok(resp)
}
