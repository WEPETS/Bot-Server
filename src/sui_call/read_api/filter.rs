use move_core_types::{account_address::AccountAddress, language_storage::StructTag};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use sui_json_rpc_types::SuiObjectDataFilter;
use sui_types::{base_types::ObjectID, Identifier, TypeTag};

pub fn new_filter(sui_struct_tag: SuiStructTag) -> SuiObjectDataFilter {
    SuiObjectDataFilter::StructType(StructTag {
        ..sui_struct_tag.into()
    })
}

// region: --- Struct Tag (Builder)
#[derive(Serialize, Deserialize, Debug, PartialEq, Hash, Eq, Clone, PartialOrd, Ord)]
pub struct SuiStructTag {
    pub address: AccountAddress,
    pub module: Identifier,
    pub name: Identifier,
    pub type_params: Vec<TypeTag>,
}

impl SuiStructTag {
    pub fn builder() -> SuiStructTagBuilder {
        SuiStructTagBuilder::default()
    }
}

impl Into<StructTag> for SuiStructTag {
    fn into(self) -> StructTag {
        StructTag {
            address: self.address,
            module: self.module,
            name: self.name,
            type_params: self.type_params,
        }
    }
}

#[derive(Default)]
pub struct SuiStructTagBuilder {
    address: Option<AccountAddress>,
    module: Option<Identifier>,
    name: Option<Identifier>,
    type_params: Option<Vec<TypeTag>>,
}

impl SuiStructTagBuilder {
    pub fn package(mut self, package: &str) -> Self {
        self.address = Some(*ObjectID::from_str(package).unwrap());
        self
    }

    pub fn module(mut self, module: &str) -> Self {
        self.module = Some(Identifier::from_str(module).unwrap());
        self
    }

    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(Identifier::from_str(name).unwrap());
        self
    }

    pub fn type_params(mut self, type_params: Vec<TypeTag>) -> Self {
        self.type_params = Some(type_params);
        self
    }

    pub fn build(self) -> SuiStructTag {
        SuiStructTag {
            address: self.address.unwrap(),
            module: self.module.unwrap(),
            name: self.name.unwrap(),
            type_params: self.type_params.unwrap_or(Vec::new()),
        }
    }
}
// endregion: --- Struct Tag (Builder)
