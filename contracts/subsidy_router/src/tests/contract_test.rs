use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{attr, from_binary, StdError};

use olympus_pro::subsidy_router::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg};

use crate::{
    contract::{execute, instantiate, query},
    tests::test_utils::instantiate_subsidy_router,
};

#[test]
fn test_initialization() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        policy: String::from("policy"),
    };

    let info = mock_info("policy", &[]);

    // we can just call .unwrap() to assert this was a success
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // it worked, let's query the config and state
    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!(
        ConfigResponse {
            policy: String::from("policy"),
        },
        config
    );
}

#[test]
fn test_update_config_fails_if_unauthorized() {
    let mut deps = mock_dependencies(&[]);

    instantiate_subsidy_router(&mut deps);

    let info = mock_info("addr", &[]);
    let msg = ExecuteMsg::UpdateConfig {
        policy: Some(String::from("new_policy")),
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(res, StdError::generic_err("unauthorized"));
}

#[test]
fn test_update_config_by_policy() {
    let mut deps = mock_dependencies(&[]);

    instantiate_subsidy_router(&mut deps);

    let info = mock_info("policy", &[]);
    let msg = ExecuteMsg::UpdateConfig {
        policy: Some(String::from("new_policy")),
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(res.attributes, vec![attr("action", "update_config"),]);

    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!(
        ConfigResponse {
            policy: String::from("new_policy"),
        },
        config
    );
}

#[test]
fn test_add_subsidy_controller_fails_if_unauthorized() {
    let mut deps = mock_dependencies(&[]);

    instantiate_subsidy_router(&mut deps);

    let info = mock_info("addr", &[]);
    let msg = ExecuteMsg::AddSubsidyController {
        subsidy_controller: String::from("subsidy_controller"),
        bond: String::from("bond"),
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(res, StdError::generic_err("unauthorized"));
}

#[test]
fn test_add_subsidy_controller_by_policy() {
    let mut deps = mock_dependencies(&[]);

    instantiate_subsidy_router(&mut deps);

    let info = mock_info("policy", &[]);
    let msg = ExecuteMsg::AddSubsidyController {
        subsidy_controller: String::from("subsidy_controller"),
        bond: String::from("bond"),
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(
        res.attributes,
        vec![
            attr("action", "add_subsidy_controller"),
            attr("subsidy_controller", "subsidy_controller"),
            attr("bond", "bond"),
        ]
    );

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::BondForController {
            subsidy_controller: String::from("subsidy_controller"),
        },
    )
    .unwrap();
    let bond: String = from_binary(&res).unwrap();
    assert_eq!(String::from("bond"), bond);
}

#[test]
fn test_remove_subsidy_controller_by_policy() {
    let mut deps = mock_dependencies(&[]);

    instantiate_subsidy_router(&mut deps);

    let info = mock_info("policy", &[]);
    let msg = ExecuteMsg::AddSubsidyController {
        subsidy_controller: String::from("subsidy_controller"),
        bond: String::from("bond"),
    };

    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    let info = mock_info("policy", &[]);
    let msg = ExecuteMsg::RemoveSubsidyController {
        subsidy_controller: String::from("subsidy_controller"),
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(
        res.attributes,
        vec![
            attr("action", "remove_subsidy_controller"),
            attr("subsidy_controller", "subsidy_controller"),
        ]
    );

    query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::BondForController {
            subsidy_controller: String::from("subsidy_controller"),
        },
    )
    .unwrap_err();
}
