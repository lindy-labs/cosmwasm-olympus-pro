use cosmwasm_std::testing::{mock_env, mock_info, MockApi, MockStorage};
use cosmwasm_std::{Decimal, Env, OwnedDeps, Uint128};

use crate::{
    contract::{execute, instantiate},
    tests::mock_querier::WasmMockQuerier,
};
use olympus_pro::custom_bond::{ExecuteMsg, InstantiateMsg, Terms};
use terraswap::asset::AssetInfo;

pub fn instantiate_custom_bond(
    deps: &mut OwnedDeps<MockStorage, MockApi, WasmMockQuerier>,
    payout_decimals: Option<u8>,
    principal_decimals: Option<u8>,
) {
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
        &[],
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

    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
}

pub fn initialize_bond(deps: &mut OwnedDeps<MockStorage, MockApi, WasmMockQuerier>, env: Env) {
    let info = mock_info("policy", &[]);
    let msg = ExecuteMsg::InitializeBond {
        terms: Terms {
            control_variable: Uint128::from(1000u128),
            vesting_term: 864000,
            minimum_price: Uint128::from(10000u128),
            max_payout: Decimal::from_ratio(1u128, 10000u128),
            max_debt: Uint128::from(1000000u128),
        },
        initial_debt: Uint128::from(100000u128),
    };

    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
}
