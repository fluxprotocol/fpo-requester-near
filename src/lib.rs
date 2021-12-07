use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::serde::{Serialize, Deserialize};
use near_sdk::json_types::{WrappedTimestamp, U128};
use near_sdk::{env, near_bindgen, ext_contract, AccountId, BorshStorageKey, PanicOnDefault, Promise};
use flux_sdk::{consts::GAS_BASE_SET_OUTCOME};
near_sdk::setup_alloc!();

#[ext_contract]
pub trait OracleContractExtern {
    fn get_entry(pair: String, user: AccountId);
    fn aggregate_avg(pairs: Vec<String>, users: Vec<AccountId>, min_last_update: WrappedTimestamp);
    fn aggregate_collect(pairs: Vec<String>, users: Vec<AccountId>, min_last_update: WrappedTimestamp);
}

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
            pairs: LookupMap::new(StorageKeys::Provider),
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

#[derive(BorshStorageKey, BorshSerialize)]
enum StorageKeys {
    Providers,
    Provider,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Requester {
    oracle: AccountId,
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
    pub fn new(oracle: AccountId) -> Self {
        Self {
            oracle,
            providers: LookupMap::new(StorageKeys::Providers),
        }
    }

    pub fn set_outcome(&mut self, providers: Vec<AccountId>, pairs: Vec<String>, entries: Vec<PriceEntry>) {
        self.assert_oracle();

        for i in 0..providers.len() {
            let mut provider = self
                .providers
                .get(&providers[i])
                .unwrap_or(Provider::new());
            provider.pairs.insert(&pairs[i], &entries[i]);
        }
        // TODO shall I return these answers?
    }

    pub fn get_entry(&self, pair: String, provider: AccountId) -> Promise {
        oracle_contract_extern::get_entry(
            pair,
            provider,
            &self.oracle,
            env::attached_deposit(),
            GAS_BASE_SET_OUTCOME / 10 // TODO is gas alright
        )
    }
    pub fn aggregate_avg(&self, pairs: Vec<String>, providers: Vec<AccountId>, min_last_update: WrappedTimestamp) -> Promise {
        oracle_contract_extern::aggregate_avg(
            pairs,
            providers,
            min_last_update,
            &self.oracle,
            env::attached_deposit(),
            GAS_BASE_SET_OUTCOME / 10 // TODO is gas alright
        )
    }
    pub fn aggregate_collect(&self, pairs: Vec<String>, providers: Vec<AccountId>, min_last_update: WrappedTimestamp) ->Promise {
        oracle_contract_extern::aggregate_collect(
            pairs,
            providers,
            min_last_update,
            &self.oracle,
            env::attached_deposit(),
            GAS_BASE_SET_OUTCOME / 10 // TODO is gas alright
        )
    }
}