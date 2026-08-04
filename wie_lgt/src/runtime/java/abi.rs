use alloc::{string::String, vec::Vec};

use serde::Deserialize;

const LGT_JAVA_ABI_TOML: &str = include_str!("../../../../data/lgt_java_abi.toml");

pub const CLASS_NATIVE_NAME_FIELD: &str = "nativeClassName";
pub const CLASS_INITIALIZATION_STATE_FIELD: &str = "initializationState";
pub const WORD_FIELD_DESCRIPTOR: &str = "I";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JavaAbi {
    #[serde(default)]
    pub class: Vec<JavaClassAbi>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JavaClassAbi {
    pub name: String,
    pub field_size: Option<usize>,
    #[serde(default)]
    pub vtable: Vec<JavaVtableIndex>,
    #[serde(default)]
    pub field: Vec<JavaFieldIndex>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JavaVtableIndex {
    pub name: String,
    pub descriptor: String,
    pub index: usize,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JavaFieldIndex {
    pub name: String,
    pub descriptor: String,
    pub index: u32,
}

impl JavaAbi {
    pub fn parse() -> Self {
        toml::from_str(LGT_JAVA_ABI_TOML).expect("parse data/lgt_java_abi.toml")
    }

    pub fn vtable_index(&self, class_name: &str, method_name: &str, descriptor: &str) -> Option<usize> {
        self.class
            .iter()
            .find(|class| class.name == class_name)?
            .vtable
            .iter()
            .find(|entry| entry.name == method_name && entry.descriptor == descriptor)
            .map(|entry| entry.index)
    }
}
