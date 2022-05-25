use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::CanonicalAddr;
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub payout_token: CanonicalAddr,
    pub policy: CanonicalAddr,
}

pub const CONFIGURATION: Item<Config> = Item::new("config");
pub const BOND_WHITELISTS: Map<&[u8], bool> = Map::new("bond_whitelist");
