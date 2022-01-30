use radix_engine::ledger::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

#[test]
fn test_create_additional_admin() {
    // Set up environment.
    let mut ledger = InMemoryLedger::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, 0, 0, true);
    let key = executor.new_public_key();
    let account = executor.new_account(key);
    let package = executor.publish_package(include_code!("flat_admin"));

    // Test the `new` function.
    let transaction1 = TransactionBuilder::new(&executor)
        .call_function(package, "FlatAdmin", "new", vec!["test".to_string()], None)
        .call_method_with_all_resources(account, "deposit_batch")
        .build(vec![key])
        .unwrap();
    let receipt1 = executor.run(transaction1).unwrap();
    println!("{:?}\n", receipt1);
    assert!(receipt1.error.is_none());

    // Test the `create_additional_admin` method.
    let flat_admin = receipt1.component(0).unwrap();
    let admin_badge = receipt1.resource_def(1).unwrap();
    let transaction2 = TransactionBuilder::new(&executor)
        .call_method(
            flat_admin,
            "create_additional_admin",
            vec![format!("1,{}", admin_badge)],
            Some(account),
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build(vec![key])
        .unwrap();
    let receipt2 = executor.run(transaction2).unwrap();
    println!("{:?}\n", receipt2);
    assert!(receipt2.error.is_none());
}
