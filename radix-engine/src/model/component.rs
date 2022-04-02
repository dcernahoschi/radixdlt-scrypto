use crate::model::method_authorization::{
    HardAuthRule, HardCount, HardDecimal, HardProofRule, HardProofRuleResourceList,
    HardResourceOrNonFungible,
};
use crate::model::{MethodAuthorization, ValidatedData};
use sbor::any::Value;
use sbor::*;
use scrypto::engine::types::*;
use scrypto::prelude::{AuthRule, SoftCount, SoftDecimal, SoftResource};
use scrypto::resource::{
    NonFungibleAddress, ProofRule, SoftResourceOrNonFungible, SoftResourceOrNonFungibleList,
};
use scrypto::rust::collections::*;
use scrypto::rust::string::String;
use scrypto::rust::vec::Vec;
use scrypto::types::CustomType;

/// A component is an instance of blueprint.
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct Component {
    package_id: PackageId,
    blueprint_name: String,
    auth_rules: HashMap<String, AuthRule>,
    state: Vec<u8>,
}

impl Component {
    pub fn new(
        package_id: PackageId,
        blueprint_name: String,
        auth_rules: HashMap<String, AuthRule>,
        state: Vec<u8>,
    ) -> Self {
        Self {
            package_id,
            blueprint_name,
            auth_rules,
            state,
        }
    }

    fn soft_to_hard_decimal(schema: &Type, soft_decimal: &SoftDecimal, dom: &Value) -> HardDecimal {
        match soft_decimal {
            SoftDecimal::Static(amount) => HardDecimal::Amount(amount.clone()),
            SoftDecimal::Dynamic(schema_path) => {
                let sbor_path = schema_path.to_sbor_path(schema);
                if let None = sbor_path {
                    return HardDecimal::SoftDecimalNotFound;
                }
                match sbor_path.unwrap().get_from_value(dom) {
                    Some(Value::Custom(ty, value)) => match CustomType::from_id(*ty).unwrap() {
                        CustomType::Decimal => {
                            HardDecimal::Amount(Decimal::try_from(value.as_slice()).unwrap())
                        }
                        _ => HardDecimal::SoftDecimalNotFound,
                    },
                    _ => HardDecimal::SoftDecimalNotFound,
                }
            }
        }
    }

    fn soft_to_hard_count(schema: &Type, soft_count: &SoftCount, dom: &Value) -> HardCount {
        match soft_count {
            SoftCount::Static(count) => HardCount::Count(count.clone()),
            SoftCount::Dynamic(schema_path) => {
                let sbor_path = schema_path.to_sbor_path(schema);
                if let None = sbor_path {
                    return HardCount::SoftCountNotFound;
                }
                match sbor_path.unwrap().get_from_value(dom) {
                    Some(Value::U8(count)) => HardCount::Count(count.clone()),
                    _ => HardCount::SoftCountNotFound,
                }
            }
        }
    }

    fn soft_to_hard_resource_list(
        schema: &Type,
        list: &SoftResourceOrNonFungibleList,
        dom: &Value,
    ) -> HardProofRuleResourceList {
        match list {
            SoftResourceOrNonFungibleList::Static(resources) => {
                let mut hard_resources = Vec::new();
                for soft_resource in resources {
                    let resource =
                        Self::soft_to_hard_resource_or_non_fungible(schema, soft_resource, dom);
                    hard_resources.push(resource);
                }
                HardProofRuleResourceList::List(hard_resources)
            }
            SoftResourceOrNonFungibleList::Dynamic(schema_path) => {
                let sbor_path = schema_path.to_sbor_path(schema);
                if let None = sbor_path {
                    return HardProofRuleResourceList::SoftResourceListNotFound;
                }

                match sbor_path.unwrap().get_from_value(dom) {
                    Some(Value::Vec(type_id, values)) => {
                        match CustomType::from_id(*type_id).unwrap() {
                            CustomType::ResourceDefId => HardProofRuleResourceList::List(
                                values
                                    .iter()
                                    .map(|v| {
                                        if let Value::Custom(_, bytes) = v {
                                            return ResourceDefId::try_from(bytes.as_slice())
                                                .unwrap()
                                                .into();
                                        }
                                        panic!("Unexpected type");
                                    })
                                    .collect(),
                            ),
                            CustomType::NonFungibleAddress => HardProofRuleResourceList::List(
                                values
                                    .iter()
                                    .map(|v| {
                                        if let Value::Custom(_, bytes) = v {
                                            return NonFungibleAddress::try_from(bytes.as_slice())
                                                .unwrap()
                                                .into();
                                        }
                                        panic!("Unexpected type");
                                    })
                                    .collect(),
                            ),
                            _ => HardProofRuleResourceList::SoftResourceListNotFound,
                        }
                    }
                    _ => HardProofRuleResourceList::SoftResourceListNotFound,
                }
            }
        }
    }

    fn soft_to_hard_resource(
        schema: &Type,
        soft_resource: &SoftResource,
        dom: &Value,
    ) -> HardResourceOrNonFungible {
        match soft_resource {
            SoftResource::Dynamic(schema_path) => {
                let sbor_path = schema_path.to_sbor_path(schema);
                if let None = sbor_path {
                    return HardResourceOrNonFungible::SoftResourceNotFound;
                }
                match sbor_path.unwrap().get_from_value(dom) {
                    Some(Value::Custom(type_id, bytes)) => {
                        match CustomType::from_id(*type_id).unwrap() {
                            CustomType::ResourceDefId => {
                                ResourceDefId::try_from(bytes.as_slice()).unwrap().into()
                            }
                            _ => HardResourceOrNonFungible::SoftResourceNotFound,
                        }
                    }
                    _ => HardResourceOrNonFungible::SoftResourceNotFound,
                }
            }
            SoftResource::Static(resource_def_id) => {
                HardResourceOrNonFungible::Resource(resource_def_id.clone())
            }
        }
    }

    fn soft_to_hard_resource_or_non_fungible(
        schema: &Type,
        proof_rule_resource: &SoftResourceOrNonFungible,
        dom: &Value,
    ) -> HardResourceOrNonFungible {
        match proof_rule_resource {
            SoftResourceOrNonFungible::Dynamic(schema_path) => {
                let sbor_path = schema_path.to_sbor_path(schema);
                if let None = sbor_path {
                    return HardResourceOrNonFungible::SoftResourceNotFound;
                }
                match sbor_path.unwrap().get_from_value(dom) {
                    Some(Value::Custom(type_id, bytes)) => {
                        match CustomType::from_id(*type_id).unwrap() {
                            CustomType::ResourceDefId => {
                                ResourceDefId::try_from(bytes.as_slice()).unwrap().into()
                            }
                            CustomType::NonFungibleAddress => {
                                NonFungibleAddress::try_from(bytes.as_slice())
                                    .unwrap()
                                    .into()
                            }
                            _ => HardResourceOrNonFungible::SoftResourceNotFound,
                        }
                    }
                    _ => HardResourceOrNonFungible::SoftResourceNotFound,
                }
            }
            SoftResourceOrNonFungible::StaticNonFungible(non_fungible_address) => {
                HardResourceOrNonFungible::NonFungible(non_fungible_address.clone())
            }
            SoftResourceOrNonFungible::StaticResource(resource_def_id) => {
                HardResourceOrNonFungible::Resource(resource_def_id.clone())
            }
        }
    }

    fn soft_to_hard_proof_rule(
        schema: &Type,
        proof_rule: &ProofRule,
        dom: &Value,
    ) -> HardProofRule {
        match proof_rule {
            ProofRule::Require(soft_resource_or_non_fungible) => {
                let resource = Self::soft_to_hard_resource_or_non_fungible(
                    schema,
                    soft_resource_or_non_fungible,
                    dom,
                );
                HardProofRule::This(resource)
            }
            ProofRule::AmountOf(soft_decimal, soft_resource) => {
                let hard_decimal = Self::soft_to_hard_decimal(schema, soft_decimal, dom);
                let resource = Self::soft_to_hard_resource(schema, soft_resource, dom);
                HardProofRule::SomeOfResource(hard_decimal, resource)
            }
            ProofRule::AllOf(resources) => {
                let hard_resources = Self::soft_to_hard_resource_list(schema, resources, dom);
                HardProofRule::AllOf(hard_resources)
            }
            ProofRule::AnyOf(resources) => {
                let hard_resources = Self::soft_to_hard_resource_list(schema, resources, dom);
                HardProofRule::AnyOf(hard_resources)
            }
            ProofRule::CountOf(soft_count, resources) => {
                let hard_count = Self::soft_to_hard_count(schema, soft_count, dom);
                let hard_resources = Self::soft_to_hard_resource_list(schema, resources, dom);
                HardProofRule::CountOf(hard_count, hard_resources)
            }
        }
    }

    fn soft_to_hard_auth_rule(schema: &Type, auth_rule: &AuthRule, dom: &Value) -> HardAuthRule {
        match auth_rule {
            AuthRule::ProofRule(proof_rule) => {
                HardAuthRule::ProofRule(Self::soft_to_hard_proof_rule(schema, proof_rule, dom))
            }
            AuthRule::AnyOf(rules) => {
                let hard_rules = rules
                    .iter()
                    .map(|r| Self::soft_to_hard_auth_rule(schema, r, dom))
                    .collect();
                HardAuthRule::AnyOf(hard_rules)
            }
            AuthRule::AllOf(rules) => {
                let hard_rules = rules
                    .iter()
                    .map(|r| Self::soft_to_hard_auth_rule(schema, r, dom))
                    .collect();
                HardAuthRule::AllOf(hard_rules)
            }
        }
    }

    pub fn initialize_method(
        &self,
        schema: &Type,
        method_name: &str,
    ) -> (ValidatedData, MethodAuthorization) {
        let data = ValidatedData::from_slice(&self.state).unwrap();
        let authorization = match self.auth_rules.get(method_name) {
            Some(auth_rule) => MethodAuthorization::Protected(Self::soft_to_hard_auth_rule(
                schema, auth_rule, &data.dom,
            )),
            None => MethodAuthorization::Public,
        };

        (data, authorization)
    }

    pub fn auth_rules(&self) -> &HashMap<String, AuthRule> {
        &self.auth_rules
    }

    pub fn package_id(&self) -> PackageId {
        self.package_id.clone()
    }

    pub fn blueprint_name(&self) -> &str {
        &self.blueprint_name
    }

    pub fn state(&self) -> &[u8] {
        &self.state
    }

    pub fn set_state(&mut self, new_state: Vec<u8>) {
        self.state = new_state;
    }
}
