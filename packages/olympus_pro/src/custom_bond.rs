use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Uint128;

use terraswap::asset::AssetInfo;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum Parameter {
    Vesting,
    Payout,
    Debt,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub custom_treasury: String,
    pub principal_token: AssetInfo,
    pub olympus_treasury: String,
    pub subsidy_router: String,
    pub initial_owner: String,
    pub tier_ceilings: Vec<u64>,
    pub fees: Vec<u64>,
    pub fee_in_payout: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    InitializeBond {
        control_variable: Uint128,
        vesting_term: Uint128,
        minimum_price: Uint128,
        max_payout: Uint128,
        max_debt: Uint128,
        initial_debt: Uint128,
    },
    SetBondTerms {
        parameter: Parameter,
        input: Uint128,
    },
    SetAdjustment {
        addition: bool,
        increment: Uint128,
        target: Uint128,
        buffer: Uint128,
    },
    UpdateConfig {
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
    pub principal_token: AssetInfo,
    pub olympus_treasury: String,
    pub subsidy_router: String,
    pub initial_owner: String,
    pub tier_ceilings: Vec<u64>,
    pub fees: Vec<u64>,
    pub fee_in_payout: bool,
}
