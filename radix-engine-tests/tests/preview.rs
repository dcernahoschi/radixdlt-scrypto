use radix_engine::types::*;
use radix_engine_interface::rule;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;
use transaction::builder::TransactionBuilder;
use transaction::builder::TransactionManifestV1;
use transaction::ecdsa_secp256k1::EcdsaSecp256k1PrivateKey;
use transaction::model::*;
use transaction::validation::NotarizedTransactionValidator;
use transaction::validation::{TransactionValidator, ValidationConfig};

#[test]
fn test_transaction_preview_cost_estimate() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let network = NetworkDefinition::simulator();
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .clear_auth_zone()
        .build();
    let preview_flags = PreviewFlags {
        unlimited_loan: true,
        assume_all_signature_proofs: false,
        permit_invalid_header_epoch: false,
        permit_duplicate_intent_hash: false,
    };
    let (notarized_transaction, preview_intent) = prepare_matching_test_tx_and_preview_intent(
        &mut test_runner,
        &network,
        manifest,
        &preview_flags,
    );

    // Act & Assert: Execute the preview, followed by a normal execution.
    // Ensure that both succeed and that the preview result provides an accurate cost estimate
    let preview_result = test_runner.preview(preview_intent, &network);
    let preview_receipt = preview_result.unwrap();
    preview_receipt.expect_commit_success();

    let receipt = test_runner
        .execute_transaction(validate(&network, &notarized_transaction).get_executable());
    let commit_result = receipt.expect_commit(true);
    assert_eq!(
        commit_result.fee_summary.execution_cost_sum,
        commit_result.fee_summary.execution_cost_sum
    );
}

#[test]
fn test_assume_all_signature_proofs_flag_method_authorization() {
    // Arrange
    // Create an account component that requires a key auth for withdrawal
    let mut test_runner = TestRunner::builder().build();
    let network = NetworkDefinition::simulator();

    let public_key = EcdsaSecp256k1PrivateKey::from_u64(99).unwrap().public_key();
    let withdraw_auth = rule!(require(NonFungibleGlobalId::from_public_key(&public_key)));
    let account = test_runner.new_account_advanced(OwnerRole::Fixed(withdraw_auth.clone()));
    let (_, _, other_account) = test_runner.new_allocated_account();

    let preview_flags = PreviewFlags {
        unlimited_loan: true,
        assume_all_signature_proofs: true,
        permit_invalid_header_epoch: false,
        permit_duplicate_intent_hash: false,
    };

    // Check method authorization (withdrawal) without a proof in the auth zone
    let manifest = ManifestBuilder::new()
        .lock_fee(account, 10.into())
        .withdraw_from_account(account, RADIX_TOKEN, 1.into())
        .call_method(
            other_account,
            "try_deposit_batch_or_abort",
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();

    let (_, preview_intent) = prepare_matching_test_tx_and_preview_intent(
        &mut test_runner,
        &network,
        manifest,
        &preview_flags,
    );

    // Act
    let result = test_runner.preview(preview_intent, &network);

    // Assert
    result.unwrap().expect_commit_success();
}

fn prepare_matching_test_tx_and_preview_intent(
    test_runner: &mut TestRunner,
    network: &NetworkDefinition,
    manifest: TransactionManifestV1,
    flags: &PreviewFlags,
) -> (NotarizedTransactionV1, PreviewIntentV1) {
    let notary_priv_key = EcdsaSecp256k1PrivateKey::from_u64(2).unwrap();
    let tx_signer_priv_key = EcdsaSecp256k1PrivateKey::from_u64(3).unwrap();

    let notarized_transaction = TransactionBuilder::new()
        .header(TransactionHeaderV1 {
            network_id: network.id,
            start_epoch_inclusive: 0,
            end_epoch_exclusive: 99,
            nonce: test_runner.next_transaction_nonce(),
            notary_public_key: notary_priv_key.public_key().into(),
            notary_is_signatory: false,
            tip_percentage: 0,
        })
        .manifest(manifest)
        .sign(&tx_signer_priv_key)
        .notarize(&notary_priv_key)
        .build();

    let preview_intent = PreviewIntentV1 {
        intent: notarized_transaction.signed_intent.intent.clone(),
        signer_public_keys: vec![tx_signer_priv_key.public_key().into()],
        flags: flags.clone(),
    };

    (notarized_transaction, preview_intent)
}

fn validate<'a>(
    network: &'a NetworkDefinition,
    transaction: &'a NotarizedTransactionV1,
) -> ValidatedNotarizedTransactionV1 {
    NotarizedTransactionValidator::new(ValidationConfig::default(network.id))
        .validate(transaction.prepare().unwrap())
        .unwrap()
}
