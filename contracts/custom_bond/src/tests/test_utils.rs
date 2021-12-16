use cosmwasm_std::testing::{mock_env, mock_info, MockApi, MockStorage};
use cosmwasm_std::{from_binary, to_binary, Decimal, Env, OwnedDeps, StdResult, Uint128};
use std::str::FromStr;

use crate::{
    contract::{execute, instantiate, query},
    tests::mock_querier::WasmMockQuerier,
};
use cw20::Cw20ReceiveMsg;
use olympus_pro::custom_bond::{
    BondInfo, BondInfoResponse, Cw20HookMsg, ExecuteMsg, InstantiateMsg, QueryMsg, Terms,
};
use terraswap::asset::AssetInfo;

pub fn instantiate_custom_bond(
    deps: &mut OwnedDeps<MockStorage, MockApi, WasmMockQuerier>,
    payout_decimals: Option<u8>,
    principal_decimals: Option<u8>,
) -> StdResult<InstantiateMsg> {
    let payout_decimals = if let Some(decimals) = payout_decimals {
        decimals
    } else {
        6u8
    };
    let principal_decimals = if let Some(decimals) = principal_decimals {
        decimals
    } else {
        6u8
    };
    deps.querier.with_token_info(
        &[(
            &String::from("payout_token"),
            &Uint128::from(1000000000000u128),
        )],
        &[
            (&String::from("principal_token"), &principal_decimals),
            (&String::from("payout_token"), &payout_decimals),
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
        fee_tiers: vec![],
        fee_in_payout: true,
    };

    let info = mock_info("policy", &[]);

    instantiate(deps.as_mut(), mock_env(), info, msg.clone()).unwrap();

    Ok(msg)
}

pub fn instantiate_custom_bond_with_principal_token(
    deps: &mut OwnedDeps<MockStorage, MockApi, WasmMockQuerier>,
    payout_decimals: Option<u8>,
    principal_token: AssetInfo,
) -> StdResult<InstantiateMsg> {
    let payout_decimals = if let Some(decimals) = payout_decimals {
        decimals
    } else {
        6u8
    };
    deps.querier.with_token_info(
        &[(
            &String::from("payout_token"),
            &Uint128::from(100000000000000u128),
        )],
        &[(&String::from("payout_token"), &payout_decimals)],
    );
    deps.querier.with_custom_treasury(
        String::from("custom_treasury"),
        String::from("payout_token"),
    );

    let msg = InstantiateMsg {
        custom_treasury: String::from("custom_treasury"),
        principal_token,
        olympus_treasury: String::from("olympus_treasury"),
        subsidy_router: String::from("subsidy_router"),
        initial_owner: String::from("policy"),
        olympus_dao: String::from("olympus_dao"),
        fee_tiers: vec![],
        fee_in_payout: true,
    };

    let info = mock_info("policy", &[]);

    instantiate(deps.as_mut(), mock_env(), info, msg.clone()).unwrap();

    Ok(msg)
}

pub fn initialize_bond(
    deps: &mut OwnedDeps<MockStorage, MockApi, WasmMockQuerier>,
    env: Env,
) -> (Terms, Uint128) {
    let info = mock_info("policy", &[]);

    let terms = Terms {
        control_variable: Decimal::from_ratio(1u128, 10u128),
        vesting_term: 864000,
        minimum_price: Decimal::from_str("0.157284").unwrap(),
        max_payout: Decimal::from_ratio(2u128, 100000u128),
        max_debt: Uint128::from(300000u128),
    };
    let initial_debt = Uint128::from(12500u128);
    let msg = ExecuteMsg::InitializeBond {
        terms: terms.clone(),
        initial_debt,
    };

    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    (terms, initial_debt)
}

pub fn deposit(deps: &mut OwnedDeps<MockStorage, MockApi, WasmMockQuerier>, env: Env) -> BondInfo {
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

    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let res = query(
        deps.as_ref(),
        env.clone(),
        QueryMsg::BondInfo {
            user: String::from("depositor"),
        },
    )
    .unwrap();
    let bond_info: BondInfoResponse = from_binary(&res).unwrap();

    bond_info.info
}

pub fn increase_time(env: &mut Env, addition: u64) {
    env.block.time = env.block.time.plus_seconds(addition);
}
