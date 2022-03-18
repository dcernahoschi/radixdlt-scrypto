use crate::errors::RuntimeError;
use crate::errors::RuntimeError::NotAuthorized;
use crate::model::Proof;
use sbor::*;
use scrypto::math::Decimal;
use scrypto::prelude::{NonFungibleAddress, ResourceDefId};
use scrypto::rust::vec;
use scrypto::rust::vec::Vec;

#[derive(Debug, Clone, PartialEq, Eq, Hash, TypeId, Encode, Decode)]
pub enum Rule {
    NonFungible(NonFungibleAddress),
    AnyOfResource(ResourceDefId),
    SomeOfResource(Decimal, ResourceDefId),
    AllOf(Vec<Rule>),
    OneOf(Vec<Rule>),
}

impl From<scrypto::resource::AuthRule> for Rule {
    fn from(auth_rule: scrypto::prelude::AuthRule) -> Self {
        match auth_rule {
            ::scrypto::resource::AuthRule::NonFungible(addr) => Rule::NonFungible(addr),
            ::scrypto::resource::AuthRule::AnyOfResource(resource_def_id) => {
                Rule::AnyOfResource(resource_def_id)
            }
            ::scrypto::resource::AuthRule::SomeOfResource(amount, resource_def_id) => {
                Rule::SomeOfResource(amount, resource_def_id)
            }
            ::scrypto::resource::AuthRule::AllOf(auth_rules) => {
                Rule::AllOf(auth_rules.into_iter().map(Rule::from).collect())
            }
            ::scrypto::resource::AuthRule::OneOf(auth_rules) => {
                Rule::OneOf(auth_rules.into_iter().map(Rule::from).collect())
            }
        }
    }
}

impl Rule {
    pub fn or(self, other: Rule) -> Self {
        match self {
            Rule::NonFungible(_) => Rule::OneOf(vec![self, other]),
            Rule::AnyOfResource(_) => Rule::OneOf(vec![self, other]),
            Rule::SomeOfResource(_, _) => Rule::OneOf(vec![self, other]),
            Rule::AllOf(rules) => Rule::OneOf(vec![Rule::AllOf(rules), other]),
            Rule::OneOf(mut rules) => {
                rules.push(other);
                Rule::OneOf(rules)
            }
        }
    }

    pub fn check(&self, proofs_vector: &[&[Proof]]) -> Result<(), RuntimeError> {
        match self {
            Rule::NonFungible(non_fungible_address) => {
                for proofs in proofs_vector {
                    for p in proofs.iter() {
                        let proof_resource_def_id = p.resource_def_id();
                        if proof_resource_def_id == non_fungible_address.resource_def_id()
                            && match p.total_amount().as_non_fungible_ids() {
                                Some(ids) => ids.contains(&non_fungible_address.non_fungible_id()),
                                None => false,
                            }
                        {
                            return Ok(());
                        }
                    }
                }

                Err(NotAuthorized)
            }
            Rule::AnyOfResource(resource_def_id) => {
                for proofs in proofs_vector {
                    for p in proofs.iter() {
                        let proof_resource_def_id = p.resource_def_id();
                        if proof_resource_def_id == *resource_def_id {
                            return Ok(());
                        }
                    }
                }

                Err(NotAuthorized)
            }
            Rule::SomeOfResource(amount, resource_def_id) => {
                for proofs in proofs_vector {
                    for p in proofs.iter() {
                        let proof_resource_def_id = p.resource_def_id();
                        if proof_resource_def_id == *resource_def_id
                            && p.total_amount().as_quantity() >= *amount
                        {
                            return Ok(());
                        }
                    }
                }

                Err(NotAuthorized)
            }
            Rule::AllOf(rules) => {
                for rule in rules {
                    if rule.check(proofs_vector).is_err() {
                        return Err(NotAuthorized);
                    }
                }

                Ok(())
            }
            Rule::OneOf(rules) => {
                for rule in rules {
                    if rule.check(proofs_vector).is_ok() {
                        return Ok(());
                    }
                }

                Err(NotAuthorized)
            }
        }
    }
}

/// Authorization Rule
#[derive(Debug, Clone, PartialEq, Eq, Hash, TypeId, Encode, Decode)]
pub enum AuthRule {
    Protected(Rule),
    Public,
    Private,
    Unsupported,
}

impl AuthRule {
    pub fn check(&self, proofs_vector: &[&[Proof]]) -> Result<(), RuntimeError> {
        match self {
            AuthRule::Protected(rule) => rule.check(proofs_vector),
            AuthRule::Public => Ok(()),
            AuthRule::Private => Err(RuntimeError::NotAuthorized),
            AuthRule::Unsupported => Err(RuntimeError::UnsupportedMethod),
        }
    }
}
