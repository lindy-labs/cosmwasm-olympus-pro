use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::CanonicalAddr;
use cw_storage_plus::{Item, Map};

use olympus_pro::custom_bond::{BondInfo, FeeTier, State};
use terraswap::asset::AssetInfoRaw;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub custom_treasury: CanonicalAddr,
    pub payout_token: CanonicalAddr,
    pub principal_token: AssetInfoRaw,
    pub olympus_treasury: CanonicalAddr,
    pub subsidy_router: CanonicalAddr,
    pub policy: CanonicalAddr,
    pub olympus_dao: CanonicalAddr,
    pub fee_tiers: Vec<FeeTier>,
    pub fee_in_payout: bool,
    pub payout_decimals: u8,
    pub principal_decimals: u8,
}

pub const CONFIGURATION: Item<Config> = Item::new("config");
pub const STATE: Item<State> = Item::new("state");
pub const BOND_INFOS: Map<&[u8], BondInfo> = Map::new("bond_infos");
