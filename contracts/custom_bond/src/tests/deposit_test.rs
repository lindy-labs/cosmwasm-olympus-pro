use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{
    attr, from_binary, to_binary, Coin, CosmosMsg, Decimal, Fraction, StdError, SubMsg, Uint128,
    WasmMsg,
};
use std::str::FromStr;

use cw20::Cw20ReceiveMsg;
use olympus_pro::{
    custom_bond::{
        Adjustment, BondInfo, BondInfoResponse, Cw20HookMsg, ExecuteMsg, QueryMsg, State,
    },
    custom_treasury::ExecuteMsg as CustomTreasuryExecuteMsg,
};
use terraswap::asset::AssetInfo;

use crate::{
    contract::{execute, query},
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
            max_price: Decimal::from_str("0.14").unwrap(),
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
        amount: Uint128::from(1000u128),
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
        amount: Uint128::from(4000000u128),
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
        amount: Uint128::from(1000000u128),
    });

    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap_err();
    assert_eq!(res, StdError::generic_err("max capacity reached"));
}

#[test]
fn test_first_deposit() {
    let mut deps = mock_dependencies(&[]);

    instantiate_custom_bond(&mut deps, None, None).unwrap();

    let mut env = mock_env();
    let (terms, initial_debt) = initialize_bond(&mut deps, env.clone());

    let time_increase = 100u64;

    increase_time(&mut env, time_increase);

    let debt_decay =
        initial_debt * Decimal::from_ratio(time_increase as u128, terms.vesting_term as u128);

    let amount = Uint128::from(100000u128);
    let info = mock_info("principal_token", &[]);
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "addr".to_string(),
        msg: to_binary(&Cw20HookMsg::Deposit {
            max_price: Decimal::from_str("0.17476").unwrap(),
            depositor: String::from("depositor"),
        })
        .unwrap(),
        amount,
    });

    let total_supply = Uint128::from(1000000000000u128);
    let total_debt = initial_debt - debt_decay + amount;
    let debt_ratio = Decimal::from_ratio(total_debt, total_supply);
    let payout = amount * terms.minimum_price.inv().unwrap();

    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    assert_eq!(
        res.attributes,
        vec![
            attr("action", "deposit"),
            attr("amount", amount.to_string()),
            attr("payout", payout.to_string()),
            attr(
                "expires",
                (env.block.time.seconds() + terms.vesting_term).to_string()
            ),
            attr("bond_price", terms.minimum_price.to_string()),
            attr("debt_ratio", debt_ratio.to_string())
        ]
    );

    assert_eq!(
        res.messages,
        vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: String::from("custom_treasury"),
            funds: vec![],
            msg: to_binary(&CustomTreasuryExecuteMsg::SendPayoutTokens { amount: payout }).unwrap(),
        })),]
    );

    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: State = from_binary(&res).unwrap();
    assert_eq!(
        State {
            total_debt,
            terms: terms.clone(),
            adjustment: Adjustment::default(),
            last_decay: env.block.time.seconds(),
            payout_since_last_subsidy: payout,
            total_principal_bonded: amount,
            total_payout_given: payout,
        },
        state
    );

    let res = query(
        deps.as_ref(),
        env.clone(),
        QueryMsg::BondInfo {
            user: String::from("depositor"),
        },
    )
    .unwrap();
    let bond_info: BondInfoResponse = from_binary(&res).unwrap();
    assert_eq!(
        BondInfo {
            payout,
            vesting: terms.vesting_term,
            last_time: env.block.time.seconds(),
            true_price_paid: terms.minimum_price,
        },
        bond_info.info
    );
}
