use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Decimal, Uint128};
use cw20::Cw20ReceiveMsg;
use terraswap::asset::AssetInfo;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub custom_treasury: String,
    pub principal_token: AssetInfo,
    pub olympus_treasury: String,
    pub subsidy_router: String,
    pub initial_owner: String,
    pub olympus_dao: String,
    pub fee_tiers: Vec<FeeTier>,
    pub fee_in_payout: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg),
    InitializeBond {
        terms: Terms,
        initial_debt: Uint128,
    },
    SetBondTerms {
        vesting_term: Option<u64>,
        max_payout: Option<Decimal>,
        max_debt: Option<Uint128>,
    },
    SetAdjustment {
        addition: bool,
        increment: Uint128,
        target: Uint128,
        buffer: u64,
    },
    UpdateConfig {
        policy: Option<String>,
        olympus_treasury: Option<String>,
    },
    PaySubsidy {},
    Deposit {
        max_price: Uint128,
        depositor: String,
    },
    Redeem {
        user: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    Deposit {
        max_price: Uint128,
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
    PayoutFor { value: Uint128 },
    CurrentDebt {},
    CurrentOlympusFee {},
    BondInfo { user: String },
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
    pub fee_tiers: Vec<FeeTier>,
    pub fee_in_payout: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Terms {
    pub control_variable: Uint128,
    pub vesting_term: u64,
    pub minimum_price: Uint128,
    pub max_payout: Decimal,
    pub max_debt: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Adjustment {
    pub addition: bool,
    pub rate: Uint128,
    pub target: Uint128,
    pub buffer: u64,
    pub last_time: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub current_debt: Uint128,
    pub total_debt: Uint128,
    pub terms: Terms,
    pub last_decay: u64,
    pub adjustment: Adjustment,
    pub payout_since_last_subsidy: Uint128,
    pub total_principal_bonded: Uint128,
    pub total_payout_given: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct FeeTier {
    pub tier_ceiling: Uint128,
    pub fee_rate: Decimal,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, Default)]
pub struct BondInfo {
    pub payout: Uint128,
    pub vesting: u64,
    pub last_time: u64,
    pub true_price_paid: Uint128,
}
