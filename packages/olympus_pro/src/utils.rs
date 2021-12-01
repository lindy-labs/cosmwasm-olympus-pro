use cosmwasm_std::{Decimal, Uint128};
use terraswap::asset::Asset;

pub fn get_value_of_token(
    principal_asset: Asset,
    payout_decimals: u8,
    principal_decimals: u8,
) -> Uint128 {
    if payout_decimals == principal_decimals {
        principal_asset.amount
    } else if payout_decimals > principal_decimals {
        principal_asset.amount
            * Uint128::from(
                10u128
                    .checked_pow((payout_decimals - principal_decimals) as u32)
                    .unwrap(),
            )
    } else {
        principal_asset.amount
            * Decimal::from_ratio(
                1u128,
                10u128
                    .checked_pow((principal_decimals - payout_decimals) as u32)
                    .unwrap(),
            )
    }
}
