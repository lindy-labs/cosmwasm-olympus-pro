use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use terraswap::asset::AssetInfo;

use crate::custom_bond::FeeTier;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub custom_bond_id: u64,
    pub custom_treasury_id: u64,
    pub treasury: String,
    pub subsidy_router: String,
    pub olympus_dao: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    UpdateConfig {
        custom_bond_id: Option<u64>,
        custom_treasury_id: Option<u64>,
        policy: Option<String>,
    },
    CreateBondAndTreasury {
        payout_token: String,
        principal_token: AssetInfo,
        initial_owner: String,
        fee_tiers: Vec<FeeTier>,
        fee_in_payout: bool,
    },
    CreateBond {
        principal_token: AssetInfo,
        custom_treasury: String,
        initial_owner: String,
        fee_tiers: Vec<FeeTier>,
        fee_in_payout: bool,
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
    BondInfo { bond_id: u64 },
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub custom_bond_id: u64,
    pub custom_treasury_id: u64,
    pub treasury: String,
    pub subsidy_router: String,
    pub olympus_dao: String,
    pub policy: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BondInfoResponse {
    pub principal_token: AssetInfo,
    pub custom_treasury: String,
    pub bond: String,
    pub initial_owner: String,
    pub fee_tiers: Vec<FeeTier>,
}
