#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult, Uint128,
};

use olympus_pro::custom_bond::{
    Adjustment, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, State, Terms,
};

use crate::{
    execute::{initialize_bond, pay_subsidy, set_adjustment, set_bond_terms, update_config},
    query::{query_config, query_custom_treasury_config, query_state},
    state::{read_config, store_config, store_state, Config},
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    if msg.tier_ceilings.len() != msg.fees.len() {
        return Err(StdError::generic_err(
            "tier length and fee length not the same",
        ));
    }

    let custom_treasury_config =
        query_custom_treasury_config(&deps.querier, msg.olympus_treasury.clone())?;

    store_config(
        deps.storage,
        &Config {
            custom_treasury: deps.api.addr_canonicalize(&msg.custom_treasury)?,
            payout_token: custom_treasury_config.payout_token.to_raw(deps.api)?,
            principal_token: msg.principal_token.to_raw(deps.api)?,
            olympus_treasury: deps.api.addr_canonicalize(&msg.olympus_treasury)?,
            subsidy_router: deps.api.addr_canonicalize(&msg.subsidy_router)?,
            policy: deps.api.addr_canonicalize(&msg.initial_owner)?,
            olympus_dao: deps.api.addr_canonicalize(&msg.olympus_dao)?,
            tier_ceilings: msg.tier_ceilings,
            fees: msg.fees,
            fee_in_payout: msg.fee_in_payout,
        },
    )?;

    store_state(
        deps.storage,
        &State {
            current_debt: Uint128::zero(),
            total_debt: Uint128::zero(),
            terms: Terms {
                control_variable: Uint128::zero(),
                vesting_term: 0u64,
                minimum_price: Uint128::zero(),
                max_payout: Uint128::zero(),
                max_debt: Uint128::zero(),
            },
            last_decay: 0u64,
            adjustment: Adjustment {
                addition: false,
                rate: Uint128::zero(),
                target: Uint128::zero(),
                buffer: Uint128::zero(),
                last_time: 0u64,
            },
            payout_since_last_subsidy: Uint128::zero(),
            total_principal_bonded: Uint128::zero(),
        },
    )?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Deposit {
            amount,
            max_price,
            depositor,
        } => Ok(Response::default()),
        ExecuteMsg::Redeem { depositor } => Ok(Response::default()),
        ExecuteMsg::PaySubsidy {} => pay_subsidy(deps, info),
        _ => {
            assert_policy_privilege(deps.as_ref(), info)?;
            match msg {
                ExecuteMsg::UpdateConfig {
                    policy,
                    olympus_treasury,
                } => update_config(deps, policy, olympus_treasury),
                ExecuteMsg::InitializeBond {
                    terms,
                    initial_debt,
                } => initialize_bond(deps, env, terms, initial_debt),
                ExecuteMsg::SetBondTerms {
                    vesting_term,
                    max_payout,
                    max_debt,
                } => set_bond_terms(deps, vesting_term, max_payout, max_debt),
                ExecuteMsg::SetAdjustment {
                    addition,
                    increment,
                    target,
                    buffer,
                } => set_adjustment(deps, env, addition, increment, target, buffer),
                _ => panic!("do not enter here"),
            }
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::State {} => to_binary(&query_state(deps)?),
        QueryMsg::BondPrice {} => to_binary(&"query_test"),
        QueryMsg::MaxPayout {} => to_binary(&"query_test"),
        QueryMsg::PayoutFor { .. } => to_binary(&"query_test"),
        QueryMsg::Debt {} => to_binary(&"query_test"),
        QueryMsg::PayoutInfo { .. } => to_binary(&"query_test"),
        QueryMsg::CurrentOlympusFee {} => to_binary(&"query_test"),
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
