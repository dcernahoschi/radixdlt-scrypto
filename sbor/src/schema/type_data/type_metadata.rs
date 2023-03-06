use crate::rust::prelude::*;
use crate::*;

/// This is the struct used in the Schema
#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
pub struct NovelTypeMetadata {
    pub type_hash: TypeHash,
    pub type_metadata: TypeMetadata,
}

/// This enables the type to be represented as eg JSON
/// Also used to facilitate type reconstruction
#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
pub struct TypeMetadata {
    pub type_name: Cow<'static, str>,
    pub child_names: Option<ChildNames>,
}

impl TypeMetadata {
    pub fn no_child_names(name: &'static str) -> Self {
        Self {
            type_name: Cow::Borrowed(name),
            child_names: None,
        }
    }

    pub fn struct_fields(name: &'static str, field_names: &[&'static str]) -> Self {
        let field_names = field_names
            .iter()
            .map(|field_name| Cow::Borrowed(*field_name))
            .collect();
        Self {
            type_name: Cow::Borrowed(name),
            child_names: Some(ChildNames::NamedFields(field_names)),
        }
    }

    pub fn enum_variants(name: &'static str, variant_naming: BTreeMap<u8, TypeMetadata>) -> Self {
        Self {
            type_name: Cow::Borrowed(name),
            child_names: Some(ChildNames::EnumVariants(variant_naming)),
        }
    }

    pub fn with_type_hash(self, type_hash: TypeHash) -> NovelTypeMetadata {
        NovelTypeMetadata {
            type_hash,
            type_metadata: self,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
pub enum ChildNames {
    NamedFields(Vec<Cow<'static, str>>),
    EnumVariants(BTreeMap<u8, TypeMetadata>),
}
