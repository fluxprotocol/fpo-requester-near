use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::serde::{Serialize, Deserialize};
use near_sdk::json_types::{WrappedTimestamp, U128};
use near_sdk::{env, near_bindgen, AccountId, PanicOnDefault, Promise};
use flux_sdk::{WrappedBalance};
use near_sdk::serde_json::json;
mod fungible_token_handler;
use fungible_token_handler::{fungible_token, ENTRY_GAS};
// use near_contract_standards::fungible_token::core_impl;
// use near_contract_standards::fungible_token::FungibleToken;
near_sdk::setup_alloc!();

// near_contract_standards::impl_fungible_token_core!(Requester, token, on_tokens_burned);

// const GAS_BASE_TRANSFER: Gas = 5_000_000_000_000;
// const ENTRY_GAS: Gas = 200_000_000_000_000;

// #[ext_contract]
// pub trait OracleContractExtern {
//     fn get_entry(pair: String, user: AccountId);
//     fn aggregate_avg(pairs: Vec<String>, users: Vec<AccountId>, min_last_update: WrappedTimestamp);
//     fn aggregate_collect(pairs: Vec<String>, users: Vec<AccountId>, min_last_update: WrappedTimestamp);
// }

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
pub struct PriceEntry {
    price: U128,                   // Last reported price
    decimals: u16,                 // Amount of decimals (e.g. if 2, 100 = 1.00)
    last_update: WrappedTimestamp, // Time or report
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
    // pub fn get_entry_expect(&self, pair: &String) -> PriceEntry {
    //     self.pairs
    //         .get(pair)
    //         .expect("no price available for this pair")
    // }
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
            // TODO return to StorageKey implementation
            providers: LookupMap::new("p".as_bytes()),
        }
    }
    pub fn set_outcome(&mut self, providers: Vec<AccountId>, pairs: Vec<String>, entries: Vec<PriceEntry>) -> (Vec<AccountId>, Vec<String>, Vec<PriceEntry>) {
        self.assert_oracle();

        for i in 0..providers.len() {
            let mut provider = self
                .providers
                .get(&providers[i])
                .unwrap_or(Provider::new());
            provider.pairs.insert(&pairs[i], &entries[i]);
        }
        (providers, pairs, entries)
    }
    #[payable]
    pub fn get_entry(&mut self, pair: String, provider: AccountId, amount: WrappedBalance) -> Promise {
        fungible_token::ft_transfer_call(
            self.oracle.clone(),
            amount,
            None,
            json!({ "pair": pair, "provider": provider }).to_string(),
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
        fungible_token::ft_transfer_call(
            self.oracle.clone(),
            amount,
            None,
            json!({ "avg": true, "pairs": pairs, "providers": providers, "min_last_update": min_last_update }).to_string(),
            // Near params
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
        amount: WrappedBalance) ->Promise {
        fungible_token::ft_transfer_call(
            self.oracle.clone(),
            amount,
            None,
            json!({ "avg": false, "pairs": pairs, "providers": providers, "min_last_update": min_last_update }).to_string(),
            // Near params
            &self.payment_token,
            1,
            ENTRY_GAS,
        )
    }
}