use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{
    attr, from_binary, to_binary, BankMsg, Coin, CosmosMsg, Decimal, StdError, SubMsg, Uint128,
    WasmMsg,
};
use std::str::FromStr;

use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};
use olympus_pro::custom_bond::{
    Adjustment, ConfigResponse, Cw20HookMsg, ExecuteMsg, FeeTier, InstantiateMsg, QueryMsg, State,
    Terms,
};
use terraswap::asset::{Asset, AssetInfo};

use crate::{
    contract::{execute, instantiate, query},
    tests::{
        mock_querier::mock_dependencies,
        test_utils::{
            increase_time, initialize_bond, instantiate_custom_bond,
            instantiate_custom_bond_with_principal_token,
        },
    },
};

#[test]
fn test_deposit_fails_if_denom_amount_is_zero() {
    let mut deps = mock_dependencies(&[]);

    instantiate_custom_bond_with_principal_token(
        &mut deps,
        None,
        AssetInfo::NativeToken {
            denom: "uusd".to_string(),
        },
    )
    .unwrap();

    let env = mock_env();
    initialize_bond(&mut deps, env);

    let info = mock_info(
        "addr",
        &[Coin {
            denom: "uusd".to_string(),
            amount: Uint128::zero(),
        }],
    );
    let msg = ExecuteMsg::Deposit {
        max_price: Decimal::from_str("0.17476").unwrap(),
        depositor: String::from("depositor"),
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(res, StdError::generic_err("amount is zero"));
}

#[test]
fn test_deposit_fails_if_several_denom_received() {
    let mut deps = mock_dependencies(&[]);

    instantiate_custom_bond_with_principal_token(
        &mut deps,
        None,
        AssetInfo::NativeToken {
            denom: "uusd".to_string(),
        },
    )
    .unwrap();

    let env = mock_env();
    initialize_bond(&mut deps, env);

    let info = mock_info(
        "addr",
        &[
            Coin {
                denom: "uusd".to_string(),
                amount: Uint128::from(100u128),
            },
            Coin {
                denom: "ukrt".to_string(),
                amount: Uint128::from(100u128),
            },
        ],
    );
    let msg = ExecuteMsg::Deposit {
        max_price: Decimal::from_str("0.17476").unwrap(),
        depositor: String::from("depositor"),
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(res, StdError::generic_err("invalid denom received"));
}

#[test]
fn test_deposit_fails_if_token_amount_is_zero() {
    let mut deps = mock_dependencies(&[]);

    instantiate_custom_bond(&mut deps, None, None).unwrap();

    let env = mock_env();
    initialize_bond(&mut deps, env);

    let info = mock_info("principal_token", &[]);
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "addr".to_string(),
        msg: to_binary(&Cw20HookMsg::Deposit {
            max_price: Decimal::from_str("0.17476").unwrap(),
            depositor: String::from("depositor"),
        })
        .unwrap(),
        amount: Uint128::zero(),
    });

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(res, StdError::generic_err("amount is zero"));
}

#[test]
fn test_deposit_fails_if_received_token_is_invalid() {
    let mut deps = mock_dependencies(&[]);

    instantiate_custom_bond(&mut deps, None, None).unwrap();

    let env = mock_env();
    initialize_bond(&mut deps, env);

    let info = mock_info("invalid_token", &[]);
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "addr".to_string(),
        msg: to_binary(&Cw20HookMsg::Deposit {
            max_price: Decimal::from_str("0.17476").unwrap(),
            depositor: String::from("depositor"),
        })
        .unwrap(),
        amount: Uint128::from(100u128),
    });

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(res, StdError::generic_err("invalid cw20 token"));
}

#[test]
fn test_deposit_fails_if_true_bond_price_is_greater_than_max_price() {
    let mut deps = mock_dependencies(&[]);

    instantiate_custom_bond(&mut deps, None, None).unwrap();

    let mut env = mock_env();
    initialize_bond(&mut deps, env.clone());

    let time_increase = 100u64;
    increase_time(&mut env, time_increase);

    let info = mock_info("principal_token", &[]);
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "addr".to_string(),
        msg: to_binary(&Cw20HookMsg::Deposit {
            max_price: Decimal::from_str("0.17476").unwrap(),
            depositor: String::from("depositor"),
        })
        .unwrap(),
        amount: Uint128::from(100000000u128),
    });

    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap_err();
    assert_eq!(
        res,
        StdError::generic_err("slippage limit: more than max price")
    );
}

#[test]
fn test_deposit_fails_if_payout_is_too_small() {
    let mut deps = mock_dependencies(&[]);

    instantiate_custom_bond(&mut deps, None, None).unwrap();

    let mut env = mock_env();
    initialize_bond(&mut deps, env.clone());

    let time_increase = 100u64;
    increase_time(&mut env, time_increase);

    let info = mock_info("principal_token", &[]);
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "addr".to_string(),
        msg: to_binary(&Cw20HookMsg::Deposit {
            max_price: Decimal::from_str("0.17476").unwrap(),
            depositor: String::from("depositor"),
        })
        .unwrap(),
        amount: Uint128::from(100000u128),
    });

    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap_err();
    assert_eq!(res, StdError::generic_err("bond too small"));
}

#[test]
fn test_deposit_fails_if_payout_is_too_large() {
    let mut deps = mock_dependencies(&[]);

    instantiate_custom_bond(&mut deps, None, None).unwrap();

    let mut env = mock_env();
    initialize_bond(&mut deps, env.clone());

    let time_increase = 100u64;
    increase_time(&mut env, time_increase);

    let info = mock_info("principal_token", &[]);
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "addr".to_string(),
        msg: to_binary(&Cw20HookMsg::Deposit {
            max_price: Decimal::from_str("0.17476").unwrap(),
            depositor: String::from("depositor"),
        })
        .unwrap(),
        amount: Uint128::from(1000000000000u128),
    });

    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap_err();
    assert_eq!(res, StdError::generic_err("bond too large"));
}

#[test]
fn test_deposit_fails_if_max_payout_reached() {
    let mut deps = mock_dependencies(&[]);

    instantiate_custom_bond(&mut deps, None, None).unwrap();

    let mut env = mock_env();
    initialize_bond(&mut deps, env.clone());

    let time_increase = 100u64;
    increase_time(&mut env, time_increase);

    let info = mock_info("principal_token", &[]);
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "addr".to_string(),
        msg: to_binary(&Cw20HookMsg::Deposit {
            max_price: Decimal::from_str("0.17476").unwrap(),
            depositor: String::from("depositor"),
        })
        .unwrap(),
        amount: Uint128::from(10000000000u128),
    });

    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap_err();
    assert_eq!(res, StdError::generic_err("max capacity reached"));
}

#[test]
fn test_deposit() {
    let mut deps = mock_dependencies(&[]);

    let initialize_msg = instantiate_custom_bond(&mut deps, None, None).unwrap();

    let mut env = mock_env();
    let (terms, initial_debt) = initialize_bond(&mut deps, env.clone());

    let time_increase = 100u64;
    increase_time(&mut env, time_increase);
    let debt_decay =
        initial_debt * Decimal::from_ratio(time_increase as u128, terms.vesting_term as u128);

    let info = mock_info("principal_token", &[]);
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "addr".to_string(),
        msg: to_binary(&Cw20HookMsg::Deposit {
            max_price: Decimal::from_str("0.17476").unwrap(),
            depositor: String::from("depositor"),
        })
        .unwrap(),
        amount: Uint128::from(10000000u128),
    });

    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap_err();
    assert_eq!(res, StdError::generic_err("max capacity reached"));
}
