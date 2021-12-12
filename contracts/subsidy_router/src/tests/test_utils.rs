use cosmwasm_std::testing::{mock_env, mock_info, MockApi, MockQuerier, MockStorage};
use cosmwasm_std::OwnedDeps;

use olympus_pro::subsidy_router::InstantiateMsg;

use crate::contract::instantiate;

pub fn instantiate_subsidy_router(deps: &mut OwnedDeps<MockStorage, MockApi, MockQuerier>) {
    let msg = InstantiateMsg {
        policy: String::from("policy"),
    };

    let info = mock_info("policy", &[]);

    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
}
