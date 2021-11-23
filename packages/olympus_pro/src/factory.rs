use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use terraswap::asset::AssetInfo;

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
    CreateTreasury {
        payout_token: AssetInfo,
        initial_owner: String,
    },
    CreateBond {
        principal_token: AssetInfo,
        custom_treasury: String,
        initial_owner: String,
        tier_ceilings: Vec<u64>,
        fees: Vec<u64>,
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
