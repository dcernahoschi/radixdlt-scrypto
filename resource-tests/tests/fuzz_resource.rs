use radix_engine::blueprints::consensus_manager::EpochChangeEvent;
use radix_engine::transaction::{TransactionOutcome, TransactionReceipt};
use radix_engine::types::*;
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;
use native_sdk::modules::metadata::Metadata;
use native_sdk::modules::role_assignment::RoleAssignment;
use native_sdk::resource::{NativeVault, ResourceManager};
use radix_engine::errors::RuntimeError;
use radix_engine::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use radix_engine::system::system_callback::SystemLockData;
use radix_engine::vm::{OverridePackageCode, VmInvoke};
use radix_engine_interface::blueprints::package::PackageDefinition;
use radix_engine_stores::memory_db::InMemorySubstateDatabase;
use resource_tests::ResourceTestFuzzer;
use scrypto_unit::*;
use transaction::prelude::*;

#[test]
fn fuzz_resource() {
    let results: Vec<BTreeMap<ResourceFuzzAction, BTreeMap<ConsensusFuzzActionResult, u64>>> =
        (1u64..64u64)
            .into_par_iter()
            .map(|seed| {
                let mut resource_fuzz_test = ResourceFuzzTest::new(seed);
                resource_fuzz_test.run_fuzz()
            })
            .collect();

    println!("{:#?}", results);

    panic!("oops");
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, FromRepr, Ord, PartialOrd, Eq, PartialEq)]
enum ResourceFuzzStartAction {
    Mint,
    VaultTake,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, FromRepr, Ord, PartialOrd, Eq, PartialEq)]
enum ResourceFuzzEndAction {
    Burn,
    VaultPut,
}

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
struct ResourceFuzzAction(ResourceFuzzStartAction, ResourceFuzzEndAction);


#[repr(u8)]
#[derive(Copy, Clone, Debug, FromRepr, Ord, PartialOrd, Eq, PartialEq)]
enum ConsensusFuzzActionResult {
    TrivialSuccess,
    Success,
    TrivialFailure,
    Failure,
}

const BLUEPRINT_NAME: &str = "MyBlueprint";
const CUSTOM_PACKAGE_CODE_ID: u64 = 1024;

#[derive(Clone)]
struct TestInvoke;
impl VmInvoke for TestInvoke {
    fn invoke<Y>(
        &mut self,
        export_name: &str,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
        where
            Y: ClientApi<RuntimeError> + KernelNodeApi + KernelSubstateApi<SystemLockData>,
    {
        match export_name {
            "call_vault" => {
                let handle = api.actor_open_field(
                    ACTOR_STATE_SELF,
                    0u8,
                    LockFlags::read_only(),
                ).unwrap();
                let vault: Vault = api.field_read_typed(handle).unwrap();

                let input: (String, ScryptoValue) = scrypto_decode(input.as_slice()).unwrap();

                let rtn = api.call_method(vault.0.as_node_id(), input.0.as_str(), scrypto_encode(&input.1).unwrap())?;
                return Ok(IndexedScryptoValue::from_vec(rtn).unwrap());
            }
            "new" => {
                let resource_address: (ResourceAddress,) = scrypto_decode(input.as_slice()).unwrap();
                let vault = Vault::create(resource_address.0, api).unwrap();

                let metadata = Metadata::create(api)?;
                let access_rules = RoleAssignment::create(OwnerRole::None, btreemap!(), api)?;
                let node_id = api.new_simple_object(BLUEPRINT_NAME, btreemap!(0u8 => FieldValue::new(&vault)))?;

                api.globalize(
                    node_id,
                    btreemap!(
                            ModuleId::Metadata => metadata.0,
                            ModuleId::RoleAssignment => access_rules.0.0,
                        ),
                    None,
                )?;
            }
            _ => {}
        }

        Ok(IndexedScryptoValue::from_typed(&()))
    }
}

struct ResourceFuzzTest {
    fuzzer: ResourceTestFuzzer,
    test_runner: TestRunner<OverridePackageCode<TestInvoke>, InMemorySubstateDatabase>,
    resource_address: ResourceAddress,
    component_address: ComponentAddress,
    account_public_key: PublicKey,
    account_component_address: ComponentAddress,
}

impl ResourceFuzzTest {
    fn new(seed: u64) -> Self {
        let fuzzer = ResourceTestFuzzer::new(seed);
        let mut test_runner = TestRunnerBuilder::new()
            .with_custom_extension(OverridePackageCode::new(CUSTOM_PACKAGE_CODE_ID, TestInvoke))
            .build();
        let package_address = test_runner.publish_native_package(
            CUSTOM_PACKAGE_CODE_ID,
            PackageDefinition::new_with_field_test_definition(
                BLUEPRINT_NAME,
                vec![("call_vault", "call_vault", true), ("new", "new", false)],
            ),
        );

        let (public_key, _, account) = test_runner.new_account(false);

        let resource_address = test_runner.create_freely_mintable_and_burnable_fungible_resource(
            OwnerRole::None,
            None,
            18u8,
            account,
        );

        let receipt = test_runner.execute_manifest(
            ManifestBuilder::new()
                .lock_fee(test_runner.faucet_component(), 500u32)
                .call_function(package_address, BLUEPRINT_NAME, "new", manifest_args!(resource_address))
                .build(),
            vec![],
        );
        let component_address = receipt.expect_commit_success().new_component_addresses()[0];

        Self {
            fuzzer,
            test_runner,
            resource_address,
            component_address,
            account_public_key: public_key.into(),
            account_component_address: account,
        }
    }

    fn next_amount(&mut self) -> Decimal {
        self.fuzzer.next_amount()
    }

    fn run_fuzz(
        &mut self,
    ) -> BTreeMap<ResourceFuzzAction, BTreeMap<ConsensusFuzzActionResult, u64>> {
        let mut fuzz_results: BTreeMap<
            ResourceFuzzAction,
            BTreeMap<ConsensusFuzzActionResult, u64>,
        > = BTreeMap::new();
        for _ in 0..100 {
            let mut builder = ManifestBuilder::new();
            let start = ResourceFuzzStartAction::from_repr(self.fuzzer.next_u8(2u8)).unwrap();
            let (mut builder, start_trivial) = match start {
                ResourceFuzzStartAction::Mint => {
                    let amount = self.next_amount();
                    let builder = builder
                        .call_method(
                            self.resource_address,
                            FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT,
                            FungibleResourceManagerMintInput {
                                amount,
                            }
                        );
                    (builder, amount.is_zero())
                }
                ResourceFuzzStartAction::VaultTake => {
                    let amount = self.next_amount();
                    let builder = builder
                        .call_method(self.component_address, "call_vault", manifest_args!(VAULT_TAKE_IDENT, (amount,)));
                    (builder, amount.is_zero())
                }
            };


            let end = ResourceFuzzEndAction::from_repr(self.fuzzer.next_u8(2u8)).unwrap();
            let (mut builder, end_trivial) = match end {
                ResourceFuzzEndAction::Burn => {
                    {
                        let amount = self.next_amount();
                        let builder = builder
                            .take_from_worktop(self.resource_address, amount, "bucket")
                            .burn_resource("bucket");
                        (builder, amount.is_zero())
                    }
                }
                ResourceFuzzEndAction::VaultPut => {
                    {
                        let amount = self.next_amount();
                        let builder = builder
                            .take_from_worktop(self.resource_address, amount, "bucket")
                            .with_bucket("bucket", |builder, bucket| {
                                builder.call_method(self.component_address, "call_vault", manifest_args!(VAULT_PUT_IDENT, (bucket,)))
                            });
                        (builder, amount.is_zero())
                    }
                }
            };

            let manifest = builder
                .deposit_batch(self.account_component_address)
                .build();
            let receipt = self.test_runner.execute_manifest_ignoring_fee(
                manifest,
                vec![NonFungibleGlobalId::from_public_key(
                    &self.account_public_key,
                )],
            );

            let result = receipt.expect_commit_ignore_outcome();
            let result = match (&result.outcome, start_trivial || end_trivial) {
                (TransactionOutcome::Success(..), true) => {
                    ConsensusFuzzActionResult::TrivialSuccess
                }
                (TransactionOutcome::Success(..), false) => ConsensusFuzzActionResult::Success,
                (TransactionOutcome::Failure(..), true) => {
                    ConsensusFuzzActionResult::TrivialFailure
                }
                (TransactionOutcome::Failure(..), false) => ConsensusFuzzActionResult::Failure,
            };

            let results = fuzz_results.entry(ResourceFuzzAction(start, end)).or_default();
            results.entry(result).or_default().add_assign(&1);


        }

        fuzz_results
    }
}
