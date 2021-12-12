use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{
    attr, from_binary, to_binary, BankMsg, Coin, CosmosMsg, Decimal, StdError, SubMsg, Uint128,
    WasmMsg,
};

use cw20::Cw20ExecuteMsg;
use olympus_pro::custom_bond::{
    ConfigResponse, ExecuteMsg, FeeTier, InstantiateMsg, QueryMsg, State,
};
use terraswap::asset::{Asset, AssetInfo};

use crate::{
    contract::{execute, instantiate, query},
    tests::{mock_querier::mock_dependencies, test_utils::instantiate_custom_bond},
};

#[test]
fn test_initialization() {
    let mut deps = mock_dependencies(&[]);

    deps.querier.with_token_info(
        &[],
        &[
            (&String::from("principal_token"), &8u8),
            (&String::from("payout_token"), &9u8),
        ],
    );
    deps.querier.with_custom_treasury(
        String::from("custom_treasury"),
        String::from("payout_token"),
    );

    let msg = InstantiateMsg {
        custom_treasury: String::from("custom_treasury"),
        principal_token: AssetInfo::Token {
            contract_addr: String::from("principal_token"),
        },
        olympus_treasury: String::from("olympus_treasury"),
        subsidy_router: String::from("subsidy_router"),
        initial_owner: String::from("policy"),
        olympus_dao: String::from("olympus_dao"),
        fee_tiers: vec![
            FeeTier {
                tier_ceiling: Uint128::from(100000u128),
                fee_rate: Decimal::from_ratio(1u128, 1000u128),
            },
            FeeTier {
                tier_ceiling: Uint128::from(200000u128),
                fee_rate: Decimal::from_ratio(2u128, 1000u128),
            },
        ],
        fee_in_payout: true,
    };

    let info = mock_info("policy", &[]);

    // we can just call .unwrap() to assert this was a success
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // it worked, let's query the config and state
    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!(
        ConfigResponse {
            custom_treasury: String::from("custom_treasury"),
            payout_token: String::from("payout_token"),
            principal_token: AssetInfo::Token {
                contract_addr: String::from("principal_token"),
            },
            olympus_treasury: String::from("olympus_treasury"),
            subsidy_router: String::from("subsidy_router"),
            policy: String::from("policy"),
            olympus_dao: String::from("olympus_dao"),
            fee_tiers: vec![
                FeeTier {
                    tier_ceiling: Uint128::from(100000u128),
                    fee_rate: Decimal::from_ratio(1u128, 1000u128),
                },
                FeeTier {
                    tier_ceiling: Uint128::from(200000u128),
                    fee_rate: Decimal::from_ratio(2u128, 1000u128),
                },
            ],
            fee_in_payout: true,
        },
        config
    );

    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: State = from_binary(&res).unwrap();
    assert_eq!(State::default(), state);
}

#[test]
fn test_update_policy_fails_if_unauthorized() {
    let mut deps = mock_dependencies(&[]);

    instantiate_custom_bond(&mut deps, None, None);

    let info = mock_info("addr", &[]);
    let msg = ExecuteMsg::UpdatePolicy {
        policy: String::from("new_policy"),
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(res, StdError::generic_err("unauthorized"));
}

#[test]
fn test_update_policy_by_policy() {
    let mut deps = mock_dependencies(&[]);

    instantiate_custom_bond(&mut deps, None, None);

    let info = mock_info("policy", &[]);
    let msg = ExecuteMsg::UpdatePolicy {
        policy: String::from("new_policy"),
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(
        res.attributes,
        vec![
            attr("action", "update_policy"),
            attr("policy", "new_policy")
        ]
    );

    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!(String::from("new_policy"), config.policy);
}

#[test]
fn test_update_olympus_treasury_fails_if_unauthorized() {
    let mut deps = mock_dependencies(&[]);

    instantiate_custom_bond(&mut deps, None, None);

    let info = mock_info("addr", &[]);
    let msg = ExecuteMsg::UpdateOlympusTreasury {
        olympus_treasury: String::from("new_olympus_treasury"),
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(res, StdError::generic_err("unauthorized"));
}

#[test]
fn test_update_olympus_treasury_by_olympus_dao() {
    let mut deps = mock_dependencies(&[]);

    instantiate_custom_bond(&mut deps, None, None);

    let info = mock_info("olympus_dao", &[]);
    let msg = ExecuteMsg::UpdateOlympusTreasury {
        olympus_treasury: String::from("new_olympus_treasury"),
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(
        res.attributes,
        vec![
            attr("action", "update_olympus_treasury"),
            attr("olympus_treasury", "new_olympus_treasury")
        ]
    );

    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!(
        String::from("new_olympus_treasury"),
        config.olympus_treasury
    );
}
