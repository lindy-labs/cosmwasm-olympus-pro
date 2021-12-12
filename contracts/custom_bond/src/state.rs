use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{CanonicalAddr, StdResult, Storage};
use cosmwasm_storage::{bucket, bucket_read, singleton, singleton_read, Bucket};

use olympus_pro::custom_bond::{BondInfo, FeeTier, State};
use terraswap::asset::AssetInfoRaw;

const KEY_CONFIG: &[u8] = b"config";
const KEY_STATE: &[u8] = b"state";
const PREFIX_KEY_BOND_INFO: &[u8] = b"prefix_bond_info";

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

pub fn store_bond_info(
    storage: &mut dyn Storage,
    bond_info: &BondInfo,
    user: CanonicalAddr,
) -> StdResult<()> {
    bucket(storage, PREFIX_KEY_BOND_INFO).save(&user.as_slice(), bond_info)
}

pub fn read_bond_info(storage: &dyn Storage, user: CanonicalAddr) -> StdResult<BondInfo> {
    bucket_read(storage, PREFIX_KEY_BOND_INFO).load(&user.as_slice())
}

pub fn remove_bond_info(storage: &mut dyn Storage, user: CanonicalAddr) {
    let mut bond_bucket: Bucket<BondInfo> = bucket(storage, PREFIX_KEY_BOND_INFO);
    bond_bucket.remove(&user.as_slice());
}
