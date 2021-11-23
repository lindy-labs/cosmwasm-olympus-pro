use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{CanonicalAddr, StdResult, Storage};
use cosmwasm_storage::{bucket, bucket_read, singleton, singleton_read};

const KEY_CONFIG: &[u8] = b"config";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub custom_bond_id: u64,
    pub custom_treasury_id: u64,
    pub treasury: CanonicalAddr,
    pub subsidy_router: CanonicalAddr,
    pub olympus_dao: CanonicalAddr,
    pub policy: CanonicalAddr,
}

pub fn store_config(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
    singleton(storage, KEY_CONFIG).save(config)
}

pub fn read_config(storage: &dyn Storage) -> StdResult<Config> {
    Ok(singleton_read(storage, KEY_CONFIG).load()?)
}
