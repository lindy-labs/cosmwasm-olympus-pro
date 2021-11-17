use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub treasury: String,
    pub factory_storage: String,
    pub subsidy_router: String,
    pub olympus_dao: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    CreateBondAndTreasury {
        payout_token: String,
        principal_token: String,
        initial_owner: String,
        tier_ceilings: Vec<u64>,
        fees: Vec<u64>,
        fee_in_payout: bool,
    },
    CreateBond {
        principal_token: String,
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
    pub treasury: String,
    pub factory_storage: String,
    pub subsidy_router: String,
    pub olympus_dao: String,
}
