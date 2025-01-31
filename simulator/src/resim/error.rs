use std::io;
use std::path::PathBuf;

use radix_engine::errors::{RejectionReason, RuntimeError};
use radix_engine::transaction::AbortReason;
use radix_engine::types::{ComponentAddress, NodeId, PackageAddress};
use radix_engine::utils::ExtractSchemaError;
use radix_engine::vm::wasm::PrepareError;
use radix_engine_interface::blueprints::resource::ParseNonFungibleGlobalIdError;
use radix_engine_interface::network::ParseNetworkError;
use radix_engine_interface::types::SchemaHash;
use sbor::*;
use transaction::errors::*;
use transaction::model::PrepareError as TransactionPrepareError;

use crate::ledger::EntityDumpError;
use crate::utils::*;

/// Represents a resim error.
#[derive(Debug)]
pub enum Error {
    NoDefaultAccount,
    NoDefaultPrivateKey,
    NoDefaultOwnerBadge,

    HomeDirUnknown,

    PackageNotFound(PackageAddress),
    SchemaNotFound(NodeId, SchemaHash),
    BlueprintNotFound(PackageAddress, String),
    ComponentNotFound(ComponentAddress),
    InstanceSchemaNot(ComponentAddress, u8),

    IOError(io::Error),

    IOErrorAtPath(io::Error, PathBuf),

    SborDecodeError(DecodeError),

    SborEncodeError(EncodeError),

    BuildError(BuildError),

    ExtractSchemaError(ExtractSchemaError),

    InvalidPackage(PrepareError),

    TransactionConstructionError(BuildCallInstructionError),

    TransactionValidationError(TransactionValidationError),

    TransactionPrepareError(TransactionPrepareError),

    TransactionFailed(RuntimeError),

    TransactionRejected(RejectionReason),

    TransactionAborted(AbortReason),

    LedgerDumpError(EntityDumpError),

    CompileError(transaction::manifest::CompileError),

    DecompileError(transaction::manifest::DecompileError),

    InvalidId(String),

    InvalidPrivateKey,

    NonFungibleGlobalIdError(ParseNonFungibleGlobalIdError),

    FailedToBuildArguments(BuildCallArgumentError),

    ParseNetworkError(ParseNetworkError),

    OwnerBadgeNotSpecified,

    InstructionSchemaValidationError(radix_engine::utils::LocatedInstructionSchemaValidationError),

    InvalidResourceSpecifier(String),

    RemoteGenericSubstitutionNotSupported,
}
