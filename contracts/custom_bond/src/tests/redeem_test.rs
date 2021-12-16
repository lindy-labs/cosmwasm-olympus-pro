use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{
    attr, from_binary, to_binary, Coin, CosmosMsg, Decimal, Fraction, StdError, SubMsg, Uint128,
    WasmMsg,
};
use std::str::FromStr;

use cw20::Cw20ExecuteMsg;
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
            deposit, increase_time, initialize_bond, instantiate_custom_bond,
            instantiate_custom_bond_with_principal_token,
        },
    },
};

#[test]
fn test_redeem_fails_if_no_pending_payout() {
    let mut deps = mock_dependencies(&[]);

    instantiate_custom_bond(&mut deps, None, None).unwrap();

    let mut env = mock_env();
    initialize_bond(&mut deps, env.clone());

    let time_increase = 100u64;
    increase_time(&mut env, time_increase);

    deposit(&mut deps, env.clone());

    let info = mock_info("depositor", &[]);
    let msg = ExecuteMsg::Redeem {};

    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap_err();
    assert_eq!(res, StdError::generic_err("nothing to redeem"));
}

#[test]
fn test_redeem_some() {
    let mut deps = mock_dependencies(&[]);

    instantiate_custom_bond(&mut deps, None, None).unwrap();

    let mut env = mock_env();
    let (terms, _) = initialize_bond(&mut deps, env.clone());

    let time_increase = 100u64;

    increase_time(&mut env, time_increase);

    let bond_info = deposit(&mut deps, env.clone());

    let info = mock_info("depositor", &[]);
    let msg = ExecuteMsg::Redeem {};

    let time_increase = 10000;
    increase_time(&mut env, time_increase);

    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let payout = bond_info.payout * Decimal::from_ratio(time_increase as u128, bond_info.vesting);

    assert_eq!(
        res.attributes,
        vec![attr("action", "redeem"), attr("amount", payout.to_string()),]
    );

    assert_eq!(
        res.messages,
        vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "payout_token".to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: String::from("depositor"),
                amount: payout,
            })
            .unwrap(),
            funds: vec![],
        }))]
    );

    let res = query(
        deps.as_ref(),
        env.clone(),
        QueryMsg::BondInfo {
            user: String::from("depositor"),
        },
    )
    .unwrap();

    let current_bond_info: BondInfoResponse = from_binary(&res).unwrap();
    assert_eq!(
        BondInfoResponse {
            info: BondInfo {
                payout: bond_info.payout - payout,
                vesting: bond_info.vesting - time_increase,
                last_time: env.block.time.seconds(),
                true_price_paid: terms.minimum_price,
            },
            pending_payout: Uint128::zero()
        },
        current_bond_info
    );
}

#[test]
fn test_redeem_all() {
    let mut deps = mock_dependencies(&[]);

    instantiate_custom_bond(&mut deps, None, None).unwrap();

    let mut env = mock_env();
    let (terms, _) = initialize_bond(&mut deps, env.clone());

    let time_increase = 100u64;

    increase_time(&mut env, time_increase);

    let bond_info = deposit(&mut deps, env.clone());

    let info = mock_info("depositor", &[]);
    let msg = ExecuteMsg::Redeem {};

    increase_time(&mut env, terms.vesting_term);

    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    assert_eq!(
        res.attributes,
        vec![
            attr("action", "redeem"),
            attr("amount", bond_info.payout.to_string()),
        ]
    );

    assert_eq!(
        res.messages,
        vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "payout_token".to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: String::from("depositor"),
                amount: bond_info.payout,
            })
            .unwrap(),
            funds: vec![],
        }))]
    );

    query(
        deps.as_ref(),
        env.clone(),
        QueryMsg::BondInfo {
            user: String::from("depositor"),
        },
    )
    .unwrap_err();
}
