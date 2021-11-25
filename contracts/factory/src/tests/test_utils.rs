use crate::contract::instantiate;
use cosmwasm_std::testing::{mock_env, mock_info, MockApi, MockQuerier, MockStorage};
use cosmwasm_std::OwnedDeps;

use olympus_pro::factory::InstantiateMsg;

pub fn instantiate_factory(deps: &mut OwnedDeps<MockStorage, MockApi, MockQuerier>) {
    let msg = InstantiateMsg {
        custom_bond_id: 1,
        custom_treasury_id: 2,
        treasury: String::from("treasury"),
        subsidy_router: String::from("subsidy_router"),
        olympus_dao: String::from("olympus_dao"),
    };

    let info = mock_info("policy", &[]);

    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
}
