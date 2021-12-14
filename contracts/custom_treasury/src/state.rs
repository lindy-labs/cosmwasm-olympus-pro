use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{CanonicalAddr, StdResult, Storage};
use cosmwasm_storage::{bucket, bucket_read, singleton, singleton_read};

const KEY_CONFIG: &[u8] = b"config";
const PREFIX_KEY_BOND_WHITELIST: &[u8] = b"bond_whitelist";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub payout_token: CanonicalAddr,
    pub policy: CanonicalAddr,
}

pub fn store_config(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
    singleton(storage, KEY_CONFIG).save(config)
}

pub fn read_config(storage: &dyn Storage) -> StdResult<Config> {
    Ok(singleton_read(storage, KEY_CONFIG).load()?)
}

pub fn store_bond_whitelist(
    storage: &mut dyn Storage,
    bond: &CanonicalAddr,
    whitelist: &bool,
) -> StdResult<()> {
    bucket(storage, PREFIX_KEY_BOND_WHITELIST).save(&bond.as_slice(), whitelist)
}

pub fn read_bond_whitelist(storage: &dyn Storage, bond: &CanonicalAddr) -> StdResult<bool> {
    bucket_read(storage, PREFIX_KEY_BOND_WHITELIST).load(&bond.as_slice())
}
