use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{CanonicalAddr, StdResult, Storage};
use cosmwasm_storage::{bucket, bucket_read, singleton, singleton_read, Singleton};

use olympus_pro::custom_bond::FeeTier;
use terraswap::asset::AssetInfoRaw;

const KEY_CONFIG: &[u8] = b"config";
const KEY_TEMP_BOND_INFO: &[u8] = b"temp_bond_info";
const KEY_STATE: &[u8] = b"state";
const PREFIX_KEY_BOND_INFO: &[u8] = b"prefix_bond_info";

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

pub fn store_temp_bond_info(
    storage: &mut dyn Storage,
    temp_bond_info: &TempBondInfo,
) -> StdResult<()> {
    singleton(storage, KEY_TEMP_BOND_INFO).save(temp_bond_info)
}

pub fn read_temp_bond_info(storage: &dyn Storage) -> StdResult<TempBondInfo> {
    Ok(singleton_read(storage, KEY_TEMP_BOND_INFO).load()?)
}

pub fn remove_temp_bond_info(storage: &mut dyn Storage) {
    let mut store: Singleton<TempBondInfo> = singleton(storage, KEY_TEMP_BOND_INFO);
    store.remove();
}

pub fn store_new_bond_info(storage: &mut dyn Storage, bond_info: &BondInfo) -> StdResult<()> {
    let mut state = read_state(storage)?;

    bucket(storage, PREFIX_KEY_BOND_INFO).save(&state.bond_length.to_be_bytes(), bond_info)?;

    state.bond_length += 1;

    store_state(storage, &state)
}

pub fn read_bond_info(storage: &dyn Storage, bond_id: u64) -> StdResult<BondInfo> {
    bucket_read(storage, PREFIX_KEY_BOND_INFO).load(&bond_id.to_be_bytes())
}
