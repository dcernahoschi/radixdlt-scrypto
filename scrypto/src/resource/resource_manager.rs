use sbor::rust::collections::HashMap;
use sbor::rust::fmt;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::*;

use crate::abi::*;
use crate::address::*;
use crate::buffer::scrypto_encode;
use crate::data::*;
use crate::engine::{api::*, types::*, utils::*};
use crate::math::*;
use crate::misc::*;
use crate::native_methods;
use crate::resource::*;
use crate::scrypto_type;
use crate::Describe;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, TypeId, Encode, Decode, Describe, PartialOrd, Ord,
)]
pub enum ResourceMethodAuthKey {
    Mint,
    Burn,
    Withdraw,
    Deposit,
    UpdateMetadata,
    UpdateNonFungibleData,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, TypeId, Encode, Decode, Describe)]
#[custom_type_id(ScryptoCustomTypeId)]
pub enum Mutability {
    LOCKED,
    MUTABLE(AccessRule),
}

#[derive(Debug, TypeId, Encode, Decode)]
#[custom_type_id(ScryptoCustomTypeId)]
pub struct ResourceManagerCreateInvocation {
    pub resource_type: ResourceType,
    pub metadata: HashMap<String, String>,
    pub access_rules: HashMap<ResourceMethodAuthKey, (AccessRule, Mutability)>,
    pub mint_params: Option<MintParams>,
}

#[derive(Debug, TypeId, Encode, Decode)]
#[custom_type_id(ScryptoCustomTypeId)]
pub struct ResourceManagerBucketBurnInvocation {
    pub bucket: Bucket,
}

#[derive(Debug, TypeId, Encode, Decode)]
#[custom_type_id(ScryptoCustomTypeId)]
pub struct ResourceManagerBurnInvocation {
    pub receiver: ResourceAddress,
    pub bucket: Bucket,
}

#[derive(Debug, TypeId, Encode, Decode)]
#[custom_type_id(ScryptoCustomTypeId)]
pub struct ResourceManagerUpdateAuthInvocation {
    pub receiver: ResourceAddress,
    pub method: ResourceMethodAuthKey,
    pub access_rule: AccessRule,
}

#[derive(Debug, TypeId, Encode, Decode)]
#[custom_type_id(ScryptoCustomTypeId)]
pub struct ResourceManagerLockAuthInvocation {
    pub receiver: ResourceAddress,
    pub method: ResourceMethodAuthKey,
}

#[derive(Debug, TypeId, Encode, Decode)]
#[custom_type_id(ScryptoCustomTypeId)]
pub struct ResourceManagerCreateVaultInvocation {
    pub receiver: ResourceAddress,
}

#[derive(Debug, TypeId, Encode, Decode)]
#[custom_type_id(ScryptoCustomTypeId)]
pub struct ResourceManagerCreateBucketInvocation {
    pub receiver: ResourceAddress,
}

#[derive(Debug, TypeId, Encode, Decode)]
#[custom_type_id(ScryptoCustomTypeId)]
pub struct ResourceManagerMintInvocation {
    pub receiver: ResourceAddress,
    pub mint_params: MintParams,
}

#[derive(Debug, TypeId, Encode, Decode)]
#[custom_type_id(ScryptoCustomTypeId)]
pub struct ResourceManagerGetMetadataInvocation {
    pub receiver: ResourceAddress,
}

#[derive(Debug, TypeId, Encode, Decode)]
#[custom_type_id(ScryptoCustomTypeId)]
pub struct ResourceManagerGetResourceTypeInvocation {
    pub receiver: ResourceAddress,
}

#[derive(Debug, TypeId, Encode, Decode)]
#[custom_type_id(ScryptoCustomTypeId)]
pub struct ResourceManagerGetTotalSupplyInvocation {
    pub receiver: ResourceAddress,
}

#[derive(Debug, TypeId, Encode, Decode)]
#[custom_type_id(ScryptoCustomTypeId)]
pub struct ResourceManagerUpdateMetadataInvocation {
    pub receiver: ResourceAddress,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, TypeId, Encode, Decode)]
#[custom_type_id(ScryptoCustomTypeId)]
pub struct ResourceManagerSetResourceAddressInvocation {
    pub receiver: ResourceAddress,
}

#[derive(Debug, TypeId, Encode, Decode)]
#[custom_type_id(ScryptoCustomTypeId)]
pub struct ResourceManagerUpdateNonFungibleDataInvocation {
    pub receiver: ResourceAddress,
    pub id: NonFungibleId,
    pub data: Vec<u8>,
}

#[derive(Debug, TypeId, Encode, Decode)]
#[custom_type_id(ScryptoCustomTypeId)]
pub struct ResourceManagerNonFungibleExistsInvocation {
    pub receiver: ResourceAddress,
    pub id: NonFungibleId,
}

#[derive(Debug, TypeId, Encode, Decode)]
#[custom_type_id(ScryptoCustomTypeId)]
pub struct ResourceManagerGetNonFungibleInvocation {
    pub receiver: ResourceAddress,
    pub id: NonFungibleId,
}

/// Represents a resource address.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ResourceAddress {
    Normal([u8; 26]),
}

/// Represents a resource manager.
#[derive(Debug)]
pub struct ResourceManager(pub(crate) ResourceAddress);

impl ResourceManager {
    pub fn set_mintable(&mut self, access_rule: AccessRule) -> () {
        let input = RadixEngineInput::InvokeNativeMethod(
            NativeMethod::ResourceManager(ResourceManagerMethod::UpdateAuth),
            scrypto_encode(&ResourceManagerUpdateAuthInvocation {
                receiver: self.0,
                method: ResourceMethodAuthKey::Mint,
                access_rule,
            }),
        );
        call_engine(input)
    }

    pub fn set_burnable(&mut self, access_rule: AccessRule) -> () {
        let input = RadixEngineInput::InvokeNativeMethod(
            NativeMethod::ResourceManager(ResourceManagerMethod::UpdateAuth),
            scrypto_encode(&ResourceManagerUpdateAuthInvocation {
                receiver: self.0,
                method: ResourceMethodAuthKey::Burn,
                access_rule,
            }),
        );
        call_engine(input)
    }

    pub fn set_withdrawable(&mut self, access_rule: AccessRule) -> () {
        let input = RadixEngineInput::InvokeNativeMethod(
            NativeMethod::ResourceManager(ResourceManagerMethod::UpdateAuth),
            scrypto_encode(&ResourceManagerUpdateAuthInvocation {
                receiver: self.0,
                method: ResourceMethodAuthKey::Withdraw,
                access_rule,
            }),
        );
        call_engine(input)
    }

    pub fn set_depositable(&mut self, access_rule: AccessRule) -> () {
        let input = RadixEngineInput::InvokeNativeMethod(
            NativeMethod::ResourceManager(ResourceManagerMethod::UpdateAuth),
            scrypto_encode(&ResourceManagerUpdateAuthInvocation {
                receiver: self.0,
                method: ResourceMethodAuthKey::Deposit,
                access_rule,
            }),
        );
        call_engine(input)
    }

    pub fn set_updateable_metadata(&self, access_rule: AccessRule) -> () {
        let input = RadixEngineInput::InvokeNativeMethod(
            NativeMethod::ResourceManager(ResourceManagerMethod::UpdateAuth),
            scrypto_encode(&ResourceManagerUpdateAuthInvocation {
                receiver: self.0,
                method: ResourceMethodAuthKey::UpdateMetadata,
                access_rule,
            }),
        );
        call_engine(input)
    }

    pub fn set_updateable_non_fungible_data(&self, access_rule: AccessRule) -> () {
        let input = RadixEngineInput::InvokeNativeMethod(
            NativeMethod::ResourceManager(ResourceManagerMethod::UpdateAuth),
            scrypto_encode(&ResourceManagerUpdateAuthInvocation {
                receiver: self.0,
                method: ResourceMethodAuthKey::UpdateNonFungibleData,
                access_rule,
            }),
        );
        call_engine(input)
    }

    pub fn lock_mintable(&mut self) -> () {
        let input = RadixEngineInput::InvokeNativeMethod(
            NativeMethod::ResourceManager(ResourceManagerMethod::LockAuth),
            scrypto_encode(&ResourceManagerLockAuthInvocation {
                receiver: self.0,
                method: ResourceMethodAuthKey::Mint,
            }),
        );
        call_engine(input)
    }

    pub fn lock_burnable(&mut self) -> () {
        let input = RadixEngineInput::InvokeNativeMethod(
            NativeMethod::ResourceManager(ResourceManagerMethod::LockAuth),
            scrypto_encode(&ResourceManagerLockAuthInvocation {
                receiver: self.0,
                method: ResourceMethodAuthKey::Burn,
            }),
        );
        call_engine(input)
    }

    pub fn lock_withdrawable(&mut self) -> () {
        let input = RadixEngineInput::InvokeNativeMethod(
            NativeMethod::ResourceManager(ResourceManagerMethod::LockAuth),
            scrypto_encode(&ResourceManagerLockAuthInvocation {
                receiver: self.0,
                method: ResourceMethodAuthKey::Withdraw,
            }),
        );
        call_engine(input)
    }

    pub fn lock_depositable(&mut self) -> () {
        let input = RadixEngineInput::InvokeNativeMethod(
            NativeMethod::ResourceManager(ResourceManagerMethod::LockAuth),
            scrypto_encode(&ResourceManagerLockAuthInvocation {
                receiver: self.0,
                method: ResourceMethodAuthKey::Deposit,
            }),
        );
        call_engine(input)
    }

    pub fn lock_updateable_metadata(&mut self) -> () {
        let input = RadixEngineInput::InvokeNativeMethod(
            NativeMethod::ResourceManager(ResourceManagerMethod::LockAuth),
            scrypto_encode(&ResourceManagerLockAuthInvocation {
                receiver: self.0,
                method: ResourceMethodAuthKey::UpdateMetadata,
            }),
        );
        call_engine(input)
    }

    pub fn lock_updateable_non_fungible_data(&mut self) -> () {
        let input = RadixEngineInput::InvokeNativeMethod(
            NativeMethod::ResourceManager(ResourceManagerMethod::LockAuth),
            scrypto_encode(&ResourceManagerLockAuthInvocation {
                receiver: self.0,
                method: ResourceMethodAuthKey::UpdateNonFungibleData,
            }),
        );
        call_engine(input)
    }

    fn mint_internal(&mut self, mint_params: MintParams) -> Bucket {
        let input = RadixEngineInput::InvokeNativeMethod(
            NativeMethod::ResourceManager(ResourceManagerMethod::Mint),
            scrypto_encode(&ResourceManagerMintInvocation {
                mint_params,
                receiver: self.0,
            }),
        );
        call_engine(input)
    }

    fn update_non_fungible_data_internal(&mut self, id: NonFungibleId, data: Vec<u8>) -> () {
        let input = RadixEngineInput::InvokeNativeMethod(
            NativeMethod::ResourceManager(ResourceManagerMethod::UpdateNonFungibleData),
            scrypto_encode(&ResourceManagerUpdateNonFungibleDataInvocation {
                id,
                data,
                receiver: self.0,
            }),
        );
        call_engine(input)
    }

    fn get_non_fungible_data_internal(&self, id: NonFungibleId) -> [Vec<u8>; 2] {
        let input = RadixEngineInput::InvokeNativeMethod(
            NativeMethod::ResourceManager(ResourceManagerMethod::GetNonFungible),
            scrypto_encode(&ResourceManagerGetNonFungibleInvocation {
                id,
                receiver: self.0,
            }),
        );
        call_engine(input)
    }

    native_methods! {
        NativeMethod::ResourceManager => {
            pub fn metadata(&self) -> HashMap<String, String> {
                ResourceManagerMethod::GetMetadata,
                ResourceManagerGetMetadataInvocation {
                    receiver: self.0,
                }
            }
            pub fn resource_type(&self) -> ResourceType {
                ResourceManagerMethod::GetResourceType,
                ResourceManagerGetResourceTypeInvocation {
                    receiver: self.0,
                }
            }
            pub fn total_supply(&self) -> Decimal {
                ResourceManagerMethod::GetTotalSupply,
                ResourceManagerGetTotalSupplyInvocation {
                    receiver: self.0,
                }
            }
            pub fn update_metadata(&mut self, metadata: HashMap<String, String>) -> () {
                ResourceManagerMethod::UpdateMetadata,
                ResourceManagerUpdateMetadataInvocation {
                    receiver: self.0,
                    metadata,
                }
            }
            pub fn non_fungible_exists(&self, id: &NonFungibleId) -> bool {
                ResourceManagerMethod::NonFungibleExists,
                ResourceManagerNonFungibleExistsInvocation {
                    receiver: self.0,
                    id: id.clone()
                }
            }
            pub fn burn(&mut self, bucket: Bucket) -> () {
                ResourceManagerMethod::Burn,
                ResourceManagerBurnInvocation {
                    receiver: self.0,
                    bucket
                }
            }
        }
    }

    /// Mints fungible resources
    pub fn mint<T: Into<Decimal>>(&mut self, amount: T) -> Bucket {
        self.mint_internal(MintParams::Fungible {
            amount: amount.into(),
        })
    }

    /// Mints non-fungible resources
    pub fn mint_non_fungible<T: NonFungibleData>(&mut self, id: &NonFungibleId, data: T) -> Bucket {
        let mut entries = HashMap::new();
        entries.insert(id.clone(), (data.immutable_data(), data.mutable_data()));
        self.mint_internal(MintParams::NonFungible { entries })
    }

    /// Returns the data of a non-fungible unit, both the immutable and mutable parts.
    ///
    /// # Panics
    /// Panics if this is not a non-fungible resource or the specified non-fungible is not found.
    pub fn get_non_fungible_data<T: NonFungibleData>(&self, id: &NonFungibleId) -> T {
        let non_fungible = self.get_non_fungible_data_internal(id.clone());
        T::decode(&non_fungible[0], &non_fungible[1]).unwrap()
    }

    /// Updates the mutable part of a non-fungible unit.
    ///
    /// # Panics
    /// Panics if this is not a non-fungible resource or the specified non-fungible is not found.
    pub fn update_non_fungible_data<T: NonFungibleData>(
        &mut self,
        id: &NonFungibleId,
        new_data: T,
    ) {
        self.update_non_fungible_data_internal(id.clone(), new_data.mutable_data())
    }
}

//========
// binary
//========

impl TryFrom<&[u8]> for ResourceAddress {
    type Error = AddressError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        match slice.len() {
            27 => match EntityType::try_from(slice[0])
                .map_err(|_| AddressError::InvalidEntityTypeId(slice[0]))?
            {
                EntityType::Resource => Ok(Self::Normal(copy_u8_array(&slice[1..]))),
                _ => Err(AddressError::InvalidEntityTypeId(slice[0])),
            },
            _ => Err(AddressError::InvalidLength(slice.len())),
        }
    }
}

impl ResourceAddress {
    pub fn to_vec(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.push(EntityType::resource(self).id());
        match self {
            Self::Normal(v) => buf.extend(v),
        }
        buf
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.to_vec())
    }

    pub fn try_from_hex(hex_str: &str) -> Result<Self, AddressError> {
        let bytes = hex::decode(hex_str).map_err(|_| AddressError::HexDecodingError)?;

        Self::try_from(bytes.as_ref())
    }
}

scrypto_type!(
    ResourceAddress,
    ScryptoCustomTypeId::ResourceAddress,
    Type::ResourceAddress,
    27
);

//======
// text
//======

impl fmt::Debug for ResourceAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.display(NO_NETWORK))
    }
}

impl<'a> ContextualDisplay<AddressDisplayContext<'a>> for ResourceAddress {
    type Error = AddressError;

    fn contextual_format<F: fmt::Write>(
        &self,
        f: &mut F,
        context: &AddressDisplayContext<'a>,
    ) -> Result<(), Self::Error> {
        if let Some(encoder) = context.encoder {
            return encoder.encode_resource_address_to_fmt(f, self);
        }

        // This could be made more performant by streaming the hex into the formatter
        match self {
            ResourceAddress::Normal(_) => {
                write!(f, "NormalResource[{}]", self.to_hex())
            }
        }
        .map_err(|err| AddressError::FormatError(err))
    }
}
