use radix_engine_interface::api::types::RENodeId;
use radix_engine_interface::api::*;
use radix_engine_interface::blueprints::clock::*;
use radix_engine_interface::blueprints::epoch_manager::*;
use radix_engine_interface::blueprints::transaction_runtime::*;
use radix_engine_interface::constants::{CLOCK, EPOCH_MANAGER};
use radix_engine_interface::crypto::hash;
use radix_engine_interface::data::scrypto::*;
use radix_engine_interface::time::*;
use sbor::generate_full_schema_from_single_type;
use sbor::rust::fmt::Debug;

#[derive(Debug)]
pub struct Runtime {}

impl Runtime {
    /// Emits an application event
    pub fn emit_event<T: ScryptoEncode + ScryptoDescribe, Y, E>(
        api: &mut Y,
        event: T,
    ) -> Result<(), E>
    where
        Y: ClientEventApi<E>,
        E: Debug + ScryptoCategorize + ScryptoDecode,
    {
        let schema_hash = {
            let (local_type_index, schema) =
                generate_full_schema_from_single_type::<T, ScryptoCustomTypeExtension>();
            scrypto_encode(&(local_type_index, schema))
                .map(hash)
                .expect("Schema can't be encoded!")
        };
        api.emit_event(schema_hash, scrypto_encode(&event).unwrap())
    }

    pub fn sys_current_epoch<Y, E>(api: &mut Y) -> Result<u64, E>
    where
        Y: ClientObjectApi<E>,
        E: Debug + ScryptoCategorize + ScryptoDecode,
    {
        let rtn = api.call_method(
            RENodeId::GlobalComponent(EPOCH_MANAGER.into()),
            EPOCH_MANAGER_GET_CURRENT_EPOCH_IDENT,
            scrypto_encode(&EpochManagerGetCurrentEpochInput).unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    pub fn sys_current_time<Y, E>(api: &mut Y, precision: TimePrecision) -> Result<Instant, E>
    where
        Y: ClientObjectApi<E>,
        E: Debug + ScryptoCategorize + ScryptoDecode,
    {
        let rtn = api.call_method(
            RENodeId::GlobalComponent(CLOCK.into()),
            CLOCK_GET_CURRENT_TIME_IDENT,
            scrypto_encode(&ClockGetCurrentTimeInput { precision }).unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    pub fn sys_compare_against_current_time<Y, E>(
        api: &mut Y,
        instant: Instant,
        precision: TimePrecision,
        operator: TimeComparisonOperator,
    ) -> Result<bool, E>
    where
        Y: ClientObjectApi<E>,
        E: Debug + ScryptoCategorize + ScryptoDecode,
    {
        let rtn = api.call_method(
            RENodeId::GlobalComponent(CLOCK.into()),
            CLOCK_COMPARE_CURRENT_TIME_IDENT,
            scrypto_encode(&ClockCompareCurrentTimeInput {
                precision,
                instant,
                operator,
            })
            .unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    /// Generates a UUID.
    pub fn generate_uuid<Y, E>(api: &mut Y) -> Result<u128, E>
    where
        Y: ClientApi<E>,
        E: Debug + ScryptoCategorize + ScryptoDecode,
    {
        let rtn = api.call_method(
            RENodeId::TransactionRuntime,
            TRANSACTION_RUNTIME_GENERATE_UUID_IDENT,
            scrypto_encode(&TransactionRuntimeGenerateUuid {}).unwrap(),
        )?;
        Ok(scrypto_decode(&rtn).unwrap())
    }
}
