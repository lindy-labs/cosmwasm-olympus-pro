use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{CanonicalAddr, StdResult, Storage};
use cosmwasm_storage::{singleton, singleton_read};

use olympus_pro::custom_bond::{FeeTier, State};
use terraswap::asset::AssetInfoRaw;

const KEY_CONFIG: &[u8] = b"config";
const KEY_STATE: &[u8] = b"state";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub custom_treasury: CanonicalAddr,
    pub payout_token: AssetInfoRaw,
    pub principal_token: AssetInfoRaw,
    pub olympus_treasury: CanonicalAddr,
    pub subsidy_router: CanonicalAddr,
    pub policy: CanonicalAddr,
    pub olympus_dao: CanonicalAddr,
    pub fee_tiers: Vec<FeeTier>,
    pub fee_in_payout: bool,
}

pub fn store_config(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
    singleton(storage, KEY_CONFIG).save(config)
}

pub fn read_config(storage: &dyn Storage) -> StdResult<Config> {
    Ok(singleton_read(storage, KEY_CONFIG).load()?)
}

pub fn store_state(storage: &mut dyn Storage, state: &State) -> StdResult<()> {
    singleton(storage, KEY_STATE).save(state)
}

pub fn read_state(storage: &dyn Storage) -> StdResult<State> {
    Ok(singleton_read(storage, KEY_STATE).load()?)
}
