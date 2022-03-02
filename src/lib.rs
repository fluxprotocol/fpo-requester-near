use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::serde::{Serialize, Deserialize};
use near_sdk::json_types::{WrappedTimestamp, U128};
use near_sdk::{log, PromiseResult, serde_json, env, Gas, ext_contract, near_bindgen, AccountId, PanicOnDefault, Promise};
use near_sdk::{Balance};
near_sdk::setup_alloc!();

pub fn is_promise_success() -> bool {
    assert_eq!(
        env::promise_results_count(),
        1,
        "Contract expected a result on the callback"
    );
    matches!(env::promise_result(0), PromiseResult::Successful(_))
}

pub fn assert_prev_promise_successful() {
    assert_eq!(is_promise_success(), true, "previous promise failed");
}

pub fn assert_self() {
    assert_eq!(
        env::predecessor_account_id(),
        env::current_account_id(),
        "Method is private"
    );
}

// const NO_DEPOSIT: Balance = 0;
const BASE_GAS: Gas = 100_000_000_000_000;

#[ext_contract(fpo)]
trait FPO {
    fn get_entry(&self, pair: String, provider: AccountId) -> Promise;
    // fn aggregate_avg(
    //     &self,
    //     pairs: Vec<String>,
    //     providers: Vec<AccountId>,
    //     min_last_update: WrappedTimestamp,
    // ) -> PromiseOrValue<U128>;
    // fn aggregate_collect(
    //     &self,
    //     pairs: Vec<String>,
    //     providers: Vec<AccountId>,
    //     min_last_update: WrappedTimestamp,
    // ) -> PromiseOrValue<Vec<Option<U128>>>;
}

#[ext_contract(ext_self)]
trait RequestResolver {
    fn set_entry(&self, pair: String, provider: AccountId) -> Promise;
}

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

// pub struct ResponsePayload {
//     method: String,
//     pairs: Vec<String>,
//     providers: Vec<AccountId>,
//     outcome: Outcome,
// }

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
    pub fn set_pair(&mut self, pair: &String, entry: &PriceEntry) {
        self.pairs.insert(pair, entry);
    }
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Requester {
    oracle: AccountId,
    payment_token: AccountId,
    providers: LookupMap<AccountId, Provider>, // maps:  AccountId => Provider
}

// impl Requester {
//     fn assert_oracle(&self) {
//         assert_eq!(
//             &env::predecessor_account_id(),
//             &self.oracle,
//             "ERR_INVALID_ORACLE_ADDRESS"
//         );
//     }
// }

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
    // pub fn set_outcome(&mut self, payload: ResponsePayload) -> PromiseOrValue<u128> {
    //     self.assert_oracle();
    //     // return refund to user from outcome
    //     match payload.method.as_ref() {
    //         "get_entry" => {
    //             let entry = payload.outcome.entry.unwrap()[0];
    //             let provider = self.providers.get(&payload.providers[0]).unwrap_or(Provider::new());
    //             provider.set_pair(&payload.pairs[0], &entry);
    //             self.providers.insert(&payload.providers[0], &provider);
    //         }
    //         "aggregate_avg" => {
    //             let entry = payload.outcome.entry.unwrap()[0];
    //             let provider = self.providers.get(&payload.providers[0]).unwrap_or(Provider::new());
    //             let pair_agg_name = payload.pairs[0].to_owned();
    //             pair_agg_name.push_str(&"AGG".to_owned()); 
    //             provider.set_pair(&pair_agg_name, &entry);
    //             self.providers.insert(&payload.providers[0], &provider);

    //         }
    //         "aggregate_collect" => {
    //             let entries = payload.outcome.entry.unwrap();
    //             for i in 0..payload.providers.len() {
    //                 let mut provider = self
    //                     .providers
    //                     .get(&payload.providers[i])
    //                     .unwrap_or(Provider::new());
    //                 provider.pairs.insert(&payload.pairs[i], &entries[i]);
    //             }
    //         }
    //     }
    //     PromiseOrValue::Value(0)
    // }
    pub fn set_entry(&mut self, 
            pair: String, 
            provider: AccountId) {
            assert_self();
            assert_prev_promise_successful();

            let entry = match env::promise_result(0) {
                PromiseResult::NotReady => unreachable!(),
                PromiseResult::Successful(value) => {
                    match serde_json::from_slice::<PriceEntry>(&value) {
                        Ok(value) => value,
                        Err(_e) => panic!("ERR_INVALID_ENTRY"),
                    }
                },
                PromiseResult::Failed => panic!("ERR_FAILED_ENTRY_FETCH"),
            };

            let provider_account_id = provider.clone();

            let mut provider = self.providers.get(&provider).unwrap_or(Provider::new());
            provider.set_pair(&pair, &entry);
            self.providers.insert(&provider_account_id, &provider);
    }
    pub fn find_entry(
        &mut self, 
        pair: String, 
        provider: AccountId
    ) -> Promise {
        fpo::get_entry(
                pair.clone(), 
                provider.clone(),
                &self.oracle, 
                0, 
                BASE_GAS
            )
            // .then(
            //     ext_self::set_entry(
            //     pair,
            //     provider,
            //     &env::current_account_id(), 
            //     0, 
            //     BASE_GAS / 2
            // )
        // )
    }
    // #[payable]
    // pub fn aggregate_avg(&mut self, 
    //         pairs: Vec<String>, 
    //         providers: Vec<AccountId>, 
    //         min_last_update: WrappedTimestamp) -> Promise {
    //     fpo::aggregate_avg(pairs, providers, min_last_update, &self.oracle, NO_DEPOSIT, BASE_GAS)
    // }
    // #[payable]
    // pub fn aggregate_collect(&mut self, 
    //     pairs: Vec<String>, 
    //     providers: Vec<AccountId>, 
    //     min_last_update: WrappedTimestamp, 
    //     amount: WrappedBalance) -> Promise {
    //     fpo::aggregate_collect(pairs, providers, min_last_update, &self.oracle, NO_DEPOSIT, BASE_GAS)
    // }
}