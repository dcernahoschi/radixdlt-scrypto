use crate::types::*;
use radix_engine_interface::blueprints::resource::*;

#[derive(Debug, ScryptoSbor, Default)]
pub struct AuthZone {
    pub proofs: Vec<Proof>,

    // Virtualized resources, note that one cannot create proofs with virtual resources but only be used for AuthZone checks
    pub virtual_resources: BTreeSet<ResourceAddress>,
    pub virtual_non_fungibles: BTreeSet<NonFungibleGlobalId>,

    /// Virtual proofs which are only valid for the current local call frame
    pub virtual_local_call_frame_proofs: BTreeSet<NonFungibleGlobalId>,

    /// Virtual proofs which are valid for the current global call frame
    pub virtual_global_call_frame_proofs: BTreeSet<NonFungibleGlobalId>,

    pub is_barrier: bool,
    pub parent: Option<Reference>,
}

impl Clone for AuthZone {
    fn clone(&self) -> Self {
        Self {
            proofs: self.proofs.iter().map(|p| Proof(p.0)).collect(),
            virtual_resources: self.virtual_resources.clone(),
            virtual_non_fungibles: self.virtual_non_fungibles.clone(),
            virtual_local_call_frame_proofs: self.virtual_local_call_frame_proofs.clone(),
            virtual_global_call_frame_proofs: self.virtual_global_call_frame_proofs.clone(),
            is_barrier: self.is_barrier,
            parent: self.parent.clone(),
        }
    }
}

impl AuthZone {
    pub fn new(
        proofs: Vec<Proof>,
        virtual_resources: BTreeSet<ResourceAddress>,
        virtual_non_fungibles: BTreeSet<NonFungibleGlobalId>,
        local_call_frame_proofs: BTreeSet<NonFungibleGlobalId>,
        global_call_frame_proofs: BTreeSet<NonFungibleGlobalId>,
        is_barrier: bool,
        parent: Option<Reference>,
    ) -> Self {
        Self {
            proofs,
            virtual_resources,
            virtual_non_fungibles,
            virtual_local_call_frame_proofs: local_call_frame_proofs,
            virtual_global_call_frame_proofs: global_call_frame_proofs,
            is_barrier,
            parent,
        }
    }

    pub fn proofs(&self) -> &[Proof] {
        &self.proofs
    }

    pub fn virtual_resources(&self) -> &BTreeSet<ResourceAddress> {
        &self.virtual_resources
    }

    pub fn virtual_non_fungibles(&self) -> &BTreeSet<NonFungibleGlobalId> {
        &self.virtual_non_fungibles
    }

    pub fn virtual_local_call_frame_proofs(&self) -> &BTreeSet<NonFungibleGlobalId> {
        &self.virtual_local_call_frame_proofs
    }

    pub fn virtual_global_call_frame_proofs(&self) -> &BTreeSet<NonFungibleGlobalId> {
        &self.virtual_global_call_frame_proofs
    }

    pub fn push(&mut self, proof: Proof) {
        self.proofs.push(proof);
    }

    pub fn pop(&mut self) -> Option<Proof> {
        self.proofs.pop()
    }

    pub fn drain(&mut self) -> Vec<Proof> {
        self.proofs.drain(0..).collect()
    }

    pub fn clear_signature_proofs(&mut self) {
        self.virtual_resources.retain(|x| {
            x != &SECP256K1_SIGNATURE_VIRTUAL_BADGE && x != &ED25519_SIGNATURE_VIRTUAL_BADGE
        });
        self.virtual_non_fungibles.retain(|x| {
            x.resource_address() != SECP256K1_SIGNATURE_VIRTUAL_BADGE
                && x.resource_address() != ED25519_SIGNATURE_VIRTUAL_BADGE
        });
        self.virtual_local_call_frame_proofs.retain(|x| {
            x.resource_address() != SECP256K1_SIGNATURE_VIRTUAL_BADGE
                && x.resource_address() != ED25519_SIGNATURE_VIRTUAL_BADGE
        });
    }
}
