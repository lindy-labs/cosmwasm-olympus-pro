use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{
    attr, from_binary, to_binary, BankMsg, Coin, CosmosMsg, StdError, SubMsg, Uint128, WasmMsg,
};

use cw20::Cw20ExecuteMsg;
use olympus_pro::custom_treasury::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use terraswap::asset::{Asset, AssetInfo};

use crate::{
    contract::{execute, instantiate, query},
    tests::{mock_querier::mock_dependencies, test_utils::instantiate_custom_treasury},
};

#[test]
fn test_initialization() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        payout_token: String::from("payout_token"),
        initial_owner: String::from("policy"),
    };

    let info = mock_info("policy", &[]);

    // we can just call .unwrap() to assert this was a success
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // it worked, let's query the config and state
    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!(
        ConfigResponse {
            payout_token: String::from("payout_token"),
            policy: String::from("policy"),
        },
        config
    );
}

#[test]
fn test_update_config_fails_if_unauthorized() {
    let mut deps = mock_dependencies(&[]);

    instantiate_custom_treasury(&mut deps);

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

    instantiate_custom_treasury(&mut deps);

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
            payout_token: String::from("payout_token"),
            policy: String::from("new_policy"),
        },
        config
    );
}

#[test]
fn test_whitelist_bond_fails_if_unauthorized() {
    let mut deps = mock_dependencies(&[]);

    instantiate_custom_treasury(&mut deps);

    let info = mock_info("addr", &[]);
    let msg = ExecuteMsg::WhitelistBond {
        bond: String::from("bond"),
        whitelist: true,
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(res, StdError::generic_err("unauthorized"));
}

#[test]
fn test_whitelist_bond_by_policy() {
    let mut deps = mock_dependencies(&[]);

    instantiate_custom_treasury(&mut deps);

    let info = mock_info("policy", &[]);
    let msg = ExecuteMsg::WhitelistBond {
        bond: String::from("bond"),
        whitelist: true,
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(
        res.attributes,
        vec![
            attr("action", "whitelist_bond"),
            attr("bond", "bond"),
            attr("whitelist", "true"),
        ]
    );

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::BondWhitelist {
            bond: String::from("bond"),
        },
    )
    .unwrap();
    let whitelist: bool = from_binary(&res).unwrap();
    assert_eq!(true, whitelist);
}

#[test]
fn test_remove_whitelist_bond_by_policy() {
    let mut deps = mock_dependencies(&[]);

    instantiate_custom_treasury(&mut deps);

    let info = mock_info("policy", &[]);
    let msg = ExecuteMsg::WhitelistBond {
        bond: String::from("bond"),
        whitelist: true,
    };

    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    let info = mock_info("policy", &[]);
    let msg = ExecuteMsg::WhitelistBond {
        bond: String::from("bond"),
        whitelist: false,
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(
        res.attributes,
        vec![
            attr("action", "whitelist_bond"),
            attr("bond", "bond"),
            attr("whitelist", "false"),
        ]
    );

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::BondWhitelist {
            bond: String::from("bond"),
        },
    )
    .unwrap();
    let whitelist: bool = from_binary(&res).unwrap();
    assert_eq!(false, whitelist);
}

#[test]
fn test_withdraw_fails_if_unauthorized() {
    let mut deps = mock_dependencies(&[]);

    instantiate_custom_treasury(&mut deps);

    let info = mock_info("addr", &[]);
    let msg = ExecuteMsg::Withdraw {
        asset: Asset {
            info: AssetInfo::NativeToken {
                denom: "utoken".to_string(),
            },
            amount: Uint128::from(100000000u128),
        },
        recipient: String::from("recipient"),
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(res, StdError::generic_err("unauthorized"));
}

#[test]
fn test_withdraw_native_token_by_policy() {
    let mut deps = mock_dependencies(&[]);

    instantiate_custom_treasury(&mut deps);

    let info = mock_info("policy", &[]);
    let msg = ExecuteMsg::Withdraw {
        asset: Asset {
            info: AssetInfo::NativeToken {
                denom: "utoken".to_string(),
            },
            amount: Uint128::from(100000000u128),
        },
        recipient: String::from("recipient"),
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(
        res.attributes,
        vec![
            attr("action", "withdraw"),
            attr("amount", "100000000"),
            attr("recipient", String::from("recipient")),
        ]
    );
    assert_eq!(
        res.messages,
        vec![SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
            to_address: String::from("recipient"),
            amount: vec![Coin {
                denom: "utoken".to_string(),
                amount: Uint128::from(100000000u128)
            }],
        }))]
    );
}

#[test]
fn test_withdraw_cw20_token_by_policy() {
    let mut deps = mock_dependencies(&[]);

    instantiate_custom_treasury(&mut deps);

    let info = mock_info("policy", &[]);
    let msg = ExecuteMsg::Withdraw {
        asset: Asset {
            info: AssetInfo::Token {
                contract_addr: "utoken".to_string(),
            },
            amount: Uint128::from(100000000u128),
        },
        recipient: String::from("recipient"),
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(
        res.attributes,
        vec![
            attr("action", "withdraw"),
            attr("amount", "100000000"),
            attr("recipient", String::from("recipient")),
        ]
    );
    assert_eq!(
        res.messages,
        vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "utoken".to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: String::from("recipient"),
                amount: 100000000u128.into(),
            })
            .unwrap(),
            funds: vec![],
        }))]
    );
}

#[test]
fn test_send_payout_tokens_fails_if_unauthorized() {
    let mut deps = mock_dependencies(&[]);

    instantiate_custom_treasury(&mut deps);

    let info = mock_info("addr", &[]);
    let msg = ExecuteMsg::SendPayoutTokens {
        amount: Uint128::from(100000000u128),
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(res, StdError::generic_err("not whitelisted"));
}

#[test]
fn test_send_payout_tokens_by_bond() {
    let mut deps = mock_dependencies(&[]);

    instantiate_custom_treasury(&mut deps);

    let info = mock_info("policy", &[]);
    let msg = ExecuteMsg::WhitelistBond {
        bond: String::from("bond"),
        whitelist: true,
    };

    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    let info = mock_info("bond", &[]);
    let msg = ExecuteMsg::SendPayoutTokens {
        amount: Uint128::from(100000000u128),
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(
        res.attributes,
        vec![
            attr("action", "send_payout_token"),
            attr("amount", "100000000"),
            attr("recipient", String::from("bond")),
        ]
    );
    assert_eq!(
        res.messages,
        vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "payout_token".to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: String::from("bond"),
                amount: 100000000u128.into(),
            })
            .unwrap(),
            funds: vec![],
        }))]
    );
}

// #[test]
// fn test_query_value_of_token_when_same_decimals() {
//     let mut deps = mock_dependencies(&[]);

//     instantiate_custom_treasury(&mut deps);

//     let res = query(
//         deps.as_ref(),
//         mock_env(),
//         QueryMsg::ValueOfToken {
//             principal_asset: Asset {
//                 info: AssetInfo::NativeToken {
//                     denom: "utoken".to_string(),
//                 },
//                 amount: Uint128::from(100000000u128),
//             },
//         },
//     )
//     .unwrap();
//     let value_of_token: Uint128 = from_binary(&res).unwrap();
//     assert_eq!(Uint128::from(100000000u128), value_of_token);
// }

// #[test]
// fn test_query_value_of_token_when_less_decimals() {
//     let mut deps = mock_dependencies(&[]);

//     instantiate_custom_treasury(&mut deps);

//     deps.querier.with_token_mock_decimals(3);

//     let res = query(
//         deps.as_ref(),
//         mock_env(),
//         QueryMsg::ValueOfToken {
//             principal_asset: Asset {
//                 info: AssetInfo::Token {
//                     contract_addr: "utoken".to_string(),
//                 },
//                 amount: Uint128::from(100000000u128),
//             },
//         },
//     )
//     .unwrap();
//     let value_of_token: Uint128 = from_binary(&res).unwrap();
//     assert_eq!(Uint128::from(100000000000u128), value_of_token);
// }

// #[test]
// fn test_query_value_of_token_when_more_decimals() {
//     let mut deps = mock_dependencies(&[]);

//     instantiate_custom_treasury(&mut deps);

//     deps.querier.with_token_mock_decimals(9);

//     let res = query(
//         deps.as_ref(),
//         mock_env(),
//         QueryMsg::ValueOfToken {
//             principal_asset: Asset {
//                 info: AssetInfo::Token {
//                     contract_addr: "utoken".to_string(),
//                 },
//                 amount: Uint128::from(100000000u128),
//             },
//         },
//     )
//     .unwrap();
//     let value_of_token: Uint128 = from_binary(&res).unwrap();
//     assert_eq!(Uint128::from(100000u128), value_of_token);
// }
