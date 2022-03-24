use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::serde::{Serialize, Deserialize};
use near_sdk::json_types::{WrappedTimestamp, U128};
use near_sdk::{PromiseResult, serde_json, env, Gas, ext_contract, near_bindgen, AccountId, PanicOnDefault, Promise};
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

const NO_DEPOSIT: Balance = 0;
const GAS_FOR_RESOLVE_TRANSFER: Gas = 5_000_000_000_000;
const GAS_FOR_FT_TRANSFER_CALL: Gas = 25_000_000_000_000 + GAS_FOR_RESOLVE_TRANSFER;

// #[ext_contract(fpo)]
// trait FPO {
//     fn get_entry(&self, pair: String, provider: AccountId) -> Promise;
//     fn get_prices(&self, pairs: Vec<String>, providers: Vec<AccountId>) -> Promise;
//     fn get_price(&self, pair: String, provider: AccountId) -> Promise;
//     fn aggregate_avg(
//         &self,
//         pairs: Vec<String>,
//         providers: Vec<AccountId>,
//         min_last_update: WrappedTimestamp,
//     ) -> Promise;
//     fn aggregate_median(
//         &self,
//         pairs: Vec<String>,
//         providers: Vec<AccountId>,
//         min_last_update: WrappedTimestamp,
//     ) -> Promise;
//     fn aggregate_collect(
//         &self,
//         pairs: Vec<String>,
//         providers: Vec<AccountId>,
//         min_last_update: WrappedTimestamp,
//     ) -> Promise;
// }

#[ext_contract(fpo)]
trait FPO {
    fn get_price(&self, pair: String, provider: AccountId) -> Option<U128>;
    fn get_price_call(
        &self,
        pair: String,
        provider: AccountId,
        receiver_id: AccountId,
    ) -> Promise ;
    fn get_prices_call(
        &self,
        pairs: Vec<String>,
        providers: Vec<AccountId>,
        receiver_id: AccountId,
    ) -> Promise;
    fn aggregate_avg_call(
        &self,
        pairs: Vec<String>,
        providers: Vec<AccountId>,
        min_last_update: WrappedTimestamp,
        receiver_id: AccountId,
    ) -> Promise;
    fn aggregate_median_call(
        &self,
        pairs: Vec<String>,
        providers: Vec<AccountId>,
        min_last_update: WrappedTimestamp,
        receiver_id: AccountId,
    ) -> Promise ;
    fn aggregate_collect_call(
        &self,
        pairs: Vec<String>,
        providers: Vec<AccountId>,
        min_last_update: WrappedTimestamp,
        receiver_id: AccountId,
    ) -> Promise;
}

#[ext_contract(ext_self)]
trait RequestResolver {
    // fn set_entry(&self, pair: String, provider: AccountId) -> Promise;
    // fn set_collection(&mut self, pairs: Vec<String>, providers: Vec<AccountId>) -> PriceEntry;
    fn on_price_received(
        &self,
        sender_id: AccountId,
        pairs: Vec<String>,
        providers: Vec<AccountId>,
        price_type: PriceType,
        results: Vec<Option<U128>>,
    ) -> Promise;
    fn my_callback(&self) -> Option<U128> ;
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
    // payment_token: AccountId,
    providers: LookupMap<AccountId, Provider>, // maps:  AccountId => Provider
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Debug)]
pub enum PriceType {
    Single,
    Multiple,
    Mean,
    Median,
    Collect, // same as multiple but with min_last_update
}

#[near_bindgen]
impl Requester {
    #[init]
    pub fn new(oracle: AccountId) -> Self {
        Self {
            oracle,
            // payment_token,
            providers: LookupMap::new("p".as_bytes()),
        }
    }
    // pub fn set_entry(&mut self, 
    //         pair: String, 
    //         provider: AccountId) -> PriceEntry {
    //         assert_self();
    //         assert_prev_promise_successful();

    //         let entry = match env::promise_result(0) {
    //             PromiseResult::NotReady => unreachable!(),
    //             PromiseResult::Successful(value) => {
    //                 match serde_json::from_slice::<PriceEntry>(&value) {
    //                     Ok(value) => value,
    //                     Err(_e) => panic!("ERR_INVALID_ENTRY"),
    //                 }
    //             },
    //             PromiseResult::Failed => panic!("ERR_FAILED_ENTRY_FETCH"),
    //         };

    //         let provider_account_id = provider.clone();

    //         let mut provider = self.providers.get(&provider).unwrap_or(Provider::new());
    //         provider.set_pair(&pair, &entry);
    //         self.providers.insert(&provider_account_id, &provider);
    //         entry
    // }
    // pub fn get_entry(
    //     &mut self, 
    //     pair: String, 
    //     provider: AccountId
    // ) -> Promise {
    //     fpo::get_entry(
    //         pair.clone(), 
    //         provider.clone(),
    //         &self.oracle, 
    //         NO_DEPOSIT, 
    //         env::prepaid_gas() - GAS_FOR_FT_TRANSFER_CALL
    //     )
    //     .then(
    //         ext_self::set_entry(
    //             pair,
    //             provider,
    //             &env::current_account_id(), 
    //             NO_DEPOSIT, 
    //             GAS_FOR_RESOLVE_TRANSFER
    //         )
    //     )
    // }

    pub fn get_price_call(
        &self,
        pair: String,
        provider: AccountId,
    ) -> Promise {
        fpo::get_price_call(
            pair.clone(), 
            provider.clone(),
            env::predecessor_account_id(),
            &self.oracle.clone(), 
             NO_DEPOSIT, 
            GAS_FOR_RESOLVE_TRANSFER
        ).then(
            ext_self::my_callback(
                &env::predecessor_account_id(), // this contract's account id
                0, // yocto NEAR to attach to the callback
                5_000_000_000_000 // gas to attach to the callback
            )
        )
            
    }
    
    pub fn get_price(
        &self,
        pair: String,
        provider: AccountId,
    ) -> Promise {
        fpo::get_price(
            pair.clone(), 
            provider.clone(),
            &self.oracle.clone(), 
             NO_DEPOSIT, 
            GAS_FOR_RESOLVE_TRANSFER
        ).then(
            ext_self::my_callback(
                &env::predecessor_account_id(), // this contract's account id
                0, // yocto NEAR to attach to the callback
                5_000_000_000_000 // gas to attach to the callback
            )
        )
            
    }
    pub fn my_callback(&self) -> Option<U128> {
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Failed => None,
            PromiseResult::Successful(result) => {
                let price = near_sdk::serde_json::from_slice::<U128>(&result).unwrap();
                Some(price)
            },
        }
    }

    pub fn on_price_received(
        &self,
        sender_id: AccountId,
        pairs: Vec<String>,
        providers: Vec<AccountId>,
        price_type: PriceType,
        results: Vec<Option<U128>>,
    ){
        println!("sender_id: {:?}", sender_id);
        println!("pairs: {:?}", pairs);
        println!("providers: {:?}", providers);
        println!("price_type: {:?}", price_type);
        println!("results: {:?}", results);
        // results

    }
}


