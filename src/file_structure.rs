use crate::doctavious_error::{
    DoctaviousError, EnumError, Result as DoctavousResult,
};
use crate::utils::parse_enum;
use clap::{ArgEnum, PossibleValue};
use lazy_static::lazy_static;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

lazy_static! {
    pub static ref FILE_STRUCTURES: HashMap<&'static str, FileStructure> = {
        let mut map = HashMap::new();
        map.insert("flat", FileStructure::Flat);
        map.insert("nested", FileStructure::Nested);
        map
    };
}

#[derive(ArgEnum, Clone, Copy, Debug)]
pub enum FileStructure {
    Flat,
    Nested,
}

impl FileStructure {
    pub(crate) fn variants() -> [&'static str; 2] {
        ["flat", "nested"]
    }
}

impl Default for FileStructure {
    fn default() -> Self {
        FileStructure::Flat
    }
}

impl Display for FileStructure {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match *self {
            FileStructure::Flat => write!(f, "flat"),
            FileStructure::Nested => write!(f, "nested"),
        }
    }
}

impl Serialize for FileStructure {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = match *self {
            FileStructure::Flat => "flat",
            FileStructure::Nested => "nested",
        };

        s.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for FileStructure {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let structure = match parse_file_structure(&s) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Error when parsing {}, fallback to default settings. Error: {:?}\n", s, e);
                FileStructure::default()
            }
        };
        Ok(structure)
    }
}

pub(crate) fn parse_file_structure(
    src: &str,
) -> Result<FileStructure, EnumError> {
    parse_enum(&FILE_STRUCTURES, src)
}
