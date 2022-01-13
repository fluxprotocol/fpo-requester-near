use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::serde::{Serialize, Deserialize};
use near_sdk::json_types::{WrappedTimestamp, U128};
use near_sdk::{env, near_bindgen, AccountId, PanicOnDefault, Promise, PromiseOrValue};
use flux_sdk::{WrappedBalance};
use near_sdk::serde_json::json;
mod fungible_token_handler;
use fungible_token_handler::{fungible_token, ENTRY_GAS};
use near_sdk::{Balance};
near_sdk::setup_alloc!();

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
pub struct PriceEntry {
    price: U128,                   // Last reported price
    decimals: u16,                 // Amount of decimals (e.g. if 2, 100 = 1.00)
    last_update: WrappedTimestamp, // Time or report
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
pub struct Outcome {
    entry: Option<Vec<PriceEntry>>,
    refund: Balance
}

pub struct ResponsePayload {
    method: String,
    pairs: Vec<String>,
    providers: Vec<AccountId>,
    outcome: Outcome,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Provider {
    pub pairs: LookupMap<String, PriceEntry>, // Maps "{TICKER_1}/{TICKER_2}" => PriceEntry - e.g.: ETHUSD => PriceEntry
}

impl Provider {
    pub fn new() -> Self {
        Self {
            pairs: LookupMap::new("ps".as_bytes()),
        }
    }
    pub fn set_pair(&mut self, pair: String, entry: PriceEntry) {
        self.pairs.insert(&pair, &entry);
    }
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Requester {
    oracle: AccountId,
    payment_token: AccountId,
    providers: LookupMap<AccountId, Provider>, // maps:  AccountId => Provider
}

impl Requester {
    fn assert_oracle(&self) {
        assert_eq!(
            &env::predecessor_account_id(),
            &self.oracle,
            "ERR_INVALID_ORACLE_ADDRESS"
        );
    }
}

#[near_bindgen]
impl Requester {
    #[init]
    pub fn new(oracle: AccountId, payment_token: AccountId) -> Self {
        Self {
            oracle,
            payment_token,
            providers: LookupMap::new("p".as_bytes()),
        }
    }
    pub fn set_outcome(&mut self, payload: ResponsePayload) -> PromiseOrValue<u128>
     {
        self.assert_oracle();
        // return refund to user from outcome
        match payload.method.as_ref() {
            "get_entry" => {
                let entry = payload.outcome.entry.unwrap()[0];
                let provider = self.providers.get(&payload.providers[0]).unwrap_or(Provider::new());
                provider.set_pair(payload.pairs[0], entry);
                self.providers.insert(&payload.providers[0], &provider);
            }
            "aggregate_avg" => {
                let entry = payload.outcome.entry.unwrap()[0];
                let provider = self.providers.get(&payload.providers[0]).unwrap_or(Provider::new());
                let pair_agg_name = payload.pairs[0].to_owned();
                pair_agg_name.push_str(&"AGG".to_owned()); 
                provider.set_pair(pair_agg_name, entry);
                self.providers.insert(&payload.providers[0], &provider);

            }
            "aggregate_collect" => {
                    let entries = payload.outcome.entry.unwrap();
                    for i in 0..payload.providers.len() {
                    let mut provider = self
                        .providers
                        .get(&payload.providers[i])
                        .unwrap_or(Provider::new());
                    provider.pairs.insert(&payload.pairs[i], &entries[i]);
                }
            }
        }
        PromiseOrValue::Value(0)
    }
    #[payable]
    pub fn get_entry(&mut self, 
            pair: String, 
            provider: AccountId, 
            amount: WrappedBalance, 
            min_last_update: WrappedTimestamp) -> Promise {
        // TODO if min_last_update within block, return most recent value
        fungible_token::ft_transfer_call(
            self.oracle.clone(),
            amount,
            None,
            json!({ "method": "get_entry", "pairs": [pair], "providers": [provider], "min_last_update": min_last_update }).to_string(),
            &self.payment_token,
            1,
            ENTRY_GAS,
        )
    }
    #[payable]
    pub fn aggregate_avg(&mut self, 
            pairs: Vec<String>, 
            providers: Vec<AccountId>, 
            min_last_update: WrappedTimestamp, 
            amount: WrappedBalance) -> Promise {
        // TODO if min_last_update within block, return most recent value
        fungible_token::ft_transfer_call(
            self.oracle.clone(),
            amount,
            None,
            json!({ "method": "aggregate_avg", "pairs": pairs, "providers": providers, "min_last_update": min_last_update }).to_string(),
            &self.payment_token,
            1,
            ENTRY_GAS,
        )
    }
    #[payable]
    pub fn aggregate_collect(&mut self, 
        pairs: Vec<String>, 
        providers: Vec<AccountId>, 
        min_last_update: WrappedTimestamp, 
        amount: WrappedBalance) -> Promise {
        // TODO if min_last_update within block, return most recent value
        fungible_token::ft_transfer_call(
            self.oracle.clone(),
            amount,
            None,
            json!({ "method": "aggregate_collect", "pairs": pairs, "providers": providers, "min_last_update": min_last_update }).to_string(),
            &self.payment_token,
            1,
            ENTRY_GAS,
        )
    }
}