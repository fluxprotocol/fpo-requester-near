use near_fpo::FPOContractContract;
use requester::RequesterContract;
use near_sdk::json_types::U128;
pub use near_sdk::json_types::{Base64VecU8, ValidAccountId, WrappedDuration, U64};
use near_sdk_sim::{call, deploy, init_simulator, to_yocto, ContractAccount, UserAccount};

near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    FPO_BYTES => "res/near_fpo.wasm",
    REQUESTER_BYTES => "res/requester.wasm"
}

pub const DEFAULT_GAS: u64 = 300_000_000_000_000;

fn init() -> (UserAccount, ContractAccount<FPOContractContract>, ContractAccount<RequesterContract>) {
    let root = init_simulator(None);
    // Deploy the compiled Wasm bytes
    let fpo: ContractAccount<FPOContractContract> = deploy!(
        contract: FPOContractContract,
        contract_id: "nearfpo".to_string(),
        bytes: &FPO_BYTES,
        signer_account: root
    );
     // Deploy the compiled Wasm bytes
     let requester: ContractAccount<RequesterContract> = deploy!(
        contract: RequesterContract,
        contract_id: "requester".to_string(),
        bytes: &REQUESTER_BYTES,
        signer_account: root
    );

    (root, fpo, requester)
}

#[test]
fn simulate_create_pair() {
    let (root, fpo, requester) = init();

    let user1 = root.create_user("user1".to_string(), to_yocto("1000000"));
    let user2 = root.create_user("user2".to_string(), to_yocto("1000000"));
    call!(user1, fpo.new()).assert_success();

    // create a price pair, check if it exists, and get the value
    call!(user1, fpo.create_pair("ETH/USD".to_string(), 8, U128(2000))).assert_success();
    call!(
        user1,
        fpo.pair_exists("ETH/USD".to_string(), user1.account_id())
    )
    .assert_success();
    let price_entry = call!(
        user1,
        fpo.get_entry("ETH/USD".to_string(), user1.account_id())
    );

    // output and check the data
    println!(
        "Returned Price: {:?}",
        &price_entry.unwrap_json_value()
    );
    debug_assert_eq!(
        &price_entry.unwrap_json_value()["price"].to_owned(),
        &"2000".to_string()
    );


    println!("fpo.account_id() {:?}", fpo.account_id());
    println!("requester.account_id() {:?}", requester.account_id());


    call!(user2, requester.new(fpo.account_id())).assert_success();


    let price = call!(user2, requester.get_price("ETH/USD".to_string(), user1.account_id()));
    println!("{:?}", price);

    // debug_assert_eq!(
    //     &price.unwrap_json_value()["price"].to_owned(),
    //     &"2000".to_string()
    // );


}

