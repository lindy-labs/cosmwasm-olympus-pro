use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{
    attr, from_binary, to_binary, BankMsg, Coin, CosmosMsg, Decimal, StdError, SubMsg, Uint128,
    WasmMsg,
};

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
            initialize_bond, instantiate_custom_bond, instantiate_custom_bond_with_principal_token,
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
    );

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
        max_price: Uint128::from(1000u128),
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
    );

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
        max_price: Uint128::from(1000u128),
        depositor: String::from("depositor"),
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(res, StdError::generic_err("invalid denom received"));
}

#[test]
fn test_deposit_fails_if_token_amount_is_zero() {
    let mut deps = mock_dependencies(&[]);

    instantiate_custom_bond(&mut deps, None, None);

    let env = mock_env();
    initialize_bond(&mut deps, env);

    let info = mock_info("principal_token", &[]);
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "addr".to_string(),
        msg: to_binary(&Cw20HookMsg::Deposit {
            max_price: Uint128::from(1000u128),
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

    instantiate_custom_bond(&mut deps, None, None);

    let env = mock_env();
    initialize_bond(&mut deps, env);

    let info = mock_info("invalid_token", &[]);
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "addr".to_string(),
        msg: to_binary(&Cw20HookMsg::Deposit {
            max_price: Uint128::from(1000u128),
            depositor: String::from("depositor"),
        })
        .unwrap(),
        amount: Uint128::from(100u128),
    });

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(res, StdError::generic_err("invalid cw20 token"));
}
