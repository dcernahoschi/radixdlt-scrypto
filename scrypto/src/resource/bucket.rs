use sbor::rust::collections::BTreeSet;
#[cfg(not(feature = "alloc"))]
use sbor::rust::fmt;
use sbor::rust::vec::Vec;
use sbor::*;

use crate::abi::*;
use crate::buffer::scrypto_encode;
use crate::data::*;
use crate::engine::{api::*, types::*, utils::*};
use crate::math::*;
use crate::misc::*;
use crate::native_methods;
use crate::resource::*;
use crate::scrypto_type;

#[derive(Debug, TypeId, Encode, Decode)]
#[custom_type_id(ScryptoCustomTypeId)]
pub struct BucketTakeInvocation {
    pub receiver: BucketId,
    pub amount: Decimal,
}

#[derive(Debug, TypeId, Encode, Decode)]
#[custom_type_id(ScryptoCustomTypeId)]
pub struct BucketPutInvocation {
    pub receiver: BucketId,
    pub bucket: scrypto::resource::Bucket,
}

#[derive(Debug, TypeId, Encode, Decode)]
#[custom_type_id(ScryptoCustomTypeId)]
pub struct BucketTakeNonFungiblesInvocation {
    pub receiver: BucketId,
    pub ids: BTreeSet<NonFungibleId>,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct BucketGetNonFungibleIdsInvocation {
    pub receiver: BucketId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct BucketGetAmountInvocation {
    pub receiver: BucketId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct BucketGetResourceAddressInvocation {
    pub receiver: BucketId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct BucketCreateProofInvocation {
    pub receiver: BucketId,
}

/// Represents a transient resource container.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Bucket(pub BucketId);

impl Bucket {
    /// Creates a new bucket to hold resources of the given definition.
    pub fn new(resource_address: ResourceAddress) -> Self {
        let input = RadixEngineInput::InvokeNativeMethod(
            NativeMethod::ResourceManager(ResourceManagerMethod::CreateBucket),
            scrypto_encode(&ResourceManagerCreateBucketInvocation {
                receiver: resource_address,
            }),
        );
        call_engine(input)
    }

    pub fn burn(self) -> () {
        let resource_address = self.resource_address();
        let input = RadixEngineInput::InvokeNativeMethod(
            NativeMethod::ResourceManager(ResourceManagerMethod::Burn),
            scrypto_encode(&ResourceManagerBurnInvocation {
                bucket: self,
                receiver: resource_address,
            }),
        );
        call_engine(input)
    }

    fn take_internal(&mut self, amount: Decimal) -> Self {
        let input = RadixEngineInput::InvokeNativeMethod(
            NativeMethod::Bucket(BucketMethod::Take),
            scrypto_encode(&BucketTakeInvocation {
                receiver: self.0,
                amount,
            }),
        );
        call_engine(input)
    }

    native_methods! {
        NativeMethod::Bucket => {
            pub fn take_non_fungibles(&mut self, non_fungible_ids: &BTreeSet<NonFungibleId>) -> Self {
                BucketMethod::TakeNonFungibles,
                BucketTakeNonFungiblesInvocation {
                    receiver: self.0,
                    ids: non_fungible_ids.clone()
                }
            }
            pub fn put(&mut self, other: Self) -> () {
                BucketMethod::Put,
                BucketPutInvocation {
                    receiver: self.0,
                    bucket: other,
                }
            }
            pub fn non_fungible_ids(&self) -> BTreeSet<NonFungibleId> {
                BucketMethod::GetNonFungibleIds,
                BucketGetNonFungibleIdsInvocation {
                    receiver: self.0,
                }
            }
            pub fn amount(&self) -> Decimal {
                BucketMethod::GetAmount,
                BucketGetAmountInvocation {
                    receiver: self.0,
                }
            }
            pub fn resource_address(&self) -> ResourceAddress {
                BucketMethod::GetResourceAddress,
                BucketGetResourceAddressInvocation {
                    receiver: self.0,
                }
            }
            pub fn create_proof(&self) -> scrypto::resource::Proof {
                BucketMethod::CreateProof,
                BucketCreateProofInvocation {
                    receiver: self.0
                }
            }
        }
    }

    /// Takes some amount of resources from this bucket.
    pub fn take<A: Into<Decimal>>(&mut self, amount: A) -> Self {
        self.take_internal(amount.into())
    }

    /// Takes a specific non-fungible from this bucket.
    ///
    /// # Panics
    /// Panics if this is not a non-fungible bucket or the specified non-fungible resource is not found.
    pub fn take_non_fungible(&mut self, non_fungible_id: &NonFungibleId) -> Bucket {
        self.take_non_fungibles(&BTreeSet::from([non_fungible_id.clone()]))
    }

    /// Uses resources in this bucket as authorization for an operation.
    pub fn authorize<F: FnOnce() -> O, O>(&self, f: F) -> O {
        ComponentAuthZone::push(self.create_proof());
        let output = f();
        ComponentAuthZone::pop().drop();
        output
    }

    /// Checks if this bucket is empty.
    pub fn is_empty(&self) -> bool {
        self.amount() == 0.into()
    }

    /// Returns all the non-fungible units contained.
    ///
    /// # Panics
    /// Panics if this is not a non-fungible bucket.
    pub fn non_fungibles<T: NonFungibleData>(&self) -> Vec<NonFungible<T>> {
        let resource_address = self.resource_address();
        self.non_fungible_ids()
            .iter()
            .map(|id| NonFungible::from(NonFungibleAddress::new(resource_address, id.clone())))
            .collect()
    }

    /// Returns a singleton non-fungible id
    ///
    /// # Panics
    /// Panics if this is not a singleton bucket
    pub fn non_fungible_id(&self) -> NonFungibleId {
        let non_fungible_ids = self.non_fungible_ids();
        if non_fungible_ids.len() != 1 {
            panic!("Expecting singleton NFT vault");
        }
        self.non_fungible_ids().into_iter().next().unwrap()
    }

    /// Returns a singleton non-fungible.
    ///
    /// # Panics
    /// Panics if this is not a singleton bucket
    pub fn non_fungible<T: NonFungibleData>(&self) -> NonFungible<T> {
        let non_fungibles = self.non_fungibles();
        if non_fungibles.len() != 1 {
            panic!("Expecting singleton NFT bucket");
        }
        non_fungibles.into_iter().next().unwrap()
    }
}

//========
// error
//========

/// Represents an error when decoding bucket.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseBucketError {
    InvalidLength(usize),
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseBucketError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseBucketError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//========
// binary
//========

impl TryFrom<&[u8]> for Bucket {
    type Error = ParseBucketError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        match slice.len() {
            4 => Ok(Self(u32::from_le_bytes(copy_u8_array(slice)))),
            _ => Err(ParseBucketError::InvalidLength(slice.len())),
        }
    }
}

impl Bucket {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_le_bytes().to_vec()
    }
}

scrypto_type!(Bucket, ScryptoCustomTypeId::Bucket, Type::Bucket, 4);
