use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

pub const SCHEMA: u32 = 1;

pub const FILE_NAME: &str = "manifest.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub schema: u32,
    pub tag: String,
    pub generated: String,
    pub languages: BTreeMap<String, Language>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Language {
    pub code: String,
    pub name: String,
    pub file: String,
    pub url: String,
    pub sha256: String,
    pub bytes: u64,
    pub entries: u64,
    pub forms: u64,
    pub pct_with_forms: f64,
    pub pct_with_syllables: f64,
    pub license: String,
    pub source: String,
    pub generated: String,
}

impl Manifest {
    pub fn new(tag: String, generated: String) -> Self {
        Self {
            schema: SCHEMA,
            tag,
            generated,
            languages: BTreeMap::new(),
        }
    }

    pub fn get(&self, code: &str) -> Option<&Language> {
        self.languages.get(code)
    }

    pub fn codes(&self) -> impl Iterator<Item = &str> {
        self.languages.keys().map(String::as_str)
    }

    pub fn is_supported(&self) -> bool {
        self.schema == SCHEMA
    }
}
