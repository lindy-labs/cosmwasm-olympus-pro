use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    attr, from_binary, to_binary, ContractResult, Decimal, Reply, ReplyOn, StdError, SubMsg,
    SubMsgExecutionResponse, Uint128, WasmMsg,
};

use olympus_pro::{
    custom_bond::{FeeTier, InstantiateMsg as CustomBondInstantiateMsg},
    custom_treasury::InstantiateMsg as CustomTreasuryInstantiateMsg,
    factory::{BondInfoResponse, ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg},
};

use protobuf::Message;
use terraswap::asset::AssetInfo;

use crate::{
    contract::{execute, instantiate, query, reply},
    response::MsgInstantiateContractResponse,
    state::State,
    tests::test_utils::instantiate_factory,
};

#[test]
fn test_initialization() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        custom_bond_id: 1,
        custom_treasury_id: 2,
        treasury: String::from("treasury"),
        subsidy_router: String::from("subsidy_router"),
        olympus_dao: String::from("olympus_dao"),
    };

    let info = mock_info("policy", &[]);

    // we can just call .unwrap() to assert this was a success
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // it worked, let's query the config and state
    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!(
        ConfigResponse {
            custom_bond_id: 1,
            custom_treasury_id: 2,
            treasury: String::from("treasury"),
            subsidy_router: String::from("subsidy_router"),
            olympus_dao: String::from("olympus_dao"),
            policy: String::from("policy"),
        },
        config
    );

    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: State = from_binary(&res).unwrap();
    assert_eq!(State { bond_length: 0 }, state);
}

#[test]
fn test_update_config_fails_if_unauthorized() {
    let mut deps = mock_dependencies(&[]);

    instantiate_factory(&mut deps);

    let info = mock_info("addr", &[]);
    let msg = ExecuteMsg::UpdateConfig {
        custom_bond_id: Some(3),
        custom_treasury_id: Some(4),
        policy: Some(String::from("new_policy")),
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(res, StdError::generic_err("unauthorized"));
}

#[test]
fn test_update_config_by_policy() {
    let mut deps = mock_dependencies(&[]);

    instantiate_factory(&mut deps);

    let info = mock_info("policy", &[]);

    let msg = ExecuteMsg::UpdateConfig {
        custom_bond_id: Some(3),
        custom_treasury_id: Some(4),
        policy: Some(String::from("new_policy")),
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    assert_eq!(res.attributes, vec![attr("action", "update_config"),]);

    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!(
        ConfigResponse {
            custom_bond_id: 3,
            custom_treasury_id: 4,
            treasury: String::from("treasury"),
            subsidy_router: String::from("subsidy_router"),
            olympus_dao: String::from("olympus_dao"),
            policy: String::from("new_policy"),
        },
        config
    );
}

#[test]
fn test_create_bond_fails_if_unauthorized() {
    let mut deps = mock_dependencies(&[]);

    instantiate_factory(&mut deps);

    let info = mock_info("addr", &[]);
    let msg = ExecuteMsg::CreateBond {
        principal_token: AssetInfo::NativeToken {
            denom: String::from("principal"),
        },
        custom_treasury: String::from("custom_treasury"),
        initial_owner: String::from("initial_owner"),
        fee_tiers: vec![
            FeeTier {
                tier_ceiling: Uint128::from(1u128),
                fee_rate: Decimal::percent(3),
            },
            FeeTier {
                tier_ceiling: Uint128::from(2u128),
                fee_rate: Decimal::percent(4),
            },
        ],
        fee_in_payout: true,
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(res, StdError::generic_err("unauthorized"));
}

#[test]
fn test_create_bond_by_policy() {
    let mut deps = mock_dependencies(&[]);

    instantiate_factory(&mut deps);

    let info = mock_info("policy", &[]);
    let msg = ExecuteMsg::CreateBond {
        principal_token: AssetInfo::NativeToken {
            denom: String::from("principal"),
        },
        custom_treasury: String::from("custom_treasury"),
        initial_owner: String::from("initial_owner"),
        fee_tiers: vec![
            FeeTier {
                tier_ceiling: Uint128::from(1u128),
                fee_rate: Decimal::percent(3),
            },
            FeeTier {
                tier_ceiling: Uint128::from(2u128),
                fee_rate: Decimal::percent(4),
            },
        ],
        fee_in_payout: true,
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(res.attributes, vec![attr("action", "create_bond"),]);
    assert_eq!(
        res.messages,
        vec![SubMsg {
            id: 2,
            gas_limit: None,
            msg: WasmMsg::Instantiate {
                code_id: 1,
                funds: vec![],
                admin: Some(String::from(MOCK_CONTRACT_ADDR)),
                label: "OlympusPro Custom Bond".to_string(),
                msg: to_binary(&CustomBondInstantiateMsg {
                    custom_treasury: String::from("custom_treasury"),
                    principal_token: AssetInfo::NativeToken {
                        denom: String::from("principal"),
                    },
                    olympus_treasury: String::from("custom_treasury"),
                    subsidy_router: String::from("subsidy_router"),
                    initial_owner: String::from("initial_owner"),
                    olympus_dao: String::from("olympus_dao"),
                    fee_tiers: vec![
                        FeeTier {
                            tier_ceiling: Uint128::from(1u128),
                            fee_rate: Decimal::percent(3),
                        },
                        FeeTier {
                            tier_ceiling: Uint128::from(2u128),
                            fee_rate: Decimal::percent(4),
                        },
                    ],
                    fee_in_payout: true,
                })
                .unwrap(),
            }
            .into(),
            reply_on: ReplyOn::Success,
        }]
    );
}

#[test]
fn test_create_bond_register_bond_on_reply() {
    let mut deps = mock_dependencies(&[]);

    instantiate_factory(&mut deps);

    let info = mock_info("policy", &[]);
    let msg = ExecuteMsg::CreateBond {
        principal_token: AssetInfo::NativeToken {
            denom: String::from("principal"),
        },
        custom_treasury: String::from("custom_treasury"),
        initial_owner: String::from("initial_owner"),
        fee_tiers: vec![
            FeeTier {
                tier_ceiling: Uint128::from(1u128),
                fee_rate: Decimal::percent(3),
            },
            FeeTier {
                tier_ceiling: Uint128::from(2u128),
                fee_rate: Decimal::percent(4),
            },
        ],
        fee_in_payout: true,
    };

    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    let mut bond_inst_res = MsgInstantiateContractResponse::new();
    bond_inst_res.set_contract_address("bond0".to_string());

    let reply_msg = Reply {
        id: 2,
        result: ContractResult::Ok(SubMsgExecutionResponse {
            events: vec![],
            data: Some(bond_inst_res.write_to_bytes().unwrap().into()),
        }),
    };

    reply(deps.as_mut(), mock_env(), reply_msg).unwrap();

    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: State = from_binary(&res).unwrap();
    assert_eq!(State { bond_length: 1 }, state);

    let res = query(deps.as_ref(), mock_env(), QueryMsg::BondInfo { bond_id: 0 }).unwrap();
    let bond_info: BondInfoResponse = from_binary(&res).unwrap();
    assert_eq!(
        BondInfoResponse {
            principal_token: AssetInfo::NativeToken {
                denom: String::from("principal"),
            },
            custom_treasury: String::from("custom_treasury"),
            bond: String::from("bond0"),
            initial_owner: String::from("initial_owner"),
            fee_tiers: vec![
                FeeTier {
                    tier_ceiling: Uint128::from(1u128),
                    fee_rate: Decimal::percent(3),
                },
                FeeTier {
                    tier_ceiling: Uint128::from(2u128),
                    fee_rate: Decimal::percent(4),
                },
            ],
        },
        bond_info
    );
}

#[test]
fn test_create_bond_and_treasury_fails_if_unauthorized() {
    let mut deps = mock_dependencies(&[]);

    instantiate_factory(&mut deps);

    let info = mock_info("addr", &[]);
    let msg = ExecuteMsg::CreateBondAndTreasury {
        payout_token: String::from("payout"),
        principal_token: AssetInfo::NativeToken {
            denom: String::from("principal"),
        },
        initial_owner: String::from("initial_owner"),
        fee_tiers: vec![
            FeeTier {
                tier_ceiling: Uint128::from(1u128),
                fee_rate: Decimal::percent(3),
            },
            FeeTier {
                tier_ceiling: Uint128::from(2u128),
                fee_rate: Decimal::percent(4),
            },
        ],
        fee_in_payout: true,
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(res, StdError::generic_err("unauthorized"));
}

#[test]
fn test_create_bond_and_treasury_by_policy() {
    let mut deps = mock_dependencies(&[]);

    instantiate_factory(&mut deps);

    let info = mock_info("policy", &[]);
    let msg = ExecuteMsg::CreateBondAndTreasury {
        payout_token: String::from("payout"),
        principal_token: AssetInfo::NativeToken {
            denom: String::from("principal"),
        },
        initial_owner: String::from("initial_owner"),
        fee_tiers: vec![
            FeeTier {
                tier_ceiling: Uint128::from(1u128),
                fee_rate: Decimal::percent(3),
            },
            FeeTier {
                tier_ceiling: Uint128::from(2u128),
                fee_rate: Decimal::percent(4),
            },
        ],
        fee_in_payout: true,
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(res.attributes, vec![attr("action", "create_treasury"),]);
    assert_eq!(
        res.messages,
        vec![SubMsg {
            id: 1,
            gas_limit: None,
            msg: WasmMsg::Instantiate {
                code_id: 2,
                funds: vec![],
                admin: Some(String::from(MOCK_CONTRACT_ADDR)),
                label: "OlympusPro Custom Treasury".to_string(),
                msg: to_binary(&CustomTreasuryInstantiateMsg {
                    payout_token: String::from("payout"),
                    initial_owner: String::from("initial_owner"),
                })
                .unwrap(),
            }
            .into(),
            reply_on: ReplyOn::Success,
        }]
    );
}

#[test]
fn test_create_bond_and_treasury_reqeust_create_bond_on_first_reply() {
    let mut deps = mock_dependencies(&[]);

    instantiate_factory(&mut deps);

    let info = mock_info("policy", &[]);
    let msg = ExecuteMsg::CreateBondAndTreasury {
        payout_token: String::from("payout"),
        principal_token: AssetInfo::NativeToken {
            denom: String::from("principal"),
        },
        initial_owner: String::from("initial_owner"),
        fee_tiers: vec![
            FeeTier {
                tier_ceiling: Uint128::from(1u128),
                fee_rate: Decimal::percent(3),
            },
            FeeTier {
                tier_ceiling: Uint128::from(2u128),
                fee_rate: Decimal::percent(4),
            },
        ],
        fee_in_payout: true,
    };

    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    let mut treasury_inst_res = MsgInstantiateContractResponse::new();
    treasury_inst_res.set_contract_address("treasury0".to_string());

    let reply_msg = Reply {
        id: 1,
        result: ContractResult::Ok(SubMsgExecutionResponse {
            events: vec![],
            data: Some(treasury_inst_res.write_to_bytes().unwrap().into()),
        }),
    };

    let res = reply(deps.as_mut(), mock_env(), reply_msg).unwrap();

    assert_eq!(res.attributes, vec![attr("action", "create_bond"),]);
    assert_eq!(
        res.messages,
        vec![SubMsg {
            id: 2,
            gas_limit: None,
            msg: WasmMsg::Instantiate {
                code_id: 1,
                funds: vec![],
                admin: Some(String::from(MOCK_CONTRACT_ADDR)),
                label: "OlympusPro Custom Bond".to_string(),
                msg: to_binary(&CustomBondInstantiateMsg {
                    custom_treasury: String::from("treasury0"),
                    principal_token: AssetInfo::NativeToken {
                        denom: String::from("principal"),
                    },
                    olympus_treasury: String::from("treasury0"),
                    subsidy_router: String::from("subsidy_router"),
                    initial_owner: String::from("initial_owner"),
                    olympus_dao: String::from("olympus_dao"),
                    fee_tiers: vec![
                        FeeTier {
                            tier_ceiling: Uint128::from(1u128),
                            fee_rate: Decimal::percent(3),
                        },
                        FeeTier {
                            tier_ceiling: Uint128::from(2u128),
                            fee_rate: Decimal::percent(4),
                        },
                    ],
                    fee_in_payout: true,
                })
                .unwrap(),
            }
            .into(),
            reply_on: ReplyOn::Success,
        }]
    );
}

#[test]
fn test_create_bond_and_treasury_register_bond_on_second_reply() {
    let mut deps = mock_dependencies(&[]);

    instantiate_factory(&mut deps);

    let info = mock_info("policy", &[]);
    let msg = ExecuteMsg::CreateBondAndTreasury {
        payout_token: String::from("payout"),
        principal_token: AssetInfo::NativeToken {
            denom: String::from("principal"),
        },
        initial_owner: String::from("initial_owner"),
        fee_tiers: vec![
            FeeTier {
                tier_ceiling: Uint128::from(1u128),
                fee_rate: Decimal::percent(3),
            },
            FeeTier {
                tier_ceiling: Uint128::from(2u128),
                fee_rate: Decimal::percent(4),
            },
        ],
        fee_in_payout: true,
    };

    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    let mut treasury_inst_res = MsgInstantiateContractResponse::new();
    treasury_inst_res.set_contract_address("treasury0".to_string());

    let reply_msg = Reply {
        id: 1,
        result: ContractResult::Ok(SubMsgExecutionResponse {
            events: vec![],
            data: Some(treasury_inst_res.write_to_bytes().unwrap().into()),
        }),
    };

    reply(deps.as_mut(), mock_env(), reply_msg).unwrap();

    let mut bond_inst_res = MsgInstantiateContractResponse::new();
    bond_inst_res.set_contract_address("bond0".to_string());

    let reply_msg = Reply {
        id: 2,
        result: ContractResult::Ok(SubMsgExecutionResponse {
            events: vec![],
            data: Some(bond_inst_res.write_to_bytes().unwrap().into()),
        }),
    };

    reply(deps.as_mut(), mock_env(), reply_msg).unwrap();

    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: State = from_binary(&res).unwrap();
    assert_eq!(State { bond_length: 1 }, state);

    let res = query(deps.as_ref(), mock_env(), QueryMsg::BondInfo { bond_id: 0 }).unwrap();
    let bond_info: BondInfoResponse = from_binary(&res).unwrap();
    assert_eq!(
        BondInfoResponse {
            principal_token: AssetInfo::NativeToken {
                denom: String::from("principal"),
            },
            custom_treasury: String::from("treasury0"),
            bond: String::from("bond0"),
            initial_owner: String::from("initial_owner"),
            fee_tiers: vec![
                FeeTier {
                    tier_ceiling: Uint128::from(1u128),
                    fee_rate: Decimal::percent(3),
                },
                FeeTier {
                    tier_ceiling: Uint128::from(2u128),
                    fee_rate: Decimal::percent(4),
                },
            ],
        },
        bond_info
    );
}
