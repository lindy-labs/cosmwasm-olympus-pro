use cosmwasm_std::testing::{mock_env, mock_info, MockApi, MockStorage};
use cosmwasm_std::OwnedDeps;

use crate::{contract::instantiate, tests::mock_querier::WasmMockQuerier};
use olympus_pro::custom_bond::InstantiateMsg;
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
