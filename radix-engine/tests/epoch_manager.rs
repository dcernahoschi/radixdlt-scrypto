use radix_engine::engine::{ApplicationError, ModuleError, RuntimeError};
use radix_engine::ledger::create_genesis;
use radix_engine::types::*;
use radix_engine_interface::core::NetworkDefinition;
use radix_engine_interface::data::*;
use radix_engine_interface::modules::auth::AuthAddresses;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;
use transaction::model::{SystemInstruction, SystemTransaction};
use transaction::signing::EcdsaSecp256k1PrivateKey;

#[test]
fn get_epoch_should_succeed() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.compile_and_publish("./tests/blueprints/epoch_manager");

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_function(package_address, "EpochManagerTest", "get_epoch", args![])
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    let epoch: u64 = receipt.output(1);
    assert_eq!(epoch, 1);
}

#[test]
fn next_round_without_supervisor_auth_fails() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.compile_and_publish("./tests/blueprints/epoch_manager");

    // Act
    let round = 9876u64;
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_function(
            package_address,
            "EpochManagerTest",
            "next_round",
            args!(EPOCH_MANAGER, round),
        )
        .call_function(package_address, "EpochManagerTest", "get_epoch", args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(e, RuntimeError::ModuleError(ModuleError::AuthError { .. }))
    });
}

#[test]
fn next_round_with_validator_auth_succeeds() {
    // Arrange
    let rounds_per_epoch = 5u64;
    let genesis = create_genesis(HashSet::new(), 1u64, rounds_per_epoch);
    let mut test_runner = TestRunner::new_with_genesis(true, genesis);

    // Act
    let instructions = vec![SystemInstruction::CallNativeMethod {
        method_ident: NativeMethodIdent {
            receiver: RENodeId::Global(GlobalAddress::System(EPOCH_MANAGER)),
            method_name: "next_round".to_string(),
        },
        args: args!(EPOCH_MANAGER, rounds_per_epoch - 1),
    }
    .into()];
    let receipt = test_runner.execute_transaction(
        SystemTransaction {
            instructions,
            blobs: vec![],
            nonce: 0,
        }
        .get_executable(vec![AuthAddresses::validator_role()]),
    );

    // Assert
    receipt.expect_commit_success();
    let result = receipt.expect_commit();
    assert!(result.next_epoch.is_none());
}

#[test]
fn next_epoch_with_validator_auth_succeeds() {
    // Arrange
    let initial_epoch = 5u64;
    let rounds_per_epoch = 2u64;
    let genesis = create_genesis(HashSet::new(), initial_epoch, rounds_per_epoch);
    let mut test_runner = TestRunner::new_with_genesis(true, genesis);

    // Act
    let instructions = vec![SystemInstruction::CallNativeMethod {
        method_ident: NativeMethodIdent {
            receiver: RENodeId::Global(GlobalAddress::System(EPOCH_MANAGER)),
            method_name: "next_round".to_string(),
        },
        args: args!(EPOCH_MANAGER, rounds_per_epoch),
    }
    .into()];
    let receipt = test_runner.execute_transaction(
        SystemTransaction {
            instructions,
            blobs: vec![],
            nonce: 0,
        }
        .get_executable(vec![AuthAddresses::validator_role()]),
    );

    // Assert
    receipt.expect_commit_success();
    let result = receipt.expect_commit();
    let next_epoch = result
        .next_epoch
        .as_ref()
        .expect("Should have next epoch")
        .1;
    assert_eq!(next_epoch, initial_epoch + 1);
}

#[test]
fn register_validator_with_auth_succeeds() {
    // Arrange
    let initial_epoch = 5u64;
    let rounds_per_epoch = 2u64;
    let genesis = create_genesis(HashSet::new(), initial_epoch, rounds_per_epoch);
    let mut test_runner = TestRunner::new_with_genesis(true, genesis);
    let (pub_key, _, _) = test_runner.new_allocated_account();

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .register_validator(pub_key)
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleAddress::from_public_key(&pub_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn register_validator_without_auth_fails() {
    // Arrange
    let initial_epoch = 5u64;
    let rounds_per_epoch = 2u64;
    let genesis = create_genesis(HashSet::new(), initial_epoch, rounds_per_epoch);
    let mut test_runner = TestRunner::new_with_genesis(true, genesis);
    let (pub_key, _, _) = test_runner.new_allocated_account();

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .register_validator(pub_key)
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::AuthZoneError(..))
        )
    });
}

#[test]
fn unregister_validator_with_auth_succeeds() {
    // Arrange
    let initial_epoch = 5u64;
    let rounds_per_epoch = 2u64;
    let genesis = create_genesis(HashSet::new(), initial_epoch, rounds_per_epoch);
    let mut test_runner = TestRunner::new_with_genesis(true, genesis);
    let (pub_key, _, _) = test_runner.new_allocated_account();

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .unregister_validator(pub_key)
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleAddress::from_public_key(&pub_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn unregister_validator_without_auth_fails() {
    // Arrange
    let initial_epoch = 5u64;
    let rounds_per_epoch = 2u64;
    let genesis = create_genesis(HashSet::new(), initial_epoch, rounds_per_epoch);
    let mut test_runner = TestRunner::new_with_genesis(true, genesis);
    let (pub_key, _, _) = test_runner.new_allocated_account();

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .unregister_validator(pub_key)
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::AuthZoneError(..))
        )
    });
}

#[test]
fn registered_validator_becomes_part_of_validator_on_epoch_change() {
    // Arrange
    let initial_epoch = 5u64;
    let rounds_per_epoch = 2u64;
    let genesis = create_genesis(HashSet::new(), initial_epoch, rounds_per_epoch);
    let mut test_runner = TestRunner::new_with_genesis(true, genesis);
    let (pub_key, _, _) = test_runner.new_allocated_account();
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .register_validator(pub_key)
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleAddress::from_public_key(&pub_key)],
    );
    receipt.expect_commit_success();

    // Act
    let instructions = vec![SystemInstruction::CallNativeMethod {
        method_ident: NativeMethodIdent {
            receiver: RENodeId::Global(GlobalAddress::System(EPOCH_MANAGER)),
            method_name: "next_round".to_string(),
        },
        args: args!(EPOCH_MANAGER, rounds_per_epoch),
    }
    .into()];
    let receipt = test_runner.execute_transaction(
        SystemTransaction {
            instructions,
            blobs: vec![],
            nonce: 0,
        }
        .get_executable(vec![AuthAddresses::validator_role()]),
    );

    // Assert
    receipt.expect_commit_success();
    let result = receipt.expect_commit();
    let next_epoch = result.next_epoch.as_ref().expect("Should have next epoch");
    assert_eq!(next_epoch.1, initial_epoch + 1);
    assert!(next_epoch.0.contains(&pub_key));
}

#[test]
fn unregistered_validator_gets_removed_on_epoch_change() {
    // Arrange
    let initial_epoch = 5u64;
    let rounds_per_epoch = 2u64;
    let pub_key = EcdsaSecp256k1PrivateKey::from_u64(1u64).unwrap().public_key();
    let mut validator_set = HashSet::new();
    validator_set.insert(pub_key);
    let genesis = create_genesis(validator_set, initial_epoch, rounds_per_epoch);
    let mut test_runner = TestRunner::new_with_genesis(true, genesis);
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .unregister_validator(pub_key)
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleAddress::from_public_key(&pub_key)],
    );
    receipt.expect_commit_success();

    // Act
    let instructions = vec![SystemInstruction::CallNativeMethod {
        method_ident: NativeMethodIdent {
            receiver: RENodeId::Global(GlobalAddress::System(EPOCH_MANAGER)),
            method_name: "next_round".to_string(),
        },
        args: args!(EPOCH_MANAGER, rounds_per_epoch),
    }
        .into()];
    let receipt = test_runner.execute_transaction(
        SystemTransaction {
            instructions,
            blobs: vec![],
            nonce: 0,
        }
            .get_executable(vec![AuthAddresses::validator_role()]),
    );

    // Assert
    receipt.expect_commit_success();
    let result = receipt.expect_commit();
    let next_epoch = result.next_epoch.as_ref().expect("Should have next epoch");
    assert_eq!(next_epoch.1, initial_epoch + 1);
    assert!(!next_epoch.0.contains(&pub_key));
}


#[test]
fn epoch_manager_create_should_fail_with_supervisor_privilege() {
    // Arrange
    let mut test_runner = TestRunner::new(true);

    // Act
    let instructions = vec![SystemInstruction::CallNativeFunction {
        function_ident: NativeFunctionIdent {
            blueprint_name: EPOCH_MANAGER_BLUEPRINT.to_owned(),
            function_name: EpochManagerFunction::Create.as_ref().to_owned(),
        },
        args: scrypto_encode(&EpochManagerCreateInvocation {
            validator_set: HashSet::new(),
            initial_epoch: 1u64,
            rounds_per_epoch: 1u64,
        })
        .unwrap(),
    }
    .into()];
    let blobs = vec![];
    let receipt = test_runner.execute_transaction(
        SystemTransaction {
            instructions,
            blobs,
            nonce: 0,
        }
        .get_executable(vec![]),
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(e, RuntimeError::ModuleError(ModuleError::AuthError { .. }))
    });
}

#[test]
fn epoch_manager_create_should_succeed_with_system_privilege() {
    // Arrange
    let mut test_runner = TestRunner::new(true);

    // Act
    let instructions = vec![SystemInstruction::CallNativeFunction {
        function_ident: NativeFunctionIdent {
            blueprint_name: EPOCH_MANAGER_BLUEPRINT.to_owned(),
            function_name: EpochManagerFunction::Create.as_ref().to_owned(),
        },
        args: scrypto_encode(&EpochManagerCreateInvocation {
            validator_set: HashSet::new(),
            initial_epoch: 1u64,
            rounds_per_epoch: 1u64,
        })
        .unwrap(),
    }
    .into()];
    let blobs = vec![];
    let receipt = test_runner.execute_transaction(
        SystemTransaction {
            instructions,
            blobs,
            nonce: 0,
        }
        .get_executable(vec![AuthAddresses::system_role()]),
    );

    // Assert
    receipt.expect_commit_success();
}
