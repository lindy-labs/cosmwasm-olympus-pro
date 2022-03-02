use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::CanonicalAddr;
use cw_storage_plus::{Item, Map};

use olympus_pro::custom_bond::FeeTier;
use terraswap::asset::AssetInfoRaw;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub custom_bond_id: u64,
    pub custom_treasury_id: u64,
    pub treasury: CanonicalAddr,
    pub subsidy_router: CanonicalAddr,
    pub olympus_dao: CanonicalAddr,
    pub policy: CanonicalAddr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub bond_length: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BondInfo {
    pub principal_token: AssetInfoRaw,
    pub custom_treasury: CanonicalAddr,
    pub bond: CanonicalAddr,
    pub initial_owner: CanonicalAddr,
    pub fee_tiers: Vec<FeeTier>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TempBondInfo {
    pub principal_token: AssetInfoRaw,
    pub custom_treasury: Option<CanonicalAddr>,
    pub initial_owner: CanonicalAddr,
    pub fee_tiers: Vec<FeeTier>,
    pub fee_in_payout: bool,
}

pub const CONFIGURATION: Item<Config> = Item::new("config");
pub const STATE: Item<State> = Item::new("state");
pub const TEMP_BOND_INFO: Item<TempBondInfo> = Item::new("temp_bond_info");
pub const BOND_INFOS: Map<&[u8], BondInfo> = Map::new("bond_infos");
