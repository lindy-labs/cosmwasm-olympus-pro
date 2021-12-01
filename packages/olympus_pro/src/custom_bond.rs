use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Decimal, Uint128};

use terraswap::asset::AssetInfo;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub custom_treasury: String,
    pub principal_token: AssetInfo,
    pub olympus_treasury: String,
    pub subsidy_router: String,
    pub initial_owner: String,
    pub olympus_dao: String,
    pub tier_ceilings: Vec<u64>,
    pub fees: Vec<u64>,
    pub fee_in_payout: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    InitializeBond {
        terms: Terms,
        initial_debt: Uint128,
    },
    SetBondTerms {
        vesting_term: Option<Uint128>,
        max_payout: Option<Uint128>,
        max_debt: Option<Uint128>,
    },
    SetAdjustment {
        addition: bool,
        increment: Uint128,
        target: Uint128,
        buffer: Uint128,
    },
    UpdateConfig {
        policy: Option<String>,
        olympus_treasury: Option<String>,
    },
    PaySubsidy {},
    Deposit {
        amount: Uint128,
        max_price: Uint128,
        depositor: String,
    },
    Redeem {
        depositor: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MigrateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    State {},
    BondPrice {},
    MaxPayout {},
    PayoutFor { value: Uint128 },
    Debt {},
    PayoutInfo { depositor: String },
    CurrentOlympusFee {},
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub custom_treasury: String,
    pub payout_token: AssetInfo,
    pub principal_token: AssetInfo,
    pub olympus_treasury: String,
    pub subsidy_router: String,
    pub policy: String,
    pub olympus_dao: String,
    pub tier_ceilings: Vec<u64>,
    pub fees: Vec<u64>,
    pub fee_in_payout: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Terms {
    pub control_variable: Uint128,
    pub vesting_term: Uint128,
    pub minimum_price: Uint128,
    pub max_payout: Uint128,
    pub max_debt: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Adjustment {
    pub addition: bool,
    pub rate: Uint128,
    pub target: Uint128,
    pub buffer: Uint128,
    pub last_block: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub current_debt: Uint128,
    pub total_debt: Uint128,
    pub terms: Terms,
    pub last_decay: u64,
    pub adjustment: Adjustment,
    pub payout_since_last_subsidy: Uint128,
}
