use cosmwasm_std::testing::{mock_env, mock_info, MockApi, MockStorage};
use cosmwasm_std::OwnedDeps;

use olympus_pro::custom_treasury::InstantiateMsg;
use terraswap::asset::AssetInfo;

use crate::{contract::instantiate, tests::mock_querier::WasmMockQuerier};

pub fn instantiate_custom_treasury(deps: &mut OwnedDeps<MockStorage, MockApi, WasmMockQuerier>) {
    let msg = InstantiateMsg {
        payout_token: AssetInfo::NativeToken {
            denom: "upayout".to_string(),
        },
        initial_owner: String::from("policy"),
    };

    let info = mock_info("policy", &[]);

    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
}
