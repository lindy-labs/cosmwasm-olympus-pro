use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{attr, from_binary, Decimal, StdError, Uint128};
use std::str::FromStr;

use olympus_pro::custom_bond::{
    Adjustment, ConfigResponse, ExecuteMsg, FeeTier, InstantiateMsg, QueryMsg, State, Terms,
};
use terraswap::asset::AssetInfo;

use crate::{
    contract::{execute, instantiate, query},
    tests::{
        mock_querier::mock_dependencies,
        test_utils::{initialize_bond, instantiate_custom_bond},
    },
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

    instantiate_custom_bond(&mut deps, None, None).unwrap();

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

    instantiate_custom_bond(&mut deps, None, None).unwrap();

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

    instantiate_custom_bond(&mut deps, None, None).unwrap();

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

    instantiate_custom_bond(&mut deps, None, None).unwrap();

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

#[test]
fn test_initialize_bond_fails_if_unauthorized() {
    let mut deps = mock_dependencies(&[]);

    instantiate_custom_bond(&mut deps, None, None).unwrap();

    let info = mock_info("addr", &[]);
    let msg = ExecuteMsg::InitializeBond {
        terms: Terms {
            control_variable: Decimal::from_ratio(1u128, 10u128),
            vesting_term: 864000,
            minimum_price: Decimal::from_str("0.157284").unwrap(),
            max_payout: Decimal::from_ratio(1u128, 10000u128),
            max_debt: Uint128::from(1000000u128),
        },
        initial_debt: Uint128::from(100000u128),
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(res, StdError::generic_err("unauthorized"));
}

#[test]
fn test_initialize_bond_by_policy() {
    let mut deps = mock_dependencies(&[]);

    instantiate_custom_bond(&mut deps, None, None).unwrap();

    let info = mock_info("policy", &[]);
    let msg = ExecuteMsg::InitializeBond {
        terms: Terms {
            control_variable: Decimal::from_ratio(1u128, 10u128),
            vesting_term: 864000,
            minimum_price: Decimal::from_str("0.157284").unwrap(),
            max_payout: Decimal::from_ratio(1u128, 10000u128),
            max_debt: Uint128::from(1000000u128),
        },
        initial_debt: Uint128::from(100000u128),
    };

    let env = mock_env();
    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    assert_eq!(res.attributes, vec![attr("action", "initialize_bond"),]);

    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: State = from_binary(&res).unwrap();
    assert_eq!(
        State {
            total_debt: Uint128::from(100000u128),
            terms: Terms {
                control_variable: Decimal::from_ratio(1u128, 10u128),
                vesting_term: 864000,
                minimum_price: Decimal::from_str("0.157284").unwrap(),
                max_payout: Decimal::from_ratio(1u128, 10000u128),
                max_debt: Uint128::from(1000000u128),
            },
            adjustment: Adjustment::default(),
            last_decay: env.block.time.seconds(),
            payout_since_last_subsidy: Uint128::zero(),
            total_principal_bonded: Uint128::zero(),
            total_payout_given: Uint128::zero(),
        },
        state
    );
}

#[test]
fn test_initialize_bond_fails_if_current_debt_is_not_zero() {
    let mut deps = mock_dependencies(&[]);

    instantiate_custom_bond(&mut deps, None, None).unwrap();

    initialize_bond(&mut deps, mock_env());

    let info = mock_info("policy", &[]);
    let msg = ExecuteMsg::InitializeBond {
        terms: Terms {
            control_variable: Decimal::from_ratio(1u128, 10u128),
            vesting_term: 864000,
            minimum_price: Decimal::from_str("0.157284").unwrap(),
            max_payout: Decimal::from_ratio(1u128, 10000u128),
            max_debt: Uint128::from(1000000u128),
        },
        initial_debt: Uint128::from(100000u128),
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(
        res,
        StdError::generic_err("debt must be 0 for initialization")
    );
}

#[test]
fn test_initialize_bond_fails_if_vesting_term_is_less_than_36hours() {
    let mut deps = mock_dependencies(&[]);

    instantiate_custom_bond(&mut deps, None, None).unwrap();

    let info = mock_info("policy", &[]);
    let msg = ExecuteMsg::InitializeBond {
        terms: Terms {
            control_variable: Decimal::from_ratio(1u128, 10u128),
            vesting_term: 86400,
            minimum_price: Decimal::from_str("0.157284").unwrap(),
            max_payout: Decimal::from_ratio(1u128, 10000u128),
            max_debt: Uint128::from(1000000u128),
        },
        initial_debt: Uint128::from(100000u128),
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(
        res,
        StdError::generic_err("vesting must be longer than 36 hours")
    );
}

#[test]
fn test_initialize_bond_fails_if_max_payout_is_greater_or_euqal_than_1percent() {
    let mut deps = mock_dependencies(&[]);

    instantiate_custom_bond(&mut deps, None, None).unwrap();

    let info = mock_info("policy", &[]);
    let msg = ExecuteMsg::InitializeBond {
        terms: Terms {
            control_variable: Decimal::from_ratio(1u128, 10u128),
            vesting_term: 864000,
            minimum_price: Decimal::from_str("0.157284").unwrap(),
            max_payout: Decimal::percent(2),
            max_debt: Uint128::from(1000000u128),
        },
        initial_debt: Uint128::from(100000u128),
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(
        res,
        StdError::generic_err("payout cannot be above 1 percent")
    );
}

#[test]
fn test_set_bond_terms_fails_if_unauthorized() {
    let mut deps = mock_dependencies(&[]);

    instantiate_custom_bond(&mut deps, None, None).unwrap();

    let info = mock_info("addr", &[]);
    let msg = ExecuteMsg::SetBondTerms {
        vesting_term: Some(864000u64),
        max_payout: Some(Decimal::from_ratio(1u128, 10000u128)),
        max_debt: None,
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(res, StdError::generic_err("unauthorized"));
}

#[test]
fn test_send_bond_terms_by_policy_set_vesting_term() {
    let mut deps = mock_dependencies(&[]);

    instantiate_custom_bond(&mut deps, None, None).unwrap();

    let info = mock_info("policy", &[]);
    let msg = ExecuteMsg::SetBondTerms {
        vesting_term: Some(864000u64),
        max_payout: None,
        max_debt: None,
    };

    let env = mock_env();
    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    assert_eq!(res.attributes, vec![attr("action", "set_bond_terms"),]);

    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: State = from_binary(&res).unwrap();
    assert_eq!(864000u64, state.terms.vesting_term);
}

#[test]
fn test_send_bond_terms_fails_if_vesting_term_is_less_than_36hours() {
    let mut deps = mock_dependencies(&[]);

    instantiate_custom_bond(&mut deps, None, None).unwrap();

    let info = mock_info("policy", &[]);
    let msg = ExecuteMsg::SetBondTerms {
        vesting_term: Some(86400u64),
        max_payout: None,
        max_debt: None,
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(
        res,
        StdError::generic_err("vesting must be longer than 36 hours")
    );
}

#[test]
fn test_send_bond_terms_by_policy_set_max_payout() {
    let mut deps = mock_dependencies(&[]);

    instantiate_custom_bond(&mut deps, None, None).unwrap();

    let info = mock_info("policy", &[]);
    let msg = ExecuteMsg::SetBondTerms {
        vesting_term: None,
        max_payout: Some(Decimal::from_ratio(1u128, 1000u128)),
        max_debt: None,
    };

    let env = mock_env();
    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    assert_eq!(res.attributes, vec![attr("action", "set_bond_terms"),]);

    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: State = from_binary(&res).unwrap();
    assert_eq!(Decimal::from_ratio(1u128, 1000u128), state.terms.max_payout);
}

#[test]
fn test_send_bond_terms_fails_if_max_payout_is_greater_or_equal_than_1percent() {
    let mut deps = mock_dependencies(&[]);

    instantiate_custom_bond(&mut deps, None, None).unwrap();

    let info = mock_info("policy", &[]);
    let msg = ExecuteMsg::SetBondTerms {
        vesting_term: None,
        max_payout: Some(Decimal::percent(2)),
        max_debt: None,
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(
        res,
        StdError::generic_err("payout cannot be above 1 percent")
    );
}

#[test]
fn test_send_bond_terms_by_policy_set_max_debt() {
    let mut deps = mock_dependencies(&[]);

    instantiate_custom_bond(&mut deps, None, None).unwrap();

    let info = mock_info("policy", &[]);
    let msg = ExecuteMsg::SetBondTerms {
        vesting_term: None,
        max_payout: None,
        max_debt: Some(Uint128::from(10000u128)),
    };

    let env = mock_env();
    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    assert_eq!(res.attributes, vec![attr("action", "set_bond_terms"),]);

    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: State = from_binary(&res).unwrap();
    assert_eq!(Uint128::from(10000u128), state.terms.max_debt);
}

#[test]
fn test_set_adjustment_fails_if_unauthorized() {
    let mut deps = mock_dependencies(&[]);

    instantiate_custom_bond(&mut deps, None, None).unwrap();

    initialize_bond(&mut deps, mock_env());

    let info = mock_info("addr", &[]);
    let msg = ExecuteMsg::SetAdjustment {
        addition: true,
        increment: Decimal::from_str("0.0002").unwrap(),
        target: Decimal::from_str("0.176476").unwrap(),
        buffer: 86400u64,
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(res, StdError::generic_err("unauthorized"));
}

#[test]
fn test_set_adjustment_by_policy() {
    let mut deps = mock_dependencies(&[]);

    instantiate_custom_bond(&mut deps, None, None).unwrap();

    initialize_bond(&mut deps, mock_env());

    let info = mock_info("policy", &[]);
    let msg = ExecuteMsg::SetAdjustment {
        addition: true,
        increment: Decimal::from_str("0.0002").unwrap(),
        target: Decimal::from_str("0.176476").unwrap(),
        buffer: 86400u64,
    };

    let env = mock_env();
    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    assert_eq!(res.attributes, vec![attr("action", "set_adjustment"),]);

    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: State = from_binary(&res).unwrap();
    assert_eq!(
        Adjustment {
            addition: true,
            rate: Decimal::from_str("0.0002").unwrap(),
            target: Decimal::from_str("0.176476").unwrap(),
            buffer: 86400u64,
            last_time: env.block.time.seconds()
        },
        state.adjustment
    );
}

#[test]
fn test_set_adjustment_fails_if_increment_is_greater_than_30percent_of_control_variable() {
    let mut deps = mock_dependencies(&[]);

    instantiate_custom_bond(&mut deps, None, None).unwrap();

    initialize_bond(&mut deps, mock_env());

    let info = mock_info("policy", &[]);
    let msg = ExecuteMsg::SetAdjustment {
        addition: true,
        increment: Decimal::from_str("0.004").unwrap(),
        target: Decimal::from_str("0.176476").unwrap(),
        buffer: 86400u64,
    };

    let env = mock_env();
    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap_err();
    assert_eq!(res, StdError::generic_err("increment too large"));
}
