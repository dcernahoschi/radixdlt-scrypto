use crate::errors::*;
use crate::kernel::kernel_api::KernelSubstateApi;
use crate::kernel::*;
use crate::system::global::GlobalAddressSubstate;
use crate::system::node::RENodeInit;
use crate::system::node::RENodeModuleInit;
use crate::system::node_modules::auth::AccessRulesChainSubstate;
use crate::system::node_modules::metadata::MetadataSubstate;
use crate::types::*;
use crate::wasm::*;
use core::fmt::Debug;
use radix_engine_interface::api::package::*;
use radix_engine_interface::api::types::*;
use radix_engine_interface::api::types::{NativeFn, PackageFn, PackageId, RENodeId};
use radix_engine_interface::api::ClientDerefApi;
use radix_engine_interface::api::ClientNativeInvokeApi;
use radix_engine_interface::blueprints::resource::{
    Bucket, ResourceManagerCreateVaultInvocation, VaultGetAmountInvocation, VaultTakeInvocation,
};

pub struct Package;

#[derive(Debug, Clone, PartialEq, Eq, Categorize, Encode, Decode)]
pub enum PackageError {
    InvalidRequestData(DecodeError),
    InvalidAbi(DecodeError),
    InvalidWasm(PrepareError),
    BlueprintNotFound,
    MethodNotFound(String),
    CouldNotEncodePackageAddress,
}

impl Package {
    fn new(
        code: Vec<u8>,
        abi: &BTreeMap<String, BlueprintAbi>,
    ) -> Result<WasmCodeSubstate, PrepareError> {
        WasmValidator::default().validate(&code, abi)?;

        Ok(WasmCodeSubstate {
            code: code,
        })
    }
}

impl ExecutableInvocation for PackagePublishNativeInvocation {
    type Exec = Self;

    fn resolve<D: ClientDerefApi<RuntimeError>>(
        self,
        _api: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let actor = ResolvedActor::function(NativeFn::Package(PackageFn::PublishNative));
        Ok((actor, CallFrameUpdate::empty(), self))
    }
}

impl Executor for PackagePublishNativeInvocation {
    type Output = PackageAddress;

    fn execute<Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(PackageAddress, CallFrameUpdate), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientNativeInvokeApi<RuntimeError>,
    {
        let metadata_substate = MetadataSubstate {
            metadata: self.metadata,
        };
        let access_rules = AccessRulesChainSubstate {
            access_rules_chain: vec![self.access_rules],
        };
        let blueprint_abis = scrypto_decode::<BTreeMap<String, BlueprintAbi>>(&self.abi).map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::PackageError(
                PackageError::InvalidAbi(e),
            ))
        })?;

        let mut node_modules = BTreeMap::new();
        node_modules.insert(
            NodeModuleId::Metadata,
            RENodeModuleInit::Metadata(metadata_substate),
        );
        node_modules.insert(
            NodeModuleId::AccessRules,
            RENodeModuleInit::AccessRulesChain(access_rules),
        );

        let package = NativePackageInfoSubstate {
            native_package_code_id: self.native_package_code_id,
            dependent_resources: self.dependent_resources.into_iter().collect(),
            blueprint_abis,
        };

        // Create package node
        let node_id = api.allocate_node_id(RENodeType::Package)?;
        api.create_node(node_id, RENodeInit::NativePackage(package), node_modules)?;
        let package_id: PackageId = node_id.into();

        // Globalize
        let global_node_id = if let Some(address) = self.package_address {
            RENodeId::Global(GlobalAddress::Package(PackageAddress::Normal(address)))
        } else {
            api.allocate_node_id(RENodeType::GlobalPackage)?
        };

        api.create_node(
            global_node_id,
            RENodeInit::Global(GlobalAddressSubstate::Package(package_id)),
            BTreeMap::new(),
        )?;

        let package_address: PackageAddress = global_node_id.into();
        Ok((package_address, CallFrameUpdate::empty()))
    }
}

impl ExecutableInvocation for PackagePublishInvocation {
    type Exec = Self;

    fn resolve<D: ClientDerefApi<RuntimeError>>(
        self,
        _api: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let mut call_frame_update = CallFrameUpdate::empty();
        call_frame_update.add_ref(RENodeId::Global(GlobalAddress::Resource(RADIX_TOKEN)));
        let actor = ResolvedActor::function(NativeFn::Package(PackageFn::Publish));
        Ok((actor, call_frame_update, self))
    }
}

impl Executor for PackagePublishInvocation {
    type Output = PackageAddress;

    fn execute<Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(PackageAddress, CallFrameUpdate), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientNativeInvokeApi<RuntimeError>,
    {
        let royalty_vault_id = api
            .call_native(ResourceManagerCreateVaultInvocation {
                receiver: RADIX_TOKEN,
            })?
            .vault_id();

        let blueprint_abis = scrypto_decode::<BTreeMap<String, BlueprintAbi>>(&self.abi).map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::PackageError(
                PackageError::InvalidAbi(e),
            ))
        })?;
        let wasm_code_substate = Package::new(self.code, &blueprint_abis).map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::PackageError(
                PackageError::InvalidWasm(e),
            ))
        })?;
        let package_info_substate = PackageInfoSubstate {
            blueprint_abis,
            dependent_resources: BTreeSet::new(),
        };
        let package_royalty_config = PackageRoyaltyConfigSubstate {
            royalty_config: self.royalty_config,
        };
        let package_royalty_accumulator = PackageRoyaltyAccumulatorSubstate {
            royalty: Own::Vault(royalty_vault_id),
        };
        let metadata_substate = MetadataSubstate {
            metadata: self.metadata,
        };
        let access_rules = AccessRulesChainSubstate {
            access_rules_chain: vec![self.access_rules],
        };

        // TODO: Can we trust developers enough to add protection for
        // - `metadata::set`
        // - `access_rules_chain::add_access_rules`
        // - `royalty::set_royalty_config`
        // - `royalty::claim_royalty`

        let mut node_modules = BTreeMap::new();
        node_modules.insert(
            NodeModuleId::PackageRoyalty,
            RENodeModuleInit::PackageRoyalty(package_royalty_config, package_royalty_accumulator),
        );
        node_modules.insert(
            NodeModuleId::Metadata,
            RENodeModuleInit::Metadata(metadata_substate),
        );
        node_modules.insert(
            NodeModuleId::AccessRules,
            RENodeModuleInit::AccessRulesChain(access_rules),
        );

        // Create package node
        let node_id = api.allocate_node_id(RENodeType::Package)?;
        api.create_node(node_id, RENodeInit::WasmPackage(package_info_substate, wasm_code_substate), node_modules)?;
        let package_id: PackageId = node_id.into();

        // Globalize
        let global_node_id = if let Some(address) = self.package_address {
            RENodeId::Global(GlobalAddress::Package(PackageAddress::Normal(address)))
        } else {
            api.allocate_node_id(RENodeType::GlobalPackage)?
        };

        api.create_node(
            global_node_id,
            RENodeInit::Global(GlobalAddressSubstate::Package(package_id)),
            BTreeMap::new(),
        )?;

        let package_address: PackageAddress = global_node_id.into();
        Ok((package_address, CallFrameUpdate::empty()))
    }
}

impl ExecutableInvocation for PackageSetRoyaltyConfigInvocation {
    type Exec = PackageSetRoyaltyConfigExecutable;

    fn resolve<D: ClientDerefApi<RuntimeError>>(
        self,
        api: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let mut call_frame_update = CallFrameUpdate::empty();
        let receiver = RENodeId::Global(GlobalAddress::Package(self.receiver));
        let resolved_receiver = deref_and_update(receiver, &mut call_frame_update, api)?;

        let actor = ResolvedActor::method(
            NativeFn::Package(PackageFn::SetRoyaltyConfig),
            resolved_receiver,
        );
        let executor = PackageSetRoyaltyConfigExecutable {
            receiver: resolved_receiver.receiver,
            royalty_config: self.royalty_config,
        };

        Ok((actor, call_frame_update, executor))
    }
}

impl Executor for PackageSetRoyaltyConfigExecutable {
    type Output = ();

    fn execute<Y, W: WasmEngine>(self, api: &mut Y) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        // TODO: auth check
        let node_id = self.receiver;
        let handle = api.lock_substate(
            node_id,
            NodeModuleId::PackageRoyalty,
            SubstateOffset::Royalty(RoyaltyOffset::RoyaltyConfig),
            LockFlags::MUTABLE,
        )?;

        let mut substate = api.get_ref_mut(handle)?;
        substate.package_royalty_config().royalty_config = self.royalty_config;

        api.drop_lock(handle)?;

        Ok(((), CallFrameUpdate::empty()))
    }
}

impl ExecutableInvocation for PackageClaimRoyaltyInvocation {
    type Exec = PackageClaimRoyaltyExecutable;

    fn resolve<D: ClientDerefApi<RuntimeError>>(
        self,
        api: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let mut call_frame_update = CallFrameUpdate::empty();
        let receiver = RENodeId::Global(GlobalAddress::Package(self.receiver));
        let resolved_receiver = deref_and_update(receiver, &mut call_frame_update, api)?;

        let actor = ResolvedActor::method(
            NativeFn::Package(PackageFn::ClaimRoyalty),
            resolved_receiver,
        );
        let executor = PackageClaimRoyaltyExecutable {
            receiver: resolved_receiver.receiver,
        };

        Ok((actor, call_frame_update, executor))
    }
}

impl Executor for PackageClaimRoyaltyExecutable {
    type Output = Bucket;

    fn execute<Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(Bucket, CallFrameUpdate), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientNativeInvokeApi<RuntimeError>,
    {
        // TODO: auth check
        let node_id = self.receiver;
        let handle = api.lock_substate(
            node_id,
            NodeModuleId::PackageRoyalty,
            SubstateOffset::Royalty(RoyaltyOffset::RoyaltyAccumulator),
            LockFlags::MUTABLE,
        )?;

        let mut substate_mut = api.get_ref_mut(handle)?;
        let royalty_vault = substate_mut.package_royalty_accumulator().royalty.clone();

        let amount = api.call_native(VaultGetAmountInvocation {
            receiver: royalty_vault.vault_id(),
        })?;

        let bucket = api.call_native(VaultTakeInvocation {
            receiver: royalty_vault.vault_id(),
            amount,
        })?;
        let bucket_id = bucket.0;

        api.drop_lock(handle)?;

        Ok((
            Bucket(bucket_id),
            CallFrameUpdate::move_node(RENodeId::Bucket(bucket_id)),
        ))
    }
}
