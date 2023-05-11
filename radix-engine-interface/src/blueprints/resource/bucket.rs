use crate::data::scrypto::model::Own;
use crate::data::scrypto::ScryptoCustomTypeKind;
use crate::data::scrypto::ScryptoCustomValueKind;
use crate::math::*;
use crate::*;
use radix_engine_common::data::scrypto::*;
use radix_engine_common::types::*;
use sbor::rust::fmt::Debug;
use sbor::*;

use super::Proof;

pub const BUCKET_TAKE_IDENT: &str = "take";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct BucketTakeInput {
    pub amount: Decimal,
}

pub type BucketTakeOutput = Bucket;

pub const BUCKET_PUT_IDENT: &str = "put";

#[derive(Debug, Eq, PartialEq, ScryptoSbor)]
pub struct BucketPutInput {
    pub bucket: Bucket,
}

pub type BucketPutOutput = ();

impl Clone for BucketPutInput {
    fn clone(&self) -> Self {
        Self {
            bucket: Bucket(self.bucket.0),
        }
    }
}

pub const BUCKET_GET_AMOUNT_IDENT: &str = "get_amount";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct BucketGetAmountInput {}

pub type BucketGetAmountOutput = Decimal;

pub const BUCKET_GET_RESOURCE_ADDRESS_IDENT: &str = "get_resource_address";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct BucketGetResourceAddressInput {}

pub type BucketGetResourceAddressOutput = ResourceAddress;

pub const BUCKET_CREATE_PROOF_IDENT: &str = "create_proof";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct BucketCreateProofInput {}

pub type BucketCreateProofOutput = Proof;

//========
// Stub
//========

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Bucket(pub Own);

//========
// binary
//========

impl Categorize<ScryptoCustomValueKind> for Bucket {
    #[inline]
    fn value_kind() -> ValueKind<ScryptoCustomValueKind> {
        Own::value_kind()
    }
}

impl<E: Encoder<ScryptoCustomValueKind>> Encode<ScryptoCustomValueKind, E> for Bucket {
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(Self::value_kind())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.0.encode_body(encoder)
    }
}

impl<D: Decoder<ScryptoCustomValueKind>> Decode<ScryptoCustomValueKind, D> for Bucket {
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<ScryptoCustomValueKind>,
    ) -> Result<Self, DecodeError> {
        Own::decode_body_with_value_kind(decoder, value_kind).map(|o| Self(o))
    }
}

impl Describe<ScryptoCustomTypeKind> for Bucket {
    const TYPE_ID: GlobalTypeId =
        GlobalTypeId::well_known(well_known_scrypto_custom_types::OWN_BUCKET_ID);

    fn type_data() -> TypeData<ScryptoCustomTypeKind, GlobalTypeId> {
        well_known_scrypto_custom_types::own_bucket_type_data()
    }
}
