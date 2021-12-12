use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    from_binary, from_slice, to_binary, Coin, ContractResult, Decimal, OwnedDeps, Querier,
    QuerierResult, QueryRequest, SystemError, SystemResult, Uint128, WasmQuery,
};
use std::collections::HashMap;

use cw20::{Cw20QueryMsg, TokenInfoResponse};
use olympus_pro::custom_treasury::{
    ConfigResponse as CustomTreasuryConfigResponse, QueryMsg as CustomTreasuryQueryMsg,
};
use terra_cosmwasm::{TaxCapResponse, TaxRateResponse, TerraQuery, TerraQueryWrapper, TerraRoute};

/// mock_dependencies is a drop-in replacement for cosmwasm_std::testing::mock_dependencies
/// this uses our CustomQuerier.
pub fn mock_dependencies(
    contract_balance: &[Coin],
) -> OwnedDeps<MockStorage, MockApi, WasmMockQuerier> {
    let custom_querier: WasmMockQuerier =
        WasmMockQuerier::new(MockQuerier::new(&[(MOCK_CONTRACT_ADDR, contract_balance)]));

    OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: custom_querier,
    }
}

pub struct WasmMockQuerier {
    base: MockQuerier<TerraQueryWrapper>,
    tax_querier: TaxQuerier,
    token_querier: TokenQuerier,
    custom_treasury: String,
    payout_token: String,
}

#[derive(Clone, Default)]
pub struct TokenQuerier {
    // this lets us iterate over all pairs that match the first string
    supplies: HashMap<String, Uint128>,
    decimals: HashMap<String, u8>,
}

impl TokenQuerier {
    pub fn new(supplies: &[(&String, &Uint128)], decimals: &[(&String, &u8)]) -> Self {
        TokenQuerier {
            supplies: supplies_to_map(supplies),
            decimals: decimals_to_map(decimals),
        }
    }
}

pub(crate) fn supplies_to_map(array: &[(&String, &Uint128)]) -> HashMap<String, Uint128> {
    let mut map: HashMap<String, Uint128> = HashMap::new();
    for (key, data) in array.iter() {
        map.insert((*key).clone(), **data);
    }
    map
}

pub(crate) fn decimals_to_map(array: &[(&String, &u8)]) -> HashMap<String, u8> {
    let mut map: HashMap<String, u8> = HashMap::new();
    for (key, data) in array.iter() {
        map.insert((*key).clone(), **data);
    }
    map
}

#[derive(Clone, Default)]
pub struct TaxQuerier {
    rate: Decimal,
    // this lets us iterate over all pairs that match the first string
    caps: HashMap<String, Uint128>,
}

impl TaxQuerier {
    pub fn new(rate: Decimal, caps: &[(&String, &Uint128)]) -> Self {
        TaxQuerier {
            rate,
            caps: caps_to_map(caps),
        }
    }
}

pub(crate) fn caps_to_map(caps: &[(&String, &Uint128)]) -> HashMap<String, Uint128> {
    let mut owner_map: HashMap<String, Uint128> = HashMap::new();
    for (denom, cap) in caps.iter() {
        owner_map.insert(denom.to_string(), **cap);
    }
    owner_map
}

impl Querier for WasmMockQuerier {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        // MockQuerier doesn't support Custom, so we ignore it completely here
        let request: QueryRequest<TerraQueryWrapper> = match from_slice(bin_request) {
            Ok(v) => v,
            Err(e) => {
                return SystemResult::Err(SystemError::InvalidRequest {
                    error: format!("Parsing query request: {}", e),
                    request: bin_request.into(),
                })
            }
        };
        self.handle_query(&request)
    }
}

impl WasmMockQuerier {
    pub fn handle_query(&self, request: &QueryRequest<TerraQueryWrapper>) -> QuerierResult {
        match &request {
            QueryRequest::Custom(TerraQueryWrapper { route, query_data }) => {
                if &TerraRoute::Treasury == route {
                    match query_data {
                        TerraQuery::TaxRate {} => {
                            let res = TaxRateResponse {
                                rate: self.tax_querier.rate,
                            };
                            SystemResult::Ok(ContractResult::from(to_binary(&res)))
                        }
                        TerraQuery::TaxCap { denom } => {
                            let cap = self
                                .tax_querier
                                .caps
                                .get(denom)
                                .copied()
                                .unwrap_or_default();
                            let res = TaxCapResponse { cap };
                            SystemResult::Ok(ContractResult::from(to_binary(&res)))
                        }
                        _ => panic!("DO NOT ENTER HERE"),
                    }
                } else {
                    panic!("DO NOT ENTER HERE")
                }
            }
            QueryRequest::Wasm(WasmQuery::Smart { contract_addr, msg }) => {
                match from_binary(&msg) {
                    Ok(CustomTreasuryQueryMsg::Config {}) => {
                        if contract_addr.clone() == self.custom_treasury {
                            SystemResult::Ok(ContractResult::from(to_binary(
                                &CustomTreasuryConfigResponse {
                                    payout_token: self.payout_token.clone(),
                                    policy: String::from("policy"),
                                },
                            )))
                        } else {
                            panic!("DO NOT ENTER HERE")
                        }
                    }

                    _ => match from_binary(&msg) {
                        Ok(Cw20QueryMsg::TokenInfo {}) => {
                            let total_supply: Uint128 = if let Some(supply) =
                                self.token_querier.supplies.get(contract_addr)
                            {
                                supply.clone()
                            } else {
                                Uint128::zero()
                            };
                            let decimals: u8 = if let Some(decimals) =
                                self.token_querier.decimals.get(contract_addr)
                            {
                                decimals.clone()
                            } else {
                                6u8
                            };
                            SystemResult::Ok(ContractResult::from(to_binary(&TokenInfoResponse {
                                name: "mock_name".to_string(),
                                symbol: "mock_symbol".to_string(),
                                decimals: decimals.clone(),
                                total_supply: total_supply.clone(),
                            })))
                        }
                        _ => panic!("DO NOT ENTER HERE"),
                    },
                }
            }
            _ => self.base.handle_query(request),
        }
    }
}

impl WasmMockQuerier {
    pub fn new(base: MockQuerier<TerraQueryWrapper>) -> Self {
        WasmMockQuerier {
            base,
            tax_querier: TaxQuerier::default(),
            token_querier: TokenQuerier::default(),
            custom_treasury: String::default(),
            payout_token: String::default(),
        }
    }

    // configure the tax mock querier
    pub fn with_tax(&mut self, rate: Decimal, caps: &[(&String, &Uint128)]) {
        self.tax_querier = TaxQuerier::new(rate, caps);
    }

    pub fn with_token_info(
        &mut self,
        supplies: &[(&String, &Uint128)],
        decimals: &[(&String, &u8)],
    ) {
        self.token_querier = TokenQuerier::new(supplies, decimals);
    }

    pub fn with_custom_treasury(&mut self, custom_treasury: String, payout_token: String) {
        self.custom_treasury = custom_treasury;
        self.payout_token = payout_token;
    }
}
