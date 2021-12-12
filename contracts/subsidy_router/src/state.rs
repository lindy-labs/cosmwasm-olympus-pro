use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{CanonicalAddr, StdResult, Storage};
use cosmwasm_storage::{bucket, bucket_read, singleton, singleton_read, Bucket};

const KEY_CONFIG: &[u8] = b"config";
const PREFIX_KEY_SUBSIDY_CONTROLLER: &[u8] = b"subsidy_controller";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub policy: CanonicalAddr,
}

pub fn store_config(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
    singleton(storage, KEY_CONFIG).save(config)
}

pub fn read_config(storage: &dyn Storage) -> StdResult<Config> {
    Ok(singleton_read(storage, KEY_CONFIG).load()?)
}

pub fn store_subsidy_controller(
    storage: &mut dyn Storage,
    subsidy_controller: &CanonicalAddr,
    bond: &CanonicalAddr,
) -> StdResult<()> {
    bucket(storage, PREFIX_KEY_SUBSIDY_CONTROLLER).save(&subsidy_controller.as_slice(), bond)
}

pub fn read_subsidy_controller(
    storage: &dyn Storage,
    subsidy_controller: &CanonicalAddr,
) -> StdResult<CanonicalAddr> {
    bucket_read(storage, PREFIX_KEY_SUBSIDY_CONTROLLER).load(&subsidy_controller.as_slice())
}

pub fn remove_subsidy_controller(storage: &mut dyn Storage, subsidy_controller: CanonicalAddr) {
    let mut bond_bucket: Bucket<CanonicalAddr> = bucket(storage, PREFIX_KEY_SUBSIDY_CONTROLLER);
    bond_bucket.remove(&subsidy_controller.as_slice());
}
